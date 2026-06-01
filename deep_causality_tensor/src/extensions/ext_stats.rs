/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;
use crate::CausalTensorError;
use deep_causality_num::{FromPrimitive, RealField};

/// Descriptive-statistics extension for a two-dimensional [`CausalTensor`].
///
/// The tensor is interpreted as a data matrix whose **rows are observations**
/// and whose **columns are variables**. Both methods are generic over any real
/// field `T` and use Bessel's correction (`ddof = 1`) for the covariance.
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
}

impl<T> CausalTensorStatsExt<T> for CausalTensor<T>
where
    T: RealField + FromPrimitive,
{
    fn sample_mean(&self) -> Result<CausalTensor<T>, CausalTensorError> {
        let (n, k) = observations_shape(self)?;
        let data = self.as_slice();

        let n_t =
            <T as FromPrimitive>::from_usize(n).ok_or(CausalTensorError::InvalidParameter(
                "observation count is not representable in the tensor's field".to_string(),
            ))?;

        let mut means = vec![T::zero(); k];
        for row in 0..n {
            let base = row * k;
            for (col, mean) in means.iter_mut().enumerate() {
                *mean += data[base + col];
            }
        }
        for mean in means.iter_mut() {
            *mean /= n_t;
        }

        Ok(CausalTensor::from_slice(&means, &[k]))
    }

    fn sample_covariance(&self) -> Result<CausalTensor<T>, CausalTensorError> {
        let (n, k) = observations_shape(self)?;
        if n < 2 {
            // Bessel's correction divides by `n - 1`; with a single observation
            // (or none) that divisor is zero or negative and the statistic is
            // undefined.
            return Err(CausalTensorError::InvalidParameter(format!(
                "sample covariance needs at least 2 observations, got {n}"
            )));
        }

        let data = self.as_slice();
        let means = self.sample_mean()?;
        let means = means.as_slice();

        let denom =
            <T as FromPrimitive>::from_usize(n - 1).ok_or(CausalTensorError::InvalidParameter(
                "observation count is not representable in the tensor's field".to_string(),
            ))?;

        let mut cov = vec![T::zero(); k * k];
        for row in 0..n {
            let base = row * k;
            for i in 0..k {
                let di = data[base + i] - means[i];
                for j in 0..k {
                    let dj = data[base + j] - means[j];
                    cov[i * k + j] += di * dj;
                }
            }
        }
        for entry in cov.iter_mut() {
            *entry /= denom;
        }

        Ok(CausalTensor::from_slice(&cov, &[k, k]))
    }
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
