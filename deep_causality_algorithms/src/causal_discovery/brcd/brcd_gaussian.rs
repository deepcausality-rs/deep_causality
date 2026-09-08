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
//!   `brcd_linalg::solve_linear`.
//! * [`transform_and_jacobian`] and [`effective_transform`] — the
//!   none/log/log1p transform ladder with its `log → log1p → yeojohnson`
//!   auto-downgrade (Yeo-Johnson is deferred; see design D7).
//! * [`gaussian_single_expert_logdensity`] — the per-row log-density of the
//!   single-expert family, evaluating the normal log-density (the exact
//!   `_normal_logpdf_1d`, matching `deep_causality_tensor`'s `gaussian_log_density`)
//!   on the per-row residual, plus the transform Jacobian.
//!
//! The F-node integration (per-regime fits and the mixture-of-experts gate)
//! builds on this in the next stage.

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::causal_discovery::brcd::brcd_gate::{GateConfig, fit_logistic_gate};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_stats::{
    RidgeConfig, fit_ridge as stats_fit_ridge, fit_ridge_streaming as stats_fit_ridge_streaming,
    gaussian_log_density,
};
use std::borrow::Cow;

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
    ///
    /// # A design row of the wrong width is truncated, not refused
    ///
    /// Stated because it was previously only implied by a `zip`. A row shorter than `beta` drops
    /// the trailing coefficients and a longer one drops the trailing features; either way the
    /// answer is a prediction from a *different* model than the one that was fitted, and it comes
    /// back as an ordinary `T` with nothing to distinguish it. Callers inside this module always
    /// build the row from the same parent set the fit was built from, so the widths agree; a
    /// caller that does not should check its own widths.
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

    // The fit itself is `deep_causality_stats::fit_ridge`. The penalty reaches every column, the
    // intercept included, which is `_fit_ridge`'s behaviour and the crate's default — so unlike the
    // logistic gate this needs no `Penalisation`. What does not carry over is the variance floor:
    // the crate reports `rss / dof` as it stands, and BRCD's callers read `σ²` as the scale of a
    // normal density, where a zero would send the log-density to infinity on a perfect fit.
    let fit = stats_fit_ridge(x, y, &RidgeConfig::new(ridge)).map_err(ridge_error)?;
    Ok(RidgeFit {
        beta: fit.beta,
        sigma2: floor(fit.sigma2, from_f64::<T>(VARIANCE_FLOOR)),
    })
}

/// Maps a ridge refusal onto BRCD's error, preserving the distinctions its callers already make.
fn ridge_error(error: deep_causality_stats::StatsError) -> BrcdError {
    match error.kind() {
        deep_causality_stats::StatsErrorEnum::EmptyInput(_) => BrcdError(BrcdErrorEnum::EmptyData),
        deep_causality_stats::StatsErrorEnum::DimensionMismatch(_) => {
            BrcdError(BrcdErrorEnum::DimensionMismatch)
        }
        // A vanishing pivot, a non-finite coefficient, or a non-finite observation the caller did
        // not filter: all of them are the degenerate design `SingularSystem` already names.
        _ => BrcdError(BrcdErrorEnum::SingularSystem),
    }
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
/// `logpdf(z; μ_i, σ²) + log|dz/dx|_i`, the normal log-density of the residual
/// (matching `deep_causality_tensor::gaussian_log_density`).
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
            ExpertMean::Linear(beta) => predict_implicit(beta, features),
        }
    }
}

/// Predicts `β · [1, features]` without materializing the design row. The
/// intercept term `β₀ · 1` is folded first (so `acc` starts at `β₀`, exactly the
/// first step of `dot(β, [1, …])`), making this bit-identical to
/// `dot(β, design_row(features))` while avoiding a per-prediction allocation.
fn predict_implicit<T: RealField>(beta: &[T], features: &[T]) -> T {
    let mut acc = beta[0];
    for (b, f) in beta[1..].iter().zip(features.iter()) {
        acc += *b * *f;
    }
    acc
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
    // `min_finite = 0`: fall back only when no rows are finite (`x_fit.is_empty()`).
    match fit_ridge_streaming(idxs, z_all, parents_t, ridge, 0) {
        Some(fit) => (ExpertMean::Linear(fit.beta), fit.sigma2),
        None => {
            let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
            (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys))
        }
    }
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
    // Guard `x_fit.len() <= p + 1` (p = feature count) → fall back.
    match fit_ridge_streaming(idxs, z_all, parents_t, ridge, p + 1) {
        Some(fit) => (ExpertMean::Linear(fit.beta), fit.sigma2),
        None => {
            let ys: Vec<T> = idxs.iter().map(|&i| z_all[i]).collect();
            (ExpertMean::Const(mean(&ys)), variance_ddof1(&ys))
        }
    }
}

/// Ridge fit over the finite rows of `idxs`: accumulates `XᵀX + λI` and `Xᵀz` from each finite
/// row's `[1, parents]` design, solves, and returns the floored residual variance. Returns `None`
/// when at most `min_finite` rows are finite (the caller's fallback).
///
/// The design matrix is never held whole — the crate reads the row source twice, once to
/// accumulate the normal equations and once to form the residuals, and keeps only the `p × p`
/// buffers in between. **Each row is still allocated**, and twice, because the shipped signature
/// takes `IntoIterator<Item = (Vec<T>, T)> + Clone` and so owns every row it is handed. The
/// hand-rolled fit this replaced copied each row into a single buffer it reused across the whole
/// fit and allocated nothing per row. Removing the per-row allocation needs a borrowed-row entry
/// point in `deep_causality_stats`; it cannot be done from this side.
fn fit_ridge_streaming<T: RealField + FromPrimitive>(
    idxs: &[usize],
    z_all: &[T],
    parents_t: &[Vec<T>],
    ridge: T,
    min_finite: usize,
) -> Option<RidgeFit<T>> {
    let p = parents_t.first().map_or(0, Vec::len) + 1;

    // The rows are a *re-iterable* view over storage this function does not own: an index list into
    // `z_all` and `parents_t`, filtered to the finite rows, with the intercept column prepended as
    // each row is yielded. The design is never materialised as a whole, which is why the shipped
    // signature takes `IntoIterator + Clone` rather than a once-consumable iterator — but the
    // item type is owned, so this closure heap-allocates one `Vec` per finite row and the crate's
    // two passes run it twice per fit. See the note on this function.
    let finite = |i: usize| z_all[i].is_finite() && parents_t[i].iter().all(|v| v.is_finite());
    let count = idxs.iter().filter(|&&i| finite(i)).count();
    if count <= min_finite {
        return None;
    }
    let rows = idxs.iter().filter(move |&&i| finite(i)).map(move |&i| {
        let mut design = Vec::with_capacity(p);
        design.push(T::one());
        design.extend_from_slice(&parents_t[i]);
        (design, z_all[i])
    });

    let fit = stats_fit_ridge_streaming(rows, &RidgeConfig::new(ridge), p).ok()?;
    Some(RidgeFit {
        beta: fit.beta,
        sigma2: floor(fit.sigma2, from_f64::<T>(VARIANCE_FLOOR)),
    })
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

/// Per-row normal log-density `logpdf(zᵢ; μᵢ, σ²)` on the residual `zᵢ − μᵢ`.
///
/// Delegates to `deep_causality_stats::gaussian_log_density`, keeping BRCD's `1e-12` variance
/// floor in front of it so a non-positive or non-finite `σ²` is floored here rather than refused
/// there. Two consequences, both deliberate. The shipped form distributes the `−½` where this
/// factored it out, so results can differ in the last place. And it takes one row at a time, so
/// `log(2πσ²)` is evaluated per row rather than hoisted out of the loop — if that shows up in a
/// BRCD profile, the answer is a slice-shaped density in the statistics crate, not a second copy
/// of the scalar one here.
fn logpdf_rows<T: RealField + FromPrimitive>(z: &[T], mu: &[T], sigma2: T) -> Vec<T> {
    let var = density_variance(sigma2);
    z.iter()
        .zip(mu.iter())
        .map(|(&zi, &mi)| logpdf_one(zi, mi, var))
        .collect()
}

/// Single-row normal log-density `logpdf(z; μ, σ²)`, inline (no allocation).
///
/// The F-as-parent branch calls this once per row. It is already scalar-shaped, so the delegation
/// is exact: same arguments, same variance floor in front of it.
fn single_logpdf<T: RealField + FromPrimitive>(z: T, mu: T, sigma2: T) -> T {
    logpdf_one(z, mu, density_variance(sigma2))
}

/// One row's log-density against an already-floored variance.
///
/// **Why this is not a bare delegation.** BRCD scores a non-finite row rather than refusing the
/// batch that contains it, and the mixture branch depends on that: a NaN parent must leave its own
/// row non-finite while the finite rows still score, which
/// `mixture_with_nonfinite_parent_falls_back_to_the_prior_gate` pins. The shipped density refuses a
/// non-finite argument instead — a defensible contract for a general-purpose function, and the
/// wrong one here.
///
/// So the finite case delegates and the non-finite case is answered from the limit the arithmetic
/// it replaces took, rather than by keeping a second copy of the formula: a NaN residual gives NaN,
/// and an infinite residual sends `(z − μ)² / σ²` to `+∞` and the density to `−∞`.
fn logpdf_one<T: RealField + FromPrimitive>(z: T, mu: T, var: T) -> T {
    match gaussian_log_density(z, mu, var) {
        Ok(value) => value,
        Err(_) => {
            let diff = z - mu;
            if diff.is_nan() {
                T::nan()
            } else {
                // `ln 0` is BRCD's spelling of −∞ throughout; `RealField` carries no
                // `neg_infinity`, which belongs to `Float` and is deliberately out of scope here.
                T::zero().ln()
            }
        }
    }
}

/// The variance used by the normal density: the fit's `σ²` when positive, else
/// the shared `1e-12` floor. Matches `gaussian_log_density`'s `variance_floor`.
fn density_variance<T: RealField + FromPrimitive>(sigma2: T) -> T {
    if sigma2 > T::zero() {
        sigma2
    } else {
        from_f64::<T>(VARIANCE_FLOOR)
    }
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
) -> Result<Cow<'_, [Vec<T>]>, BrcdError> {
    // The common case (no transform, e.g. the verification config) borrows the
    // parent rows instead of cloning the whole matrix per family.
    if !transform_parents || eff == Transform::None {
        return Ok(Cow::Borrowed(parents));
    }
    let transformed = parents
        .iter()
        .map(|row| {
            row.iter()
                .map(|&v| transform_and_jacobian(v, eff).map(|(z, _)| z))
                .collect::<Result<Vec<T>, _>>()
        })
        .collect::<Result<Vec<Vec<T>>, _>>()?;
    Ok(Cow::Owned(transformed))
}

/// Two-term `log(eᵃ + eᵇ)`, delegated to `deep_causality_stats` (brcd.py `_logsumexp2`).
///
/// Bit-identical to the form it replaces. This computed `m + (e^(a−m) + e^(b−m)).ln()` where `m` is
/// the larger, so one of the two exponentials is `e⁰`; the shipped form writes that term as the
/// exact `1` it always is and evaluates one exponential instead of two.
fn logaddexp<T: RealField>(a: T, b: T) -> T {
    deep_causality_stats::log_add_exp(a, b)
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
///
/// Dispatches to `deep_causality_linear::dot` (`unified-math-next` task 6.7), with the truncation
/// moved in front of the call rather than dropped.
///
/// The crate **refuses** a length mismatch, which is the right contract for an inner product and
/// not the one this call site can honour: [`RidgeFit::predict`] returns `T` and has nowhere to put
/// a refusal, and widening it to `Result` breaks a public API this stage is not otherwise breaking.
/// So the truncation stays — but it is now written down here, in two visible slice bounds, instead
/// of hiding inside a `zip` where nobody reading `predict` would find it. `predict`'s own docstring
/// states the consequence.
///
/// The `unwrap_or_else` arm cannot be reached: both operands are cut to the same `n` on the line
/// above, so the crate's length check passes by construction.
fn dot<T: RealField>(a: &[T], b: &[T]) -> T {
    let n = a.len().min(b.len());
    deep_causality_linear::dot(&a[..n], &b[..n]).unwrap_or_else(|_| T::zero())
}

/// Sample mean; `0` for the empty slice.
///
/// Delegates to `deep_causality_stats::mean`, keeping BRCD's empty-slice sentinel in front of it.
/// The crate refuses an empty sample; this returns zero, as it did before, because the callers use
/// the mean as a constant expert prediction and a family with no rows in a regime must still score.
/// Neither implementation guards non-finite values, so a NaN observation still yields a NaN mean.
fn mean<T: RealField + FromPrimitive>(v: &[T]) -> T {
    // The zero is reachable but cannot reach an answer, which a defect audit of this delegation
    // established (`unified-math-next` task 5.19): the only caller that can hand this an empty
    // slice is `fit_expert_guarded` on an empty regime with no parents, and an empty regime scores
    // no rows — so the model built from the sentinel is discarded before anything is written. The
    // sentinel is kept because `fit_expert_guarded` has no empty guard of its own and a panic here
    // would be worse than a value nothing reads.
    deep_causality_stats::mean(v).unwrap_or_else(|_| T::zero())
}

/// Sample variance with Bessel's correction; `1` when fewer than two values
/// (matching `brcd.py`'s fallback).
///
/// Delegates to `deep_causality_stats::variance`, which is the same two-pass form, and keeps the
/// sentinel in front of it. The crate refuses a sample shorter than two with a typed error, which
/// is the better contract for a general-purpose statistic and the wrong one here: BRCD's callers
/// read this variance as a scale for a normal density, and `brcd.py` supplies `1` there so a
/// single-row regime scores against a unit variance rather than failing the family. Seven call
/// sites reach it that way, all in this file, and
/// `f_in_parents_single_row_regime_uses_unit_variance` pins the result.
///
/// The sentinel is applied to every refusal, not only to the short sample. The others —
/// `EmptyInput`, and a count the scalar cannot hold — are unreachable for the shipped scalars, and
/// an empty slice already meant `1` here before the delegation.
fn variance_ddof1<T: RealField + FromPrimitive>(v: &[T]) -> T {
    deep_causality_stats::variance(v).unwrap_or_else(|_| T::one())
}

/// Returns `x` if it exceeds `floor`, else `floor`.
fn floor<T: RealField>(x: T, floor: T) -> T {
    if x > floor { x } else { floor }
}

/// `T` from an `f64` constant.
fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
