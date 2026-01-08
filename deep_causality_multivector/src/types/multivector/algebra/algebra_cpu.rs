/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU-only implementations for Tier 3 algebra operations.
//! This module is compiled when the MLX feature is disabled.

use crate::{CausalMultiVector, CausalMultiVectorError};
use deep_causality_num::{Field, RealField};
use std::ops::{AddAssign, Neg, SubAssign};

// Internal implementation methods
impl<T> CausalMultiVector<T> {
    /// Computes the squared magnitude (squared norm) of the multivector.
    ///
    /// $$ ||A||^2 = \langle A \tilde{A} \rangle_0 $$
    pub(in crate::types::multivector) fn squared_magnitude_impl(&self) -> T
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        let reverse = self.reversion_impl();
        let product = self.geometric_product_impl(&reverse);
        product.data[0] // Scalar part
    }

    /// Computes the inverse of the multivector $A^{-1}$ (CPU-only).
    pub(in crate::types::multivector) fn inverse_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign,
    {
        let sq_mag = self.squared_magnitude_impl();
        if sq_mag == T::zero() {
            return Err(CausalMultiVectorError::zero_magnitude());
        }

        let reverse = self.reversion_impl();
        Ok(reverse / sq_mag)
    }

    /// Computes the dual of the multivector $A^*$ (CPU-only).
    pub(in crate::types::multivector) fn dual_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign,
    {
        let pseudo = Self::pseudoscalar(self.metric);
        let pseudo_inv = pseudo.inverse_impl()?;
        Ok(self.geometric_product_impl(&pseudo_inv))
    }
}

// Public API methods - Tier 3 operations (CPU version)
impl<T> CausalMultiVector<T>
where
    T: RealField + Copy,
{
    /// Normalizes the multivector to unit magnitude.
    pub fn normalize(&self) -> Self {
        let mag_sq = self.squared_magnitude_impl();
        if mag_sq <= T::epsilon() {
            return self.clone();
        }
        let scale_factor = T::one() / mag_sq.sqrt();
        self.scale(scale_factor)
    }
}

impl<T> CausalMultiVector<T>
where
    T: Field + Copy + RealField,
{
    /// Computes the Lie Commutator: $[A, B] = AB - BA$.
    /// Valid for all associative algebras.
    pub fn commutator(&self, rhs: &Self) -> Self {
        self.commutator_lie_impl(rhs)
    }

    /// Computes the Multiplicative Inverse.
    /// $A^{-1} = \tilde{A} / |A|^2$ (For Versors).
    /// Requires Division (Field).
    pub fn inverse(&self) -> Result<Self, CausalMultiVectorError> {
        let mag_sq = self.squared_magnitude_impl();

        if mag_sq.abs() <= T::epsilon() {
            return Err(CausalMultiVectorError::zero_magnitude());
        }

        let conjugate = self.reversion_impl();
        let scale = T::one() / mag_sq;

        Ok(conjugate.scale(scale))
    }

    /// The Geometric Product for Commutative Coefficients.
    /// This is the standard CPU implementation.
    pub fn geometric_product(&self, rhs: &Self) -> Self {
        self.geometric_product_impl(rhs)
    }

    /// Computes the Euclidean squared magnitude of a 3D spatial vector.
    ///
    /// For 4D Lorentzian multivectors with spatial components at indices 2, 3, 4
    /// (corresponding to x, y, z), this returns:
    ///
    /// $$ |v|^2_{\text{Euclidean}} = v_x^2 + v_y^2 + v_z^2 $$
    ///
    /// This differs from `squared_magnitude()` which applies the Lorentzian metric
    /// signature, potentially yielding negative values for spatial vectors.
    ///
    /// # Use Case
    /// Use this for classical EM quantities like energy density where the physical
    /// norm must be positive-definite.
    pub fn euclidean_squared_magnitude_3d(&self) -> T {
        let vx = self.data.get(2).copied().unwrap_or_else(T::zero);
        let vy = self.data.get(3).copied().unwrap_or_else(T::zero);
        let vz = self.data.get(4).copied().unwrap_or_else(T::zero);
        vx * vx + vy * vy + vz * vz
    }

    /// Computes the Euclidean magnitude of a 3D spatial vector.
    ///
    /// $$ |v|_{\text{Euclidean}} = \sqrt{v_x^2 + v_y^2 + v_z^2} $$
    pub fn euclidean_magnitude_3d(&self) -> T {
        self.euclidean_squared_magnitude_3d().sqrt()
    }

    /// Computes the 3D Euclidean cross product of two spatial vectors.
    ///
    /// For vectors with spatial components at indices 2, 3, 4 (x, y, z):
    ///
    /// $$ \mathbf{a} \times \mathbf{b} = (a_y b_z - a_z b_y, a_z b_x - a_x b_z, a_x b_y - a_y b_x) $$
    ///
    /// The result is returned in the same multivector format with the cross product
    /// components at indices 2, 3, 4.
    ///
    /// # Use Case
    /// Use this for classical EM quantities like the Poynting vector S = E × B.
    pub fn euclidean_cross_product_3d(&self, rhs: &Self) -> Self {
        let ax = self.data.get(2).copied().unwrap_or_else(T::zero);
        let ay = self.data.get(3).copied().unwrap_or_else(T::zero);
        let az = self.data.get(4).copied().unwrap_or_else(T::zero);

        let bx = rhs.data.get(2).copied().unwrap_or_else(T::zero);
        let by = rhs.data.get(3).copied().unwrap_or_else(T::zero);
        let bz = rhs.data.get(4).copied().unwrap_or_else(T::zero);

        // Cross product: c = a × b
        let cx = ay * bz - az * by;
        let cy = az * bx - ax * bz;
        let cz = ax * by - ay * bx;

        let mut result_data = vec![T::zero(); self.data.len()];
        result_data[2] = cx;
        result_data[3] = cy;
        result_data[4] = cz;

        Self {
            data: result_data,
            metric: self.metric,
        }
    }
}
