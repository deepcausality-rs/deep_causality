/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
}
