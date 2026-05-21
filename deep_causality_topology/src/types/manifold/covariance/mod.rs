/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of covariance analysis for Manifold fields.

use crate::{Manifold, SimplicialComplex, TopologyError};
use deep_causality_num::{FromPrimitive, RealField};

impl<C, D> Manifold<SimplicialComplex<C>, D>
where
    C: RealField,
    D: RealField + FromPrimitive,
{
    /// CPU implementation of covariance matrix computation.
    ///
    /// Computes the covariance matrix of the field data across simplices.
    pub(crate) fn covariance_matrix_impl(&self) -> Result<Vec<Vec<D>>, TopologyError> {
        let data = self.data.as_slice();
        let n = data.len();

        if n == 0 {
            return Err(TopologyError::InvalidInput(
                "Cannot compute covariance of empty data".to_string(),
            ));
        }

        let n_d = <D as FromPrimitive>::from_usize(n)
            .ok_or_else(|| TopologyError::InvalidInput("n not representable in D".to_string()))?;
        let one = D::one();
        let n_minus_one = n_d - one;

        // Mean
        let mut sum = D::zero();
        for &x in data.iter() {
            sum += x;
        }
        let mean = sum / n_d;

        // Variance (centered sum of squares / (n - 1))
        let mut acc = D::zero();
        for &x in data.iter() {
            let d = x - mean;
            acc += d * d;
        }
        let variance = acc / n_minus_one;

        Ok(vec![vec![variance]])
    }

    /// CPU implementation of eigenvalue decomposition for covariance analysis.
    ///
    /// Returns eigenvalues sorted in descending order.
    pub(crate) fn eigen_covariance_impl(&self) -> Result<Vec<D>, TopologyError> {
        let cov = self.covariance_matrix_impl()?;

        if cov.len() == 1 && cov[0].len() == 1 {
            return Ok(vec![cov[0][0]]);
        }

        Err(TopologyError::InvalidInput(
            "Multi-dimensional covariance eigenvalues not yet implemented".to_string(),
        ))
    }
}
