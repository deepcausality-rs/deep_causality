/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algebraic structure implementations for CausalMultiField.
//!
//! This module provides algebraic trait implementations that mirror those
//! of `CausalMultiVector`, but operate on entire spatial grids of multivectors
//! using batched tensor operations.
//!
//! # Algebraic Hierarchy
//!
//! CausalMultiField implements a tiered set of algebraic structures:
//!
//! ## Tier 1: Additive Structure
//! - `Add`, `Sub`, `Neg` - Element-wise field operations
//! - Field addition is commutative
//!
//! ## Tier 2: Vector Space
//! - `scale()` - Multiply all cells by a scalar
//!
//! ## Tier 3: Ring Structure
//! - `Mul` - Geometric product via batched matrix multiplication
//! - Product is associative: `(A * B) * C = A * (B * C)`
//!
//! ## Tier 4: Field Operations
//! - `normalize()`, `inverse()` - Require RealField coefficients
//!
//! # GPU Acceleration
//!
//! All operations dispatch to the backend's tensor operations, enabling
//! automatic MLX/CUDA acceleration when the appropriate feature flags are set.
use crate::BatchedMatMul;
use crate::CausalMultiField;
use crate::MultiVector as MultiVectorTrait;
use deep_causality_num::{RealField, Ring};
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};

// ============================================================================
// TIER 2: Vector Space (Scaling)
// ============================================================================

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend,
    T: TensorData,
{
    /// Multiplies the entire field by a scalar value.
    ///
    /// This is a batched operation that scales all grid cells simultaneously.
    ///
    /// # Arguments
    /// * `scalar` - The scalar multiplier
    ///
    /// # Returns
    /// A new field with all cells scaled by the given value.
    pub fn scale(&self, scalar: T) -> Self
    where
        T: Clone,
    {
        let scalar_tensor = B::from_shape_fn(&[1], |_| scalar);
        let result = B::mul(&self.data, &scalar_tensor);
        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }
}

// ============================================================================
// TIER 4: Field Operations (normalize, inverse)
// ============================================================================

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T> + crate::types::multifield::gamma::GammaProvider<T>,
    T: TensorData
        + RealField
        + Default
        + PartialOrd
        + Clone
        + std::ops::AddAssign
        + std::ops::SubAssign
        + std::ops::Neg<Output = T>
        + std::ops::Div<Output = T>,
{
    /// Normalizes each cell in the field to unit magnitude.
    ///
    /// For each cell, computes: `cell' = cell / |cell|`
    ///
    /// Cells with magnitude below epsilon are left unchanged.
    pub fn normalize(&self) -> Self {
        // Download to coefficients
        let mut mvs = self.to_coefficients();

        // Normalize each multivector
        for mv in &mut mvs {
            *mv = mv.normalize();
        }

        // Upload back
        Self::from_coefficients(&mvs, self.shape, self.dx)
    }

    /// Computes the multiplicative inverse of each cell in the field.
    ///
    /// For versors (products of vectors), the inverse is:
    /// `A^{-1} = ~A / |A|^2`
    ///
    /// where `~A` is the reversion.
    ///
    /// # Returns
    /// - `Ok(field)` if all cells are invertible
    /// - `Err` if any cell has zero magnitude
    pub fn inverse(&self) -> Result<Self, crate::CausalMultiVectorError> {
        // Download to coefficients
        let mvs = self.to_coefficients();

        // Invert each multivector
        let mut inverted = Vec::with_capacity(mvs.len());
        for mv in mvs {
            inverted.push(mv.inverse()?);
        }

        // Upload back
        Ok(Self::from_coefficients(&inverted, self.shape, self.dx))
    }

    /// Computes the reversion of each cell in the field.
    ///
    /// The reversion operation reverses the order of basis vectors in each blade:
    /// `~(e_1 e_2 ... e_k) = e_k ... e_2 e_1`
    pub fn reversion(&self) -> Self {
        // Download to coefficients
        let mvs = self.to_coefficients();

        // Revert each multivector
        let reverted: Vec<_> = mvs.iter().map(|mv| mv.reversion()).collect();

        // Upload back
        Self::from_coefficients(&reverted, self.shape, self.dx)
    }

    /// Computes the squared magnitude of each cell in the field.
    ///
    /// Returns a scalar field where each value is `|cell|^2`.
    pub fn squared_magnitude(&self) -> B::Tensor<T> {
        // Download to coefficients
        let mvs = self.to_coefficients();

        // Compute squared magnitude for each
        let mags: Vec<T> = mvs.iter().map(|mv| mv.squared_magnitude()).collect();

        // Return as tensor [Nx, Ny, Nz]
        B::create(&mags, &self.shape)
    }
}

// ============================================================================
// Lie Algebra Operations
// ============================================================================

impl<B, T> CausalMultiField<B, T>
where
    B: LinearAlgebraBackend + BatchedMatMul<T>,
    T: TensorData + Ring + Default + PartialOrd,
{
    /// Computes the Lie bracket commutator `[A, B] = AB - BA`.
    ///
    /// This is a batched operation performed entirely on the GPU.
    pub fn commutator_lie(&self, rhs: &Self) -> Self {
        assert_eq!(self.metric, rhs.metric, "Metric mismatch");
        assert_eq!(self.shape, rhs.shape, "Shape mismatch");

        let ab_data = B::batched_matmul(&self.data, &rhs.data);
        let ba_data = B::batched_matmul(&rhs.data, &self.data);
        let result = B::sub(&ab_data, &ba_data);

        Self {
            data: result,
            metric: self.metric,
            dx: self.dx,
            shape: self.shape,
        }
    }

    /// Computes the geometric commutator `(AB - BA) / 2`.
    pub fn commutator_geometric(&self, rhs: &Self) -> Self
    where
        T: std::ops::Div<Output = T>,
    {
        let lie = self.commutator_lie(rhs);

        // Scale by 0.5
        let half = T::one() / (T::one() + T::one());
        lie.scale(half)
    }
}
