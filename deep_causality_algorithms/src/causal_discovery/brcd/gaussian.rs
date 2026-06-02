/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Plug-in ridge-Gaussian continuous family estimator for BRCD.
//!
//! For a family `(node, parents)` the authoritative `brcd.py`
//! (`gaussian_conditional_postpred_rowwise` / `_fit_ridge`) scores each row with
//! a plug-in linear-Gaussian: ridge least squares for the conditional mean, the
//! residual variance as the (floored) variance, and the 1-D normal log-density
//! per row, with an optional monotone transform of the node and its Jacobian on
//! the original scale.
//!
//! This module ports that single-expert scorer and its pieces:
//! * [`fit_ridge`] — `β = solve(XᵀX + λI, Xᵀy)`, `σ² = ‖resid‖² / max(n−p, 1)`
//!   floored to `1e-12`, via the shared dense SPD solver
//!   [`crate::causal_discovery::brcd::linalg::solve_spd`].
//! * [`transform_and_jacobian`] and [`effective_transform`] — the
//!   none/log/log1p transform ladder with its `log → log1p → yeojohnson`
//!   auto-downgrade (Yeo-Johnson is deferred; see design D7).
//! * [`gaussian_single_expert_logdensity`] — the per-row log-density of the
//!   single-expert family, composing [`deep_causality_tensor`]'s
//!   `gaussian_log_density` (the exact `_normal_logpdf_1d`) on the per-row
//!   residual, plus the transform Jacobian.
//!
//! The F-node integration (per-regime fits and the mixture-of-experts gate)
//! builds on this in the next stage.

use crate::causal_discovery::brcd::linalg::solve_spd;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::{CausalTensor, CausalTensorStatsExt};

/// Default ridge `λ` for the conditional-mean fit (matches `brcd.py`'s `1e-4`).
pub const RIDGE_DEFAULT: f64 = 1e-4;

/// The variance floor shared with `gaussian_log_density` (`1e-12`).
const VARIANCE_FLOOR: f64 = 1e-12;

/// Monotone transform applied to the node before fitting, with its Jacobian
/// taken on the original scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    /// Identity; Jacobian `0`.
    None,
    /// `z = ln(x)`, requires `x > 0`; `log|dz/dx| = −ln(x)`.
    Log,
    /// `z = ln(1 + x)`, requires `x ≥ −1`; `log|dz/dx| = −ln(1 + x)`.
    Log1p,
    /// Yeo-Johnson — **deferred** (design D7). Selected by the auto-downgrade
    /// ladder for data with values `< −1`; scoring it returns
    /// [`GaussianError::YeojohnsonUnsupported`].
    Yeojohnson,
}

/// Reasons a Gaussian family could not be scored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GaussianError {
    /// No observations were supplied.
    EmptyData,
    /// The parent-row count or width does not match the node values.
    DimensionMismatch,
    /// A transform was applied to data outside its domain after the
    /// auto-downgrade ladder (should not occur when `effective_transform`
    /// selects the transform).
    InvalidTransformDomain,
    /// The Yeo-Johnson transform is not yet implemented (deferred; design D7).
    YeojohnsonUnsupported,
}

/// A fitted ridge least-squares regression.
#[derive(Debug, Clone, PartialEq)]
pub struct RidgeFit<T> {
    /// Coefficients, one per design column (the caller's design includes the
    /// intercept column).
    pub beta: Vec<T>,
    /// Residual variance, floored to `1e-12`.
    pub sigma2: T,
}

impl<T: RealField> RidgeFit<T> {
    /// Predicts the mean for one design row (intercept column included).
    pub fn predict(&self, design_row: &[T]) -> T {
        dot(&self.beta, design_row)
    }
}

/// Fits ridge least squares `β = solve(XᵀX + λI, Xᵀy)` with residual variance
/// `σ² = ‖y − Xβ‖² / max(n − p, 1)`, floored to `1e-12`.
///
/// `x` is `n` design rows, each of width `p` **including the intercept column**
/// (faithful to `_fit_ridge`, which penalizes every column — intercept
/// included). `ridge` is `λ`.
///
/// # Errors
/// [`GaussianError::EmptyData`] if `x` is empty, [`GaussianError::DimensionMismatch`]
/// if `y.len() != x.len()`, the rows are ragged, or a row is empty.
pub fn fit_ridge<T: RealField + FromPrimitive>(
    x: &[Vec<T>],
    y: &[T],
    ridge: T,
) -> Result<RidgeFit<T>, GaussianError> {
    let n = x.len();
    if n == 0 {
        return Err(GaussianError::EmptyData);
    }
    if y.len() != n {
        return Err(GaussianError::DimensionMismatch);
    }
    let p = x[0].len();
    if p == 0 || x.iter().any(|r| r.len() != p) {
        return Err(GaussianError::DimensionMismatch);
    }

    // Normal equations: XtX = XᵀX + λI (p × p), Xty = Xᵀy (p).
    let mut xtx = vec![T::zero(); p * p];
    let mut xty = vec![T::zero(); p];
    for (row, &yi) in x.iter().zip(y.iter()) {
        for a in 0..p {
            xty[a] += row[a] * yi;
            let ra = row[a];
            for b in 0..p {
                xtx[a * p + b] += ra * row[b];
            }
        }
    }
    for a in 0..p {
        xtx[a * p + a] += ridge;
    }

    // Solve in place; xty becomes β.
    solve_spd(&mut xtx, &mut xty, p);
    let beta = xty;

    // Residual variance with dof = max(n − p, 1), floored.
    let mut rss = T::zero();
    for (row, &yi) in x.iter().zip(y.iter()) {
        let r = yi - dot(&beta, row);
        rss += r * r;
    }
    let dof = t_usize::<T>(n.saturating_sub(p).max(1));
    let sigma2 = floor(rss / dof, from_f64::<T>(VARIANCE_FLOOR));

    Ok(RidgeFit { beta, sigma2 })
}

/// Returns the effective transform after the `log → log1p → yeojohnson`
/// auto-downgrade ladder, given the node `values` (port of `brcd.py`'s
/// `eff_transform` selection):
/// * `log` downgrades to `log1p` if any value `≤ 0`, or to `yeojohnson` if any
///   value `< −1`;
/// * `log1p` downgrades to `yeojohnson` if any value `< −1`;
/// * `none` and `yeojohnson` are returned unchanged.
pub fn effective_transform<T: RealField>(values: &[T], requested: Transform) -> Transform {
    let neg_one = -T::one();
    let zero = T::zero();
    let any_lt_neg1 = values.iter().any(|&v| v < neg_one);
    match requested {
        Transform::None | Transform::Yeojohnson => requested,
        Transform::Log => {
            if any_lt_neg1 {
                Transform::Yeojohnson
            } else if values.iter().any(|&v| v <= zero) {
                Transform::Log1p
            } else {
                Transform::Log
            }
        }
        Transform::Log1p => {
            if any_lt_neg1 {
                Transform::Yeojohnson
            } else {
                Transform::Log1p
            }
        }
    }
}

/// Transforms one value, returning `(z, log|dz/dx|)`.
///
/// # Errors
/// [`GaussianError::InvalidTransformDomain`] if the value is outside the
/// transform's domain; [`GaussianError::YeojohnsonUnsupported`] for Yeo-Johnson.
pub fn transform_and_jacobian<T: RealField>(
    x: T,
    kind: Transform,
) -> Result<(T, T), GaussianError> {
    match kind {
        Transform::None => Ok((x, T::zero())),
        Transform::Log => {
            if x <= T::zero() {
                return Err(GaussianError::InvalidTransformDomain);
            }
            let lx = x.ln();
            Ok((lx, -lx))
        }
        Transform::Log1p => {
            if x < -T::one() {
                return Err(GaussianError::InvalidTransformDomain);
            }
            let l1p = (T::one() + x).ln();
            Ok((l1p, -l1p))
        }
        Transform::Yeojohnson => Err(GaussianError::YeojohnsonUnsupported),
    }
}

/// Per-row log-density of the single-expert ridge-Gaussian family `p(node |
/// parents)`, on the original (untransformed) scale.
///
/// `y` is the node's `n` values; `parents` is `n` rows of `p` parent features
/// (no intercept column), or empty for a parentless family. The node transform
/// is chosen by [`effective_transform`], applied with its Jacobian; the mean is
/// the ridge fit's prediction (or the sample mean when no finite rows are
/// available), and the variance is the fit's residual variance (or the sample
/// variance). The returned vector holds the per-row log-densities
/// `logpdf(z; μ_i, σ²) + log|dz/dx|_i`, composing
/// `deep_causality_tensor::gaussian_log_density` on the residual.
///
/// # Errors
/// As [`fit_ridge`] and [`transform_and_jacobian`].
pub fn gaussian_single_expert_logdensity<T: RealField + FromPrimitive>(
    y: &[T],
    parents: &[Vec<T>],
    transform: Transform,
    ridge: T,
) -> Result<Vec<T>, GaussianError> {
    let n = y.len();
    if n == 0 {
        return Err(GaussianError::EmptyData);
    }
    let p_feat = parents.first().map_or(0, Vec::len);
    if !parents.is_empty() && (parents.len() != n || parents.iter().any(|r| r.len() != p_feat)) {
        return Err(GaussianError::DimensionMismatch);
    }

    // Transform the node and capture the per-row Jacobian.
    let eff = effective_transform(y, transform);
    let mut z = Vec::with_capacity(n);
    let mut log_jac = Vec::with_capacity(n);
    for &yi in y {
        let (zi, ji) = transform_and_jacobian(yi, eff)?;
        z.push(zi);
        log_jac.push(ji);
    }

    // Conditional mean per row and the shared residual variance.
    let (mu, sigma2): (Vec<T>, T) = if p_feat > 0 {
        // Fit on the finite rows (ridge keeps the solve well-posed even if
        // n ≤ p); predict the mean for every row.
        let mut x_fit = Vec::new();
        let mut z_fit = Vec::new();
        for i in 0..n {
            if z[i].is_finite() && parents[i].iter().all(|v| v.is_finite()) {
                x_fit.push(design_row(&parents[i]));
                z_fit.push(z[i]);
            }
        }
        if x_fit.is_empty() {
            (vec![mean(&z); n], variance_ddof1(&z))
        } else {
            let fit = fit_ridge(&x_fit, &z_fit, ridge)?;
            let mus = (0..n)
                .map(|i| fit.predict(&design_row(&parents[i])))
                .collect();
            (mus, fit.sigma2)
        }
    } else {
        (vec![mean(&z); n], variance_ddof1(&z))
    };

    // Residual r_i = z_i − μ_i; gaussian_log_density(0, σ²) is logpdf(z; μ, σ²).
    let residuals: Vec<T> = z.iter().zip(mu.iter()).map(|(&zi, &mi)| zi - mi).collect();
    let logdens = CausalTensor::from_slice(&residuals, &[n])
        .gaussian_log_density(T::zero(), sigma2)
        .expect("residual tensor has a valid 1-D shape");

    Ok(logdens
        .as_slice()
        .iter()
        .zip(log_jac.iter())
        .map(|(&ld, &j)| ld + j)
        .collect())
}

// --- helpers ----------------------------------------------------------------

/// Builds the design row `[1, features...]` (intercept prepended).
fn design_row<T: RealField>(features: &[T]) -> Vec<T> {
    let mut row = Vec::with_capacity(features.len() + 1);
    row.push(T::one());
    row.extend_from_slice(features);
    row
}

/// Dot product over the shorter length.
fn dot<T: RealField>(a: &[T], b: &[T]) -> T {
    a.iter()
        .zip(b.iter())
        .fold(T::zero(), |acc, (&x, &y)| acc + x * y)
}

/// Sample mean.
fn mean<T: RealField + FromPrimitive>(v: &[T]) -> T {
    if v.is_empty() {
        return T::zero();
    }
    v.iter().fold(T::zero(), |acc, &x| acc + x) / t_usize::<T>(v.len())
}

/// Sample variance with Bessel's correction; `1` when fewer than two values
/// (matching `brcd.py`'s fallback).
fn variance_ddof1<T: RealField + FromPrimitive>(v: &[T]) -> T {
    if v.len() < 2 {
        return T::one();
    }
    let m = mean(v);
    let ss = v.iter().fold(T::zero(), |acc, &x| {
        let d = x - m;
        acc + d * d
    });
    ss / t_usize::<T>(v.len() - 1)
}

/// Returns `x` if it exceeds `floor`, else `floor`.
fn floor<T: RealField>(x: T, floor: T) -> T {
    if x > floor { x } else { floor }
}

/// `T` from a `usize`.
fn t_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}

/// `T` from an `f64` constant.
fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
