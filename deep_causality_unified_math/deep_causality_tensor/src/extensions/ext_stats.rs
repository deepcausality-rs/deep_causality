/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensor;
use crate::CausalTensorError;
use alloc::format;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_stats::{
    StatsError, StatsErrorEnum, column_means, conditional_variance, covariance_matrix,
    gaussian_log_density, log_sum_exp,
};

/// Descriptive-statistics extension for a two-dimensional [`CausalTensor`].
///
/// The tensor is interpreted as a data matrix whose **rows are observations**
/// and whose **columns are variables**. Both methods are generic over any real
/// field `T` and use Bessel's correction (`ddof = 1`) for the covariance.
///
/// Every statistic here comes from `deep_causality_stats`; what remains is the axis work — reading
/// a row-major matrix as observations and variables, and handing the result back as a tensor. That
/// is the line the statistics crate is drawn along: it owns the reductions over slices, and the
/// container that has axes owns the naming of them.
///
/// This reverses design decision **D7**, which kept five implementations here because delegating
/// would move `tensor` to tier 5, `multivector` to 6 and `topology` to 7. That renumbering is real
/// and has been done. The judgement changed because five duplicated statistics are a maintenance
/// surface that grows, and consolidating them is worth a row moving in a table. There was never a
/// cycle to prevent it: `deep_causality_stats` reaches only `num`, `algebra`, `haft` and `linear`,
/// none of which reach `tensor`.
pub trait CausalTensorStatsExt<T> {
    /// Computes the per-column (per-variable) sample means.
    ///
    /// # Returns
    /// A 1-D `CausalTensor` of shape `[k]` holding the mean of each of the `k`
    /// columns, or a `CausalTensorError` if the tensor is not 2-D or is empty.
    fn sample_mean(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes the sample covariance matrix (`ddof = 1`).
    ///
    /// For a tensor with `n` observations (rows) and `k` variables (columns),
    /// returns the `k × k` matrix whose `(i, j)` entry is
    /// `Σ_r (x_ri − μ_i)(x_rj − μ_j) / (n − 1)`.
    ///
    /// # Returns
    /// A `CausalTensor` of shape `[k, k]`, or a `CausalTensorError` if the
    /// tensor is not 2-D, has no columns, or has fewer than two observations
    /// (the `ddof = 1` divisor `n − 1` would be zero or negative).
    fn sample_covariance(&self) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes `log(Σ_i exp(x_i))` over all elements using the max-shift
    /// formulation, so the result does not overflow for large inputs.
    ///
    /// # Returns
    /// The log-sum-exp of every element. An empty tensor has an empty sum, so
    /// the result is `log(0) = −∞`.
    fn logsumexp(&self) -> T;

    /// Evaluates the one-dimensional normal log-density of every element given a
    /// shared `mean` and `variance`:
    /// `−½ · (log(2π·var) + (xᵢ − μ)² / var)`.
    ///
    /// A non-positive variance is floored to a small positive constant so the
    /// density stays finite.
    ///
    /// # Returns
    /// A `CausalTensor` of element-wise log-densities with the same shape as
    /// `self`.
    fn gaussian_log_density(
        &self,
        mean: T,
        variance: T,
    ) -> Result<CausalTensor<T>, CausalTensorError>;

    /// Computes the conditional variance of a target variable given a parent
    /// set, treating `self` as a joint covariance matrix.
    ///
    /// Returns the Schur complement `Σ_yy − Σ_yP Σ_PP⁻¹ Σ_Py`, the residual
    /// variance of regressing the target on its parents. The parent block is
    /// ridge-regularized (`Σ_PP + λI`) so the solve stays finite when the
    /// parents are collinear.
    ///
    /// # Arguments
    /// * `target` — row/column index of the target variable.
    /// * `parents` — indices of the conditioning variables (length `k`).
    /// * `ridge` — non-negative ridge `λ` added to the diagonal of `Σ_PP`.
    ///
    /// # Returns
    /// The conditional variance, or a `CausalTensorError` if `self` is not a
    /// square 2-D matrix or any index is out of bounds. An empty parent set
    /// returns the target's marginal variance `Σ_yy`.
    fn conditional_variance(
        &self,
        target: usize,
        parents: &[usize],
        ridge: T,
    ) -> Result<T, CausalTensorError>;
}

impl<T> CausalTensorStatsExt<T> for CausalTensor<T>
where
    T: RealField + FromPrimitive,
{
    fn sample_mean(&self) -> Result<CausalTensor<T>, CausalTensorError> {
        let (n, k) = observations_shape(self)?;
        let means = column_means(self.as_slice(), n, k).map_err(stats_error)?;
        Ok(CausalTensor::from_slice(&means, &[k]))
    }

    fn sample_covariance(&self) -> Result<CausalTensor<T>, CausalTensorError> {
        let (n, k) = observations_shape(self)?;
        let cov = covariance_matrix(self.as_slice(), n, k).map_err(stats_error)?;
        Ok(CausalTensor::from_slice(&cov, &[k, k]))
    }

    fn logsumexp(&self) -> T {
        log_sum_exp(self.as_slice())
    }

    fn gaussian_log_density(
        &self,
        mean: T,
        variance: T,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if self.is_empty() {
            return Ok(CausalTensor::from_slice(&[], self.shape()));
        }
        // The floor stays here rather than moving into the statistics crate. That crate refuses a
        // non-positive variance, which is the right contract for a density asked about one; this
        // method's contract is to return a finite density for every element it is handed.
        let var = if variance > T::zero() {
            variance
        } else {
            variance_floor::<T>()
        };
        let new_data: Vec<T> = self
            .as_slice()
            .iter()
            .map(|&x| gaussian_log_density(x, mean, var).map_err(stats_error))
            .collect::<Result<Vec<T>, CausalTensorError>>()?;
        Ok(CausalTensor::from_slice(&new_data, self.shape()))
    }

    fn conditional_variance(
        &self,
        target: usize,
        parents: &[usize],
        ridge: T,
    ) -> Result<T, CausalTensorError> {
        let shape = self.shape();
        if shape.len() != 2 || shape[0] != shape[1] {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = shape[0];
        // Index bounds are checked here rather than left to the statistics crate. They are the axis
        // work this wrapper owns, and `IndexOutOfBounds` says what went wrong where the crate's
        // coarser `DimensionMismatch` would not. The crate keeps its own guard as a second line.
        if target >= m || !parents.iter().all(|&p| p < m) {
            return Err(CausalTensorError::IndexOutOfBounds);
        }
        conditional_variance(self.as_slice(), m, target, parents, ridge).map_err(stats_error)
    }
}

/// Maps a refusal from the statistics crate onto this crate's error.
///
/// `CausalTensorError` is the coarser of the two — it has no variant for "too few observations" or
/// "rank deficient" — so those distinctions fold onto `InvalidParameter`, where they read as what
/// they are: something wrong with the numbers rather than with the tensor's shape.
fn stats_error(error: StatsError) -> CausalTensorError {
    match error.kind() {
        StatsErrorEnum::EmptyInput(_) => CausalTensorError::EmptyTensor,
        StatsErrorEnum::DimensionMismatch(_) => CausalTensorError::DimensionMismatch,
        other => CausalTensorError::InvalidParameter(format!("{other:?}")),
    }
}

/// Smallest variance the Gaussian log-density will use; non-positive variances
/// are floored to this value to keep the density finite.
fn variance_floor<T>() -> T
where
    T: RealField + FromPrimitive,
{
    <T as FromPrimitive>::from_f64(1e-12).expect("1e-12 is representable in every RealField")
}

/// Validates that `tensor` is a non-empty 2-D matrix and returns `(rows, cols)`.
fn observations_shape<T>(tensor: &CausalTensor<T>) -> Result<(usize, usize), CausalTensorError> {
    let shape = tensor.shape();
    if shape.len() != 2 {
        return Err(CausalTensorError::DimensionMismatch);
    }
    let (n, k) = (shape[0], shape[1]);
    if n == 0 || k == 0 {
        return Err(CausalTensorError::EmptyTensor);
    }
    Ok((n, k))
}
