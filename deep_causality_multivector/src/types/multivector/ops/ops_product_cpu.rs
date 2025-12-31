/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU-only implementations for geometric product operations.
//! This module is compiled when the MLX feature is disabled.

use crate::CausalMultiVector;
use core::ops::{AddAssign, Neg, SubAssign};
use deep_causality_num::Field;

impl<T> CausalMultiVector<T> {
    /// Core geometric product implementation (CPU-only).
    pub(in crate::types::multivector) fn geometric_product_impl(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        if self.metric != rhs.metric {
            panic!(
                "Geometric Product Metric mismatch: {:?} vs {:?}",
                self.metric, rhs.metric
            );
        }

        let dim = self.metric.dimension();

        // CPU dispatch based on dimension threshold
        if dim <= Self::SPARSE_THRESHOLD {
            self.geometric_product_dense(rhs, dim)
        } else {
            self.geometric_product_sparse(rhs, dim)
        }
    }
}
