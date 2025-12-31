/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MLX-accelerated implementations for geometric product operations.
//! This module is compiled only when the MLX feature is enabled.

use crate::CausalMultiVector;
use core::ops::{AddAssign, Neg, SubAssign};
use deep_causality_num::Field;

impl<T> CausalMultiVector<T> {
    /// Geometric product with automatic MLX acceleration for high-dimensional algebras.
    pub(in crate::types::multivector) fn geometric_product_impl(&self, rhs: &Self) -> Self
    where
        T: Field
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        if self.metric != rhs.metric {
            panic!(
                "Geometric Product Metric mismatch: {:?} vs {:?}",
                self.metric, rhs.metric
            );
        }

        let dim = self.metric.dimension();

        // Automatic MLX acceleration for high-dimensional algebras (Dixon, etc.)
        if dim >= Self::GPU_DIMENSION_THRESHOLD {
            return self.geometric_product_matrix_bridge(rhs);
        }

        // CPU dispatch based on dimension threshold
        if dim <= Self::SPARSE_THRESHOLD {
            self.geometric_product_dense(rhs, dim)
        } else {
            self.geometric_product_sparse(rhs, dim)
        }
    }

    /// GPU-accelerated geometric product using Matrix Isomorphism Bridge.
    ///
    /// Converts coefficients to matrix representation, performs matmul on GPU,
    /// then converts back to coefficients. Efficient for high-dimensional algebras.
    fn geometric_product_matrix_bridge(&self, rhs: &Self) -> Self
    where
        T: Field
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        use deep_causality_tensor::{LinearAlgebraBackend, MlxBackend};

        // Convert to matrix representation on MLX
        let m_self = self.to_matrix_on_backend::<MlxBackend>();
        let m_rhs = rhs.to_matrix_on_backend::<MlxBackend>();

        // Perform matmul on GPU
        let m_result = MlxBackend::matmul(&m_self, &m_rhs);

        // Convert back to coefficients
        Self::from_matrix_on_backend::<MlxBackend>(m_result, self.metric)
    }
}
