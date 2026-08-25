/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algebraic operations for CausalMultiField.
//!
//! Implements vector space operations (scaling) and advanced algebraic operations
//! (reversion, inverse, normalization, commutators).
//!
//! # Algebraic Hierarchy
//!
//! 1. **Vector Space**: `scale(scalar)` - Scalar multiplication
//! 2. **Normed Space**: `normalize()`, `squared_magnitude()` - Length operations
//! 3. **Algebra**: `inverse()`, `reversion()` - Algebraic inverses
//! 4. **Lie/Geometric Algebra**: `commutator_lie()`, `commutator_geometric()` - Bracket operations
//!
//! All operations preserve the Matrix Isomorphism: they work directly on the
//! matrix representation, avoiding costly coefficient extraction.

use alloc::vec::Vec;
// Import local modules
use crate::CausalMultiField;
use crate::MultiVector;
use crate::types::multifield::ops::batched_matmul::BatchedMatMul;
use deep_causality_algebra::{DivisibleByIntegers, Field, NormedScalar, RealField, Ring};
use deep_causality_linear::vector_norm_sq;
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::CausalTensor;

// ============================================================================
// TIER 2: Vector Space (Scaling)
// ============================================================================

impl<T> CausalMultiField<T>
where
    T: Field + Copy + Default + PartialOrd,
{
    /// Scales the field by a scalar: `result = scalar * self`.
    ///
    /// # Arguments
    /// * `scalar` - The scalar value to multiply by
    pub fn scale(&self, scalar: T) -> Self {
        let scalar_tensor = CausalTensor::<T>::from_shape_fn(&[1], |_| scalar);
        let result = &self.data * &scalar_tensor;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// ============================================================================
// TIER 3: Normed Space (Normalize, Magnitude)
// ============================================================================

impl<T> CausalMultiField<T>
where
    T: Field + RealField + Copy + Default + PartialOrd + FromPrimitive,
{
    /// Normalizes the field: `result = self / ||self||`.
    ///
    /// Returns the field scaled to unit magnitude.
    pub fn normalize(&self) -> Self {
        let mag_sq = self.squared_magnitude();
        if mag_sq.is_zero() {
            return self.clone();
        }
        let mag = mag_sq.sqrt();
        let inv_mag = T::one() / mag;
        self.scale(inv_mag)
    }

    /// Computes the squared magnitude of the field: the squared 2-norm of its coefficients.
    ///
    /// # Routed through `deep_causality_linear`
    ///
    /// The body was `Σ *val * *val` over the tensor's buffer. That is the squared modulus **only
    /// because the impl is bounded on `RealField`** — for a complex scalar `Σ z·z` is not `Σ |z|²`,
    /// and nothing in the expression says so. It was not wrong; it was correct for a reason that
    /// lived in the bound rather than in the code, and a later widening would have made it
    /// silently wrong.
    ///
    /// [`vector_norm_sq`](deep_causality_linear::vector_norm_sq) uses `modulus_squared`, which is
    /// the operation that stays right when the scalar does not. The return type is unchanged:
    /// `<T as Normed>::Real` is `T` for a real field, which is what this impl is bounded on.
    pub fn squared_magnitude(&self) -> T {
        vector_norm_sq(self.data.as_slice())
    }
}

// ============================================================================
// TIER 4: Full Algebra (Inverse, Reversion)
// ============================================================================

impl<T> CausalMultiField<T>
where
    T: DivisibleByIntegers + Copy + Default + PartialOrd + core::ops::Neg<Output = T> + 'static,
{
    /// Computes the reversion (reversal) of the field.
    ///
    /// The reversion is computed by extracting multivectors, applying
    /// the reversion operation, and reconstructing.
    pub fn reversion(&self) -> Self {
        let mvs = self.to_coefficients();
        let reversed: Vec<_> = mvs.iter().map(|mv| mv.reversion()).collect();
        Self::from_coefficients(&reversed, self.shape, self.dx)
    }

    /// Computes the multiplicative inverse of the field.
    ///
    /// Uses matrix inverse for each cell.
    pub fn inverse(&self) -> Self
    where
        T: RealField + NormedScalar,
    {
        let mvs = self.to_coefficients();
        let inverted: Vec<_> = mvs
            .iter()
            .map(|mv| mv.inverse().expect("Failed to invert multivector"))
            .collect();
        Self::from_coefficients(&inverted, self.shape, self.dx)
    }
}

// ============================================================================
// TIER 5: Lie Algebra / Geometric Algebra (Commutators)
// ============================================================================

impl<T> CausalMultiField<T>
where
    T: Field + Ring + Copy + Default + PartialOrd,
    CausalTensor<T>: BatchedMatMul<T>,
{
    /// Computes the Lie commutator: `[A, B] = AB - BA`.
    ///
    /// The Lie bracket measures the non-commutativity of the geometric product.
    pub fn commutator_lie(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        // AB - BA using batched matmul
        let ab = self.data.batched_matmul(&rhs.data);
        let ba = rhs.data.batched_matmul(&self.data);
        let result = &ab - &ba;

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

/// The geometric commutator halves, so it needs a scalar in which `1 + 1` is invertible.
///
/// [`DivisibleByIntegers`] rather than `Field`: `Field` is blanket-implemented and so admits every
/// finite field automatically, and over 𝔽₂ the `half` below is a division by zero. The Lie
/// commutator above does not halve and keeps the looser bound.
impl<T> CausalMultiField<T>
where
    T: DivisibleByIntegers + Ring + Copy + Default + PartialOrd,
    CausalTensor<T>: BatchedMatMul<T>,
{
    /// Computes the geometric commutator: `(AB - BA) / 2`.
    ///
    /// Equivalent to the Lie commutator scaled by 1/2.
    pub fn commutator_geometric(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let ab = self.data.batched_matmul(&rhs.data);
        let ba = rhs.data.batched_matmul(&self.data);
        let diff = &ab - &ba;

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
}
