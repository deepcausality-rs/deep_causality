/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Public covariance analysis API for Manifold.
//!
//! Dispatches to CPU or MLX implementations based on feature flags and data size.

use crate::{Manifold, TopologyError};

/// Threshold for using GPU acceleration (number of elements).
#[allow(dead_code)]
const GPU_THRESHOLD: usize = 1000;

impl<T> Manifold<T>
where
    T: Into<f64> + Copy,
{
    /// Computes the covariance matrix of the field data.
    ///
    /// GPU-accelerated when `mlx` feature is enabled and n ≥ 1000.
    ///
    /// # Returns
    /// * `Ok(Vec<Vec<f64>>)` - The covariance matrix
    /// * `Err(TopologyError)` - If data is empty or computation fails
    ///
    /// # Example
    /// ```rust,ignore
    /// let cov = manifold.covariance_matrix()?;
    /// ```
    pub fn covariance_matrix(&self) -> Result<Vec<Vec<f64>>, TopologyError> {
        if self.data.len() >= GPU_THRESHOLD {
            return self.covariance_matrix_mlx();
        }

        self.covariance_matrix_cpu()
    }

    /// Computes eigenvalues of the covariance matrix for field analysis.
    ///
    /// GPU-accelerated when `mlx` feature is enabled and n ≥ 1000.
    ///
    /// # Returns
    /// * `Ok(Vec<f64>)` - Eigenvalues sorted in descending order
    /// * `Err(TopologyError)` - If computation fails
    pub fn eigen_covariance(&self) -> Result<Vec<f64>, TopologyError> {
        if self.data.len() >= GPU_THRESHOLD {
            return self.eigen_covariance_mlx();
        }

        self.eigen_covariance_cpu()
    }
}
