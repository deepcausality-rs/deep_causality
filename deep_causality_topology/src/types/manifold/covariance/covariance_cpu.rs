/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of covariance analysis for Manifold fields.

use crate::{Manifold, TopologyError};

impl<C, D> Manifold<C, D>
where
    D: Into<f64> + Copy,
{
    /// CPU implementation of covariance matrix computation.
    ///
    /// Computes the covariance matrix of the field data across simplices.
    pub(crate) fn covariance_matrix_cpu(&self) -> Result<Vec<Vec<f64>>, TopologyError> {
        let data = self.data.as_slice();
        let n = data.len();

        if n == 0 {
            return Err(TopologyError::InvalidInput(
                "Cannot compute covariance of empty data".to_string(),
            ));
        }

        // Convert to f64
        let values: Vec<f64> = data.iter().map(|&x: &D| x.into()).collect();

        // Compute mean
        let mean: f64 = values.iter().sum::<f64>() / n as f64;

        // Compute centered data
        let centered: Vec<f64> = values.iter().map(|&x| x - mean).collect();

        // For a 1D field, return 1x1 covariance (variance)
        let variance: f64 = centered.iter().map(|&x| x * x).sum::<f64>() / (n - 1) as f64;

        Ok(vec![vec![variance]])
    }

    /// CPU implementation of eigenvalue decomposition for covariance analysis.
    ///
    /// Returns eigenvalues sorted in descending order.
    pub(crate) fn eigen_covariance_cpu(&self) -> Result<Vec<f64>, TopologyError> {
        let cov = self.covariance_matrix_cpu()?;

        // For 1x1 matrix, eigenvalue is just the variance
        if cov.len() == 1 && cov[0].len() == 1 {
            return Ok(vec![cov[0][0]]);
        }

        // For larger matrices, use power iteration or QR
        // This is a simplified implementation
        Err(TopologyError::InvalidInput(
            "Multi-dimensional covariance eigenvalues not yet implemented".to_string(),
        ))
    }
}
