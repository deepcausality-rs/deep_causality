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

use crate::causal_discovery::brcd::linalg::solve_spd;
use deep_causality_num::{FromPrimitive, RealField};

/// Reasons a gate could not be fit. The caller falls back to the empirical F
/// prior on any of these (mirroring the reference's `except` path).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateError {
    /// No rows were supplied.
    EmptyData,
    /// The label count does not match the row count, or the rows are ragged.
    DimensionMismatch,
    /// The Newton iteration produced a non-finite parameter (e.g. a degenerate
    /// design that ridge did not regularize).
    SingularSystem,
}

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
/// * [`GateError::EmptyData`] if `rows` is empty.
/// * [`GateError::DimensionMismatch`] if `y.len() != rows.len()` or the rows are
///   ragged.
/// * [`GateError::SingularSystem`] if the Newton iteration diverges to a
///   non-finite parameter.
pub fn fit_logistic_gate<T: RealField + FromPrimitive>(
    rows: &[Vec<T>],
    y: &[bool],
    config: &GateConfig<T>,
) -> Result<LogisticGate<T>, GateError> {
    let n = rows.len();
    if n == 0 {
        return Err(GateError::EmptyData);
    }
    if y.len() != n {
        return Err(GateError::DimensionMismatch);
    }
    let p = rows[0].len();
    if rows.iter().any(|r| r.len() != p) {
        return Err(GateError::DimensionMismatch);
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

    // IRLS / Newton on θ = (bias, weights), with the design row z_i = [1, x_i].
    let dim = p + 1;
    let mut theta = vec![T::zero(); dim];

    for _ in 0..config.max_iter {
        // Gradient g = Zᵀ(π − y) + Λθ and Hessian H = ZᵀWZ + Λ, where
        // W = diag(π_i(1−π_i)) and Λ penalizes the weights but not the intercept.
        let mut grad = vec![T::zero(); dim];
        let mut hess = vec![T::zero(); dim * dim];

        for (row, &label) in rows.iter().zip(y.iter()) {
            let eta = theta[0] + dot(&theta[1..], row);
            let pi = sigmoid(eta);
            let w = pi * (T::one() - pi);
            let resid = pi - if label { T::one() } else { T::zero() };

            // z_i = [1, row...]; accumulate gradient and Hessian.
            accumulate_row(&mut grad, &mut hess, dim, row, resid, w);
        }

        // Ridge on the weights (indices 1..dim), intercept untouched.
        for a in 1..dim {
            grad[a] += config.ridge * theta[a];
            hess[a * dim + a] += config.ridge;
        }

        // Newton step: solve H·step = grad, then θ ← θ − step.
        solve_spd(&mut hess, &mut grad, dim);
        let mut max_step = T::zero();
        for a in 0..dim {
            theta[a] -= grad[a];
            let s = grad[a].abs();
            if s > max_step {
                max_step = s;
            }
        }
        if theta.iter().any(|t| !t.is_finite()) {
            return Err(GateError::SingularSystem);
        }
        if max_step < config.tol {
            break;
        }
    }

    Ok(LogisticGate {
        bias: theta[0],
        weights: theta[1..].to_vec(),
    })
}

// --- helpers ----------------------------------------------------------------

/// Adds row `i`'s contribution to the gradient and Hessian, with the implicit
/// intercept column `z_i[0] = 1`.
fn accumulate_row<T: RealField>(
    grad: &mut [T],
    hess: &mut [T],
    dim: usize,
    row: &[T],
    resid: T,
    w: T,
) {
    // z_a is 1 for a == 0, else row[a-1].
    let z = |a: usize| if a == 0 { T::one() } else { row[a - 1] };
    for a in 0..dim {
        let za = z(a);
        grad[a] += za * resid;
        let zaw = za * w;
        for b in 0..dim {
            hess[a * dim + b] += zaw * z(b);
        }
    }
}

/// Dot product of `a` and `b` (truncated to the shorter length).
fn dot<T: RealField>(a: &[T], b: &[T]) -> T {
    a.iter()
        .zip(b.iter())
        .fold(T::zero(), |acc, (&x, &y)| acc + x * y)
}

/// Numerically-stable logistic sigmoid `1 / (1 + e^{−x})`.
fn sigmoid<T: RealField>(x: T) -> T {
    let one = T::one();
    if x >= T::zero() {
        one / (one + (-x).exp())
    } else {
        let e = x.exp();
        e / (one + e)
    }
}

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
