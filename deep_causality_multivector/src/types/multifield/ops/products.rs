/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Clifford algebra products for CausalMultiField.
//!
//! All products are implemented via the Matrix Isomorphism:
//! - Geometric product → batched matmul
//! - Inner product → grade-0 projection
//! - Outer product → antisymmetrization
//! - Commutator → AB - BA

use crate::CausalMultiField;
use crate::traits::multi_vector::MultiVector as MultiVectorTrait;
use crate::types::multifield::ops::batched_matmul::BatchedMatMul;
use deep_causality_num::{Field, Ring};
use deep_causality_tensor::CausalTensor;

impl<T> CausalMultiField<T>
where
    T: Field + Ring + Copy + Default + PartialOrd,
    CausalTensor<T>: BatchedMatMul<T>,
{
    /// Computes the inner product (grade-0 projection of geometric product).
    pub fn inner_product(&self, rhs: &Self) -> Self
    where
        T: std::ops::Neg<Output = T>,
    {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let product_data = self.data.batched_matmul(&rhs.data);
        let product = Self {
            data: product_data,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        };
        product.grade_project(0)
    }

    /// Computes the outer product (antisymmetric part).
    ///
    /// A ∧ B = (AB - BA) / 2
    pub fn outer_product(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let ab_data = self.data.batched_matmul(&rhs.data);
        let ba_data = rhs.data.batched_matmul(&self.data);
        let diff = &ab_data - &ba_data;

        // Scale by 0.5
        let half = T::one() / (T::one() + T::one());
        let half_tensor = CausalTensor::<T>::from_shape_fn(&[1], |_| half);
        let result = &diff * &half_tensor;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }

    /// Computes the cross product via Hodge dual of wedge.
    ///
    /// A × B = -I(A ∧ B) where I is the pseudoscalar.
    pub fn cross(&self, rhs: &Self) -> Self
    where
        T: std::ops::AddAssign + std::ops::SubAssign + std::ops::Neg<Output = T>,
    {
        let wedge = self.outer_product(rhs);
        wedge.hodge_dual()
    }

    /// Applies the Hodge dual operation.
    ///
    /// A* = A · I⁻¹ where I is the pseudoscalar.
    pub fn hodge_dual(&self) -> Self
    where
        T: std::ops::AddAssign + std::ops::SubAssign + std::ops::Neg<Output = T>,
    {
        // Download to Coefficients
        let mut mvs = self.to_coefficients();

        // Apply Dual (CPU)
        for mv in &mut mvs {
            *mv = mv.dual().expect("Hodge dual failed (degenerate metric?)");
        }

        // Upload
        Self::from_coefficients(&mvs, self.shape, self.dx)
    }
}
