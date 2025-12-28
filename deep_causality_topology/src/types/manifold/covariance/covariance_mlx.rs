/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX GPU-accelerated covariance analysis for Manifold fields.

#![cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]

use crate::{Manifold, TopologyError};
use deep_causality_tensor::MlxCausalTensor;

impl<T> Manifold<T>
where
    T: Into<f64> + Copy,
{
    /// MLX GPU-accelerated covariance matrix computation.
    ///
    /// Uses GPU for large datasets (n >= 1000).
    pub(crate) fn covariance_matrix_mlx(&self) -> Result<Vec<Vec<f64>>, TopologyError> {
        let data = self.data.as_slice();
        let n = data.len();

        if n == 0 {
            return Err(TopologyError::InvalidInput(
                "Cannot compute covariance of empty data".to_string(),
            ));
        }

        // Convert to f64 then downcast to f32 for GPU
        let values: Vec<f64> = data.iter().map(|&x| x.into()).collect();

        // Create MLX tensor
        let mlx_data = MlxCausalTensor::new_from_f64(&values, vec![n])
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;

        // Compute mean on GPU using sum (returns f32 directly)
        let sum_val: f32 = mlx_data
            .sum()
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        let mean = sum_val as f64 / n as f64;

        // Center the data (element-wise subtraction)
        let mean_vec = vec![mean as f32; n];
        let mean_tensor = MlxCausalTensor::new_f32(mean_vec, vec![n])
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;

        let centered = mlx_data
            .sub(&mean_tensor)
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;

        // Compute variance: sum(centered^2) / (n-1)
        let centered_sq = centered
            .mul(&centered)
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;

        let sum_sq_val: f32 = centered_sq
            .sum()
            .map_err(|e| TopologyError::TensorError(e.to_string()))?;
        let variance = sum_sq_val as f64 / (n - 1) as f64;

        Ok(vec![vec![variance]])
    }

    /// MLX GPU-accelerated eigenvalue decomposition for covariance analysis.
    ///
    /// Uses GPU for eigh decomposition when available.
    pub(crate) fn eigen_covariance_mlx(&self) -> Result<Vec<f64>, TopologyError> {
        let cov = self.covariance_matrix_mlx()?;

        // For 1x1, eigenvalue is variance
        if cov.len() == 1 && cov[0].len() == 1 {
            return Ok(vec![cov[0][0]]);
        }

        // For larger matrices, would use MLX eigh
        // Fall back to CPU for now
        self.eigen_covariance_cpu()
    }
}
