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
//!   [`crate::causal_discovery::brcd::brcd_linalg::solve_spd`].
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

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::causal_discovery::brcd::brcd_gate::{GateConfig, fit_logistic_gate};
use crate::causal_discovery::brcd::brcd_linalg::solve_spd;
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
    /// [`BrcdErrorEnum::YeojohnsonUnsupported`].
    Yeojohnson,
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
/// [`BrcdErrorEnum::EmptyData`] if `x` is empty, [`BrcdErrorEnum::DimensionMismatch`]
/// if `y.len() != x.len()`, the rows are ragged, or a row is empty.
pub fn fit_ridge<T: RealField + FromPrimitive>(
    x: &[Vec<T>],
    y: &[T],
    ridge: T,
) -> Result<RidgeFit<T>, BrcdError> {
    let n = x.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    if y.len() != n {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    let p = x[0].len();
    if p == 0 || x.iter().any(|r| r.len() != p) {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
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
/// [`BrcdErrorEnum::InvalidTransformDomain`] if the value is outside the
/// transform's domain; [`BrcdErrorEnum::YeojohnsonUnsupported`] for Yeo-Johnson.
pub fn transform_and_jacobian<T: RealField>(x: T, kind: Transform) -> Result<(T, T), BrcdError> {
    match kind {
        Transform::None => Ok((x, T::zero())),
        Transform::Log => {
            if x <= T::zero() {
                return Err(BrcdError(BrcdErrorEnum::InvalidTransformDomain));
            }
            let lx = x.ln();
            Ok((lx, -lx))
        }
        Transform::Log1p => {
            if x < -T::one() {
                return Err(BrcdError(BrcdErrorEnum::InvalidTransformDomain));
            }
            let l1p = (T::one() + x).ln();
            Ok((l1p, -l1p))
        }
        Transform::Yeojohnson => Err(BrcdError(BrcdErrorEnum::YeojohnsonUnsupported)),
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
) -> Result<Vec<T>, BrcdError> {
    let n = y.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    let p_feat = parents.first().map_or(0, Vec::len);
    if !parents.is_empty() && (parents.len() != n || parents.iter().any(|r| r.len() != p_feat)) {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
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

    let logdens = logpdf_rows(&z, &mu, sigma2);
    Ok(add_jacobian(logdens, &log_jac))
}

/// Configuration for the F-integrated continuous family scorer.
#[derive(Debug, Clone)]
pub struct GaussianFamilyConfig<T> {
    /// Node transform (with auto-downgrade).
    pub transform: Transform,
    /// Apply the node's effective transform to continuous parents (no Jacobian).
    pub transform_parents: bool,
    /// Ridge `λ` for the conditional-mean fits.
    pub ridge: T,
    /// Logistic-gate configuration for the F-not-parent mixture.
    pub gate: GateConfig<T>,
}

impl<T: RealField + FromPrimitive> Default for GaussianFamilyConfig<T> {
    fn default() -> Self {
        Self {
            transform: Transform::None,
            transform_parents: false,
            ridge: from_f64::<T>(RIDGE_DEFAULT),
            gate: GateConfig::default(),
        }
    }
}

/// Per-row log-density of the continuous family `p(node | parents)` with the
/// F-node integrated, porting `gaussian_conditional_postpred_rowwise` (brcd.py
/// L324–L552). `parents` holds the **continuous** (non-F) parent features.
///
/// * `f = None` — F is absent from the data: a single expert over `parents`.
/// * `f = Some(_)` and `f_is_parent` — F conditions the family: a separate
///   ridge-Gaussian per regime (`F = 0` / `F = 1`), each row scored within its
///   own regime.
/// * `f = Some(_)` and not `f_is_parent` — F is integrated as a mixture of two
///   regime experts combined through the logistic gate `π(F = 1 | parents)`,
///   `log P = logsumexp(log(1−π) + logN₀, log π + logN₁)`.
///
/// # Errors
/// As [`fit_ridge`] / [`transform_and_jacobian`]; [`BrcdErrorEnum::DimensionMismatch`]
/// if `f` or `parents` lengths disagree with `node`.
pub fn gaussian_family_logdensity<T: RealField + FromPrimitive>(
    node: &[T],
    parents: &[Vec<T>],
    f: Option<&[bool]>,
    f_is_parent: bool,
    config: &GaussianFamilyConfig<T>,
) -> Result<Vec<T>, BrcdError> {
    let n = node.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    let p = parents.first().map_or(0, Vec::len);
    if !parents.is_empty() && (parents.len() != n || parents.iter().any(|r| r.len() != p)) {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }
    if let Some(fv) = f
        && fv.len() != n
    {
        return Err(BrcdError(BrcdErrorEnum::DimensionMismatch));
    }

    // Node transform (with Jacobian) and the matching parent transform (no Jacobian).
    let eff = effective_transform(node, config.transform);
    let mut z = Vec::with_capacity(n);
    let mut log_jac = Vec::with_capacity(n);
    for &yi in node {
        let (zi, ji) = transform_and_jacobian(yi, eff)?;
        z.push(zi);
        log_jac.push(ji);
    }
    let parents_t = apply_parent_transform(parents, eff, config.transform_parents)?;

    let logdens = match (f, f_is_parent) {
        // F absent → a single expert over all rows.
        (None, _) => {
            let all: Vec<usize> = (0..n).collect();
            let (mean_model, var) = fit_expert(&all, &z, &parents_t, config.ridge);
            let mu = predict_all(&mean_model, &parents_t, n);
            logpdf_rows(&z, &mu, var)
        }
        // F is a parent → per-regime fit; each row scored within its regime.
        (Some(fv), true) => {
            let mut out = vec![T::zero(); n];
            for regime in [false, true] {
                let idxs: Vec<usize> = (0..n).filter(|&i| fv[i] == regime).collect();
                if idxs.is_empty() {
                    continue;
                }
                let (mean_model, var) = fit_expert_guarded(&idxs, &z, &parents_t, config.ridge);
                for &i in &idxs {
                    let mu = mean_model.predict(parents_t.get(i).map_or(&[][..], Vec::as_slice));
                    out[i] = single_logpdf(z[i], mu, var);
                }
            }
            out
        }
        // F present but not a parent → mixture of experts through the gate.
        (Some(fv), false) => {
            let idx0: Vec<usize> = (0..n).filter(|&i| !fv[i]).collect();
            let idx1: Vec<usize> = (0..n).filter(|&i| fv[i]).collect();
            let (m0, var0) = fit_expert(&idx0, &z, &parents_t, config.ridge);
            let (m1, var1) = fit_expert(&idx1, &z, &parents_t, config.ridge);
            let mu0 = predict_all(&m0, &parents_t, n);
            let mu1 = predict_all(&m1, &parents_t, n);
            let log_n0 = logpdf_rows(&z, &mu0, var0);
            let log_n1 = logpdf_rows(&z, &mu1, var1);
            let pi1 = gate_probabilities(&parents_t, fv, &config.gate, idx1.len(), n);

            let one = T::one();
            (0..n)
                .map(|i| {
                    let p1 = clamp_unit(pi1[i]);
                    logaddexp((one - p1).ln() + log_n0[i], p1.ln() + log_n1[i])
                })
                .collect()
        }
    };

    Ok(add_jacobian(logdens, &log_jac))
}

// --- helpers ----------------------------------------------------------------

/// A fitted expert's conditional mean: a constant, or a linear predictor whose
/// `beta` includes the intercept.
enum ExpertMean<T> {
    Const(T),
    Linear(Vec<T>),
}

impl<T: RealField> ExpertMean<T> {
    /// Predicts the mean for one row's `features` (no intercept column).
    fn predict(&self, features: &[T]) -> T {
        match self {
            ExpertMean::Const(m) => *m,
            ExpertMean::Linear(beta) => dot(beta, &design_row(features)),
        }
    }
}

/// Predicts the conditional mean for every row.
fn predict_all<T: RealField>(model: &ExpertMean<T>, parents: &[Vec<T>], n: usize) -> Vec<T> {
    (0..n)
        .map(|i| model.predict(parents.get(i).map_or(&[][..], Vec::as_slice)))
        .collect()
}

/// Fits one expert on the rows `idxs`, returning its conditional-mean model and
/// residual variance. Mirrors `_fit_expert` (brcd.py L499–516): an empty regime
/// falls back to the full-column sample mean/variance; with no finite rows it
/// falls back to the regime's mean/variance; otherwise a ridge fit (ridge keeps
/// the solve well-posed even when `n ≤ p`).
fn fit_expert<T: RealField + FromPrimitive>(
    idxs: &[usize],
    z_all: &[T],
    parents_t: &[Vec<T>],
    ridge: T,
) -> (ExpertMean<T>, T) {
    if idxs.is_empty() {
        return (ExpertMean::Const(mean(z_all)), variance_ddof1(z_all));
    }
    let p = parents_t.first().map_or(0, Vec::len);
    if p == 0 {
        let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
        return (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys));
    }
    let (x_fit, z_fit) = finite_design(idxs, z_all, parents_t);
    if x_fit.is_empty() {
        let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
        return (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys));
    }
    let fit = fit_ridge(&x_fit, &z_fit, ridge).expect("validated finite design");
    (ExpertMean::Linear(fit.beta), fit.sigma2)
}

/// As [`fit_expert`], but with the `n ≤ p` guard of the F-as-parent branch
/// (brcd.py L433–438): too few finite rows for the design fall back to the
/// regime's sample mean/variance instead of a ridge fit.
fn fit_expert_guarded<T: RealField + FromPrimitive>(
    idxs: &[usize],
    z_all: &[T],
    parents_t: &[Vec<T>],
    ridge: T,
) -> (ExpertMean<T>, T) {
    let p = parents_t.first().map_or(0, Vec::len);
    if p == 0 {
        let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
        return (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys));
    }
    let (x_fit, z_fit) = finite_design(idxs, z_all, parents_t);
    if x_fit.len() <= p + 1 {
        let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
        return (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys));
    }
    let fit = fit_ridge(&x_fit, &z_fit, ridge).expect("validated finite design");
    (ExpertMean::Linear(fit.beta), fit.sigma2)
}

/// Collects the finite-row design `[1, parents_i]` and targets `z_i` over `idxs`.
fn finite_design<T: RealField>(
    idxs: &[usize],
    z_all: &[T],
    parents_t: &[Vec<T>],
) -> (Vec<Vec<T>>, Vec<T>) {
    let mut x_fit = Vec::new();
    let mut z_fit = Vec::new();
    for &i in idxs {
        if z_all[i].is_finite() && parents_t[i].iter().all(|v| v.is_finite()) {
            x_fit.push(design_row(&parents_t[i]));
            z_fit.push(z_all[i]);
        }
    }
    (x_fit, z_fit)
}

/// Per-row gate probability `π(F = 1 | parents)` via the logistic gate, with the
/// empirical prior `|F=1| / n` as the fallback when the fit fails.
fn gate_probabilities<T: RealField + FromPrimitive>(
    parents_t: &[Vec<T>],
    f: &[bool],
    gate: &GateConfig<T>,
    ones: usize,
    n: usize,
) -> Vec<T> {
    // The gate features are the (possibly empty) parent rows; an empty feature
    // row degenerates the gate to the base rate, matching the reference's
    // ones((n,1)) design when there are no parents.
    let rows: Vec<Vec<T>> = (0..n)
        .map(|i| parents_t.get(i).cloned().unwrap_or_default())
        .collect();
    match fit_logistic_gate(&rows, f, gate) {
        Ok(model) => (0..n).map(|i| model.predict_proba(&rows[i])).collect(),
        Err(_) => {
            let prior = from_f64::<T>(ones as f64) / from_f64::<T>(n.max(1) as f64);
            vec![prior; n]
        }
    }
}

/// Per-row normal log-density `logpdf(zᵢ; μᵢ, σ²)`, composing
/// `gaussian_log_density(0, σ²)` on the residual `zᵢ − μᵢ`.
fn logpdf_rows<T: RealField + FromPrimitive>(z: &[T], mu: &[T], sigma2: T) -> Vec<T> {
    let residuals: Vec<T> = z.iter().zip(mu.iter()).map(|(&zi, &mi)| zi - mi).collect();
    CausalTensor::from_slice(&residuals, &[residuals.len()])
        .gaussian_log_density(T::zero(), sigma2)
        .expect("residual tensor has a valid 1-D shape")
        .as_slice()
        .to_vec()
}

/// Single-row normal log-density `logpdf(z; μ, σ²)`.
fn single_logpdf<T: RealField + FromPrimitive>(z: T, mu: T, sigma2: T) -> T {
    logpdf_rows(&[z], &[mu], sigma2)[0]
}

/// Adds the per-row transform Jacobian to the log-densities.
fn add_jacobian<T: RealField>(mut logdens: Vec<T>, log_jac: &[T]) -> Vec<T> {
    for (ld, &j) in logdens.iter_mut().zip(log_jac.iter()) {
        *ld += j;
    }
    logdens
}

/// Applies the node's effective transform to every continuous parent feature
/// (no Jacobian), when `transform_parents` is set and the transform is active.
/// Mirrors brcd.py L409–421.
fn apply_parent_transform<T: RealField + FromPrimitive>(
    parents: &[Vec<T>],
    eff: Transform,
    transform_parents: bool,
) -> Result<Vec<Vec<T>>, BrcdError> {
    if !transform_parents || eff == Transform::None {
        return Ok(parents.to_vec());
    }
    parents
        .iter()
        .map(|row| {
            row.iter()
                .map(|&v| transform_and_jacobian(v, eff).map(|(z, _)| z))
                .collect::<Result<Vec<T>, _>>()
        })
        .collect()
}

/// Two-term `log(eᵃ + eᵇ)`, stable against overflow (brcd.py `_logsumexp2`).
fn logaddexp<T: RealField>(a: T, b: T) -> T {
    let m = if a >= b { a } else { b };
    if !m.is_finite() {
        return m;
    }
    m + ((a - m).exp() + (b - m).exp()).ln()
}

/// Clamps a probability into `(ε, 1 − ε)` so its log is finite.
fn clamp_unit<T: RealField + FromPrimitive>(p: T) -> T {
    let eps = from_f64::<T>(1e-12);
    p.clamp(eps, T::one() - eps)
}

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
