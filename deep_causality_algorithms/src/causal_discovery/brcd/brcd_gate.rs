/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Logistic-regression gate `π(F = 1 | X)` for BRCD's mixture-of-experts
//! F-integration.
//!
//! When the F-node is present but not a parent of a family's node, BRCD
//! integrates it as a mixture of two regime experts combined through a gate
//! probability `π(X) = P(F = 1 | X)` (authoritative `brcd.py`, the `gating="auto"`
//! branch around L534). The reference fits an L2-penalized logistic regression
//! (`sklearn.LogisticRegression`, `lbfgs`, `C = 1.0`, intercept fit and
//! unpenalized) and reads `predict_proba(X)[:, 1]`.
//!
//! This is the in-repo port: ridge-penalized logistic regression solved by
//! Newton / iteratively-reweighted-least-squares (IRLS), generic over
//! `T: RealField`, deterministic, with no external crate. The objective matches
//! sklearn's default —
//! `min_{w,b}  0.5·ridge·‖w‖²  +  Σ_i log(1 + exp(−ỹ_i (w·x_i + b)))`
//! with `ỹ ∈ {−1, +1}` and the intercept `b` unpenalized — so the gate
//! reproduces the reference's gating closely enough for ranking. When the label
//! has a single class the gate degenerates to the constant base rate (matching
//! the reference's behaviour and its empirical-prior fallback).

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_stats::{LogisticConfig, Penalisation, fit_logistic, sigmoid};

/// Configuration for the logistic-gate fit.
#[derive(Debug, Clone, Copy)]
pub struct GateConfig<T> {
    /// L2 penalty on the weights (not the intercept). Default `1.0`, matching
    /// `sklearn.LogisticRegression`'s `C = 1.0`.
    pub ridge: T,
    /// Maximum Newton iterations.
    pub max_iter: usize,
    /// Convergence tolerance on the maximum absolute Newton step.
    pub tol: T,
}

impl<T: RealField + FromPrimitive> Default for GateConfig<T> {
    fn default() -> Self {
        Self {
            ridge: T::one(),
            max_iter: 100,
            tol: from_f64::<T>(1e-8),
        }
    }
}

/// A fitted logistic gate: `π(x) = sigmoid(bias + weights·x)`.
#[derive(Debug, Clone, PartialEq)]
pub struct LogisticGate<T> {
    bias: T,
    weights: Vec<T>,
}

impl<T: RealField + FromPrimitive> LogisticGate<T> {
    /// Returns the gate probability `P(F = 1 | x)` for one feature row `x`
    /// (without an intercept column; the intercept is held internally).
    pub fn predict_proba(&self, x: &[T]) -> T {
        let mut eta = self.bias;
        for (w, &xi) in self.weights.iter().zip(x.iter()) {
            eta += *w * xi;
        }
        sigmoid(eta)
    }

    /// The fitted intercept.
    pub fn bias(&self) -> T {
        self.bias
    }

    /// The fitted weights, one per feature.
    pub fn weights(&self) -> &[T] {
        &self.weights
    }
}

/// Fits the logistic gate on feature rows `rows` (each row is `p` features, no
/// intercept column) against binary labels `y`.
///
/// # Errors
/// * [`BrcdErrorEnum::EmptyData`] if `rows` is empty.
/// * [`BrcdErrorEnum::DimensionMismatch`] if `y.len() != rows.len()` or the rows are
///   ragged.
/// * [`BrcdErrorEnum::SingularSystem`] if the Newton iteration diverges to a
///   non-finite parameter.
pub fn fit_logistic_gate<T: RealField + FromPrimitive>(
    rows: &[Vec<T>],
    y: &[bool],
    config: &GateConfig<T>,
) -> Result<LogisticGate<T>, BrcdError> {
    let n = rows.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    if y.len() != n {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    let p = rows[0].len();
    if rows.iter().any(|r| r.len() != p) {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }

    // Single-class label → the gate is the constant base rate (0 or 1), matching
    // the reference's empirical-prior fallback. Logistic regression would push
    // the unpenalized intercept to ±∞ here; we encode that directly.
    let ones = y.iter().filter(|&&v| v).count();
    if ones == 0 || ones == n {
        let rate = from_f64::<T>(ones as f64) / from_f64::<T>(n as f64);
        return Ok(LogisticGate {
            bias: logit_clamped(rate),
            weights: vec![T::zero(); p],
        });
    }

    // The IRLS core is `deep_causality_stats::fit_logistic`. Three adaptations, each of which is
    // this gate's contract rather than the crate's:
    //
    // 1. The crate fits one coefficient per design column and has no intercept of its own, so the
    //    ones-column that was implicit in `θ = (bias, weights)` is now written into the design.
    // 2. That column is exempted from the penalty — `Penalisation::Excluding(0)` — which is what
    //    keeps this a port of sklearn's default rather than a different objective. Penalising it
    //    would shrink the fitted odds toward even and discard the base rate.
    // 3. The crate refuses to return an unconverged iterate; this gate returned its last one. The
    //    refusal is propagated, because `gate_probabilities` already answers a failed gate with the
    //    empirical base rate — which is a better estimate than a half-finished Newton step.
    let design: Vec<Vec<T>> = rows
        .iter()
        .map(|row| {
            let mut z = Vec::with_capacity(p + 1);
            z.push(T::one());
            z.extend_from_slice(row);
            z
        })
        .collect();
    let labels: Vec<T> = y
        .iter()
        .map(|&v| if v { T::one() } else { T::zero() })
        .collect();

    let fit = fit_logistic(
        &design,
        &labels,
        &LogisticConfig::new(config.ridge, config.max_iter, config.tol)
            .with_penalisation(Penalisation::Excluding(0)),
    )
    .map_err(|_| BrcdError(BrcdErrorEnum::SingularSystem))?;

    Ok(LogisticGate {
        bias: fit.beta[0],
        weights: fit.beta[1..].to_vec(),
    })
}

// --- helpers ----------------------------------------------------------------

/// `logit(p) = ln(p / (1 − p))`, with `p` clamped away from `0` and `1` so the
/// result is finite.
fn logit_clamped<T: RealField + FromPrimitive>(p: T) -> T {
    let eps = from_f64::<T>(1e-12);
    let one = T::one();
    let clamped = p.clamp(eps, one - eps);
    (clamped / (one - clamped)).ln()
}

/// Constructs a `T` from an `f64` constant that is representable in every
/// `RealField`.
fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
