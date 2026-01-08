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
use deep_causality_num::Ring;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T>,
    T: TensorData + Ring + Default + PartialOrd,
{
    // ... geometric_product ...

    /// Computes the inner product (grade-0 projection of geometric product).
    ///
    /// ⟨A·B⟩₀ = Tr(A_mat * B_mat†) / dim
    pub fn inner_product(&self, rhs: &Self) -> Self
    where
        T: Clone + Default + PartialOrd + std::ops::Div<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        // Inline geometric product logic to avoid trait/method confusion
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let product_data = B::batched_matmul(&self.data, &rhs.data);
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
    /// A ∧ B = (AB - BA) / 2 (simplified for bivector extraction)
    pub fn outer_product(&self, rhs: &Self) -> Self
    where
        T: Clone + Ring, // Retained Ring bound for T::one()
    {
        // Inline matmul logic
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let ab_data = B::batched_matmul(&self.data, &rhs.data);
        let ba_data = B::batched_matmul(&rhs.data, &self.data);
        let diff = B::sub(&ab_data, &ba_data);

        // Scale by 0.5
        let half = T::one() / (T::one() + T::one());
        let half_tensor = B::from_shape_fn(&[1], |_| half);
        let result = B::mul(&diff, &half_tensor);

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
        T: Clone
            + deep_causality_num::Ring
            + Default
            + PartialOrd
            + std::ops::Div<Output = T>
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        let wedge = self.outer_product(rhs);
        // Apply Hodge dual (multiply by pseudoscalar and negate)
        wedge.hodge_dual()
    }

    /// Applies the Hodge dual operation.
    ///
    /// A* = A · I⁻¹ where I is the pseudoscalar.
    pub fn hodge_dual(&self) -> Self
    where
        T: Clone
            + deep_causality_num::Ring
            + Default
            + PartialOrd
            + std::ops::Div<Output = T>
            + std::ops::AddAssign
            + std::ops::SubAssign
            + std::ops::Neg<Output = T>,
        B: crate::types::multifield::gamma::GammaProvider<T>,
    {
        // 1. Download to Coefficients
        let mut mvs = self.to_coefficients();

        // 2. Apply Dual (CPU)
        for mv in &mut mvs {
            // CausalMultiVector::dual() handles the intricate sign logic of I^-1
            *mv = mv.dual().expect("Hodge dual failed (degenerate metric?)");
        }

        // 3. Upload
        Self::from_coefficients(&mvs, self.shape, self.dx)
    }
}
