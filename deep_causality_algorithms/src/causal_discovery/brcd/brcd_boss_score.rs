/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The linear-Gaussian BIC-from-covariance score that BOSS maximizes.
//!
//! BOSS is BRCD's optional structure-learning preprocessor; this is the only
//! score it uses (the RKHS and BDeu scores in the reference
//! `LocalScoreFunction.py` are out of scope). It is computed entirely from the
//! sample covariance via
//! [`deep_causality_tensor::CausalTensorStatsExt::conditional_variance`] (the
//! ridge Schur complement `Σ_yy − Σ_yP (Σ_PP + εI)⁻¹ Σ_Py`).
//!
//! # Sign convention (corrects a reference bug)
//!
//! The score is the **higher-is-better** SEM-BIC the order search maximizes:
//!
//! ```text
//! score(i, PA) = −½·n·ln(σ²) − ½·ln(n)·(|PA| + 1)·λ
//! ```
//!
//! where `σ²` is the (ridge) conditional variance of node `i` given `PA`
//! (its marginal variance when `PA` is empty). A parent set that reduces the
//! conditional variance enough to overcome its penalty scores **strictly
//! higher** than the empty set, so grow/shrink and the order search add genuine
//! parents.
//!
//! This is the convention of causal-learn's `local_score_BIC_from_cov` and of the
//! BOSS the BRCD paper runs ("default setting of BOSS from causal-learn",
//! Appendix D). The vendored reference `LocalScoreFunction.py` returns the
//! *negated* `n·ln(σ²) + ln(n)·|PA|·λ` (lower = better) while its BOSS
//! *maximizes* — so the vendored search adds no parents and learns the empty
//! graph on a clean chain. See the `brcd-bootstrap` design note (D2).

use crate::brcd::brcd_boss_config::BossConfig;
use crate::brcd::{BrcdError, BrcdErrorEnum};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::{CausalTensor, CausalTensorError, CausalTensorStatsExt};

/// A family scorer the grow-shrink tree and order search query.
///
/// Abstracting over the concrete [`BicScorer`] keeps the search structures
/// scorer-agnostic (they only need a `(node, parents) → score` oracle and the
/// variable count) and lets tests substitute a call-counting scorer to assert
/// the tree's caching. Static dispatch only — no `dyn`.
pub trait FamilyScorer<T> {
    /// The higher-is-better score of the family `(node, parents)`.
    fn score(&self, node: usize, parents: &[usize]) -> Result<T, BrcdError>;

    /// Number of variables in scope (valid `node`/parent indices are `0..n`).
    fn num_vars(&self) -> usize;
}

/// Smallest variance the score will use; a non-positive or non-finite variance
/// (a constant column, or a Schur complement driven negative by round-off) is
/// floored to this so `ln(σ²)` stays finite. Matches the tensor crate's
/// `variance_floor` and the reference's practice of dropping zero-variance
/// metrics before scoring.
const VARIANCE_FLOOR: f64 = 1e-12;

/// Scores node families against a fixed sample covariance.
///
/// Holds the `k × k` covariance, the sample count `n`, and the score
/// hyperparameters, so the grow-shrink tree and the order search can score any
/// `(node, parents)` family without re-reading the raw data.
pub struct BicScorer<'a, T> {
    cov: &'a CausalTensor<T>,
    n: usize,
    ridge_eps: T,
    bic_lambda: T,
}

impl<'a, T: RealField + FromPrimitive> BicScorer<'a, T> {
    /// Builds a scorer over a `k × k` sample covariance and the sample count `n`
    /// that produced it.
    ///
    /// # Errors
    /// [`BrcdErrorEnum::DimensionMismatch`] if `cov` is not a square 2-D matrix,
    /// or [`BrcdErrorEnum::EmptyData`] if `n` is zero.
    pub fn new(
        cov: &'a CausalTensor<T>,
        n: usize,
        config: &BossConfig<T>,
    ) -> Result<Self, BrcdError> {
        match cov.shape() {
            [rows, cols] if rows == cols && *rows > 0 => {}
            _ => return Err(BrcdError(BrcdErrorEnum::DimensionMismatch)),
        }
        if n == 0 {
            return Err(BrcdError(BrcdErrorEnum::EmptyData));
        }
        Ok(Self {
            cov,
            n,
            ridge_eps: config.ridge_eps,
            bic_lambda: config.bic_lambda,
        })
    }

    /// Number of variables (the covariance dimension).
    pub fn num_vars(&self) -> usize {
        // The constructor guarantees a square 2-D shape.
        self.cov.shape()[0]
    }

    /// The higher-is-better BIC score of the family `(node, parents)`.
    ///
    /// # Errors
    /// [`BrcdErrorEnum::NodeOutOfBounds`] if `node` or any parent index is
    /// outside `0..num_vars`.
    pub fn score(&self, node: usize, parents: &[usize]) -> Result<T, BrcdError> {
        family_bic(
            self.cov,
            self.n,
            node,
            parents,
            self.ridge_eps,
            self.bic_lambda,
        )
    }
}

impl<T: RealField + FromPrimitive> FamilyScorer<T> for BicScorer<'_, T> {
    fn score(&self, node: usize, parents: &[usize]) -> Result<T, BrcdError> {
        BicScorer::score(self, node, parents)
    }

    fn num_vars(&self) -> usize {
        BicScorer::num_vars(self)
    }
}

/// The higher-is-better linear-Gaussian BIC of node `node` given `parents`,
/// computed from the sample covariance `cov` (`k × k`) and sample count `n`.
///
/// Returns `−½·n·ln(σ²) − ½·ln(n)·(|parents| + 1)·λ`, where `σ²` is the ridge
/// conditional variance (the marginal variance when `parents` is empty), floored
/// to `VARIANCE_FLOOR` so a constant column cannot produce `ln(0)`.
///
/// # Errors
/// [`BrcdErrorEnum::NodeOutOfBounds`] if `node` or any parent index is outside
/// `0..k`; [`BrcdErrorEnum::DimensionMismatch`] if `cov` is not square 2-D.
pub fn family_bic<T: RealField + FromPrimitive>(
    cov: &CausalTensor<T>,
    n: usize,
    node: usize,
    parents: &[usize],
    ridge_eps: T,
    bic_lambda: T,
) -> Result<T, BrcdError> {
    let raw = cov
        .conditional_variance(node, parents, ridge_eps)
        .map_err(map_tensor_err)?;

    let floor = from_f64::<T>(VARIANCE_FLOOR);
    let var = if raw.is_finite() && raw > floor {
        raw
    } else {
        floor
    };

    let n_t = from_usize::<T>(n);
    let half = from_f64::<T>(0.5);
    let p1 = from_usize::<T>(parents.len() + 1);

    // −½·n·ln(σ²) − ½·ln(n)·(|PA|+1)·λ
    let fit = -half * n_t * var.ln();
    let penalty = half * n_t.ln() * p1 * bic_lambda;
    Ok(fit - penalty)
}

/// Maps a covariance-extension error onto the BRCD error set.
fn map_tensor_err(err: CausalTensorError) -> BrcdError {
    match err {
        CausalTensorError::IndexOutOfBounds => BrcdError(BrcdErrorEnum::NodeOutOfBounds),
        _ => BrcdError(BrcdErrorEnum::DimensionMismatch),
    }
}

fn from_usize<T: FromPrimitive>(n: usize) -> T {
    <T as FromPrimitive>::from_usize(n).expect("count is representable in every RealField")
}

fn from_f64<T: FromPrimitive>(x: f64) -> T {
    <T as FromPrimitive>::from_f64(x).expect("constant is representable in every RealField")
}
