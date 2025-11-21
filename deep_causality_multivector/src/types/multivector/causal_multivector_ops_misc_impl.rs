/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, CausalMultiVectorError, MultiVector};
use deep_causality_num::Num;
use std::ops::{AddAssign, Neg, SubAssign};

impl<T> CausalMultiVector<T> {
    /// Projects the multivector onto a specific grade $k$.
    ///
    /// $$ \langle A \rangle_k = \sum_{I : |I|=k} a_I e_I $$
    pub(super) fn grade_projection_impl(&self, k: u32) -> Self
    where
        T: Num + Copy + Clone,
    {
        let mut result_data = vec![T::zero(); self.data.len()];
        for (i, val) in self.data.iter().enumerate() {
            if i.count_ones() == k {
                result_data[i] = *val;
            }
        }
        Self {
            data: result_data,
            metric: self.metric,
        }
    }
    /// Computes the reverse of the multivector, denoted $\tilde{A}$ or $A^\dagger$.
    ///
    /// Reverses the order of vectors in each basis blade.
    /// $$ \tilde{A} = \sum_{k=0}^N (-1)^{k(k-1)/2} \langle A \rangle_k $$
    pub(super) fn reversion_impl(&self) -> Self
    where
        T: Num + Copy + Clone + Neg<Output = T>,
    {
        let mut result_data = vec![T::zero(); self.data.len()];
        for (i, val) in self.data.iter().enumerate() {
            let grade = i.count_ones() as i32;
            let sign_power = (grade * (grade - 1)) / 2;
            if sign_power % 2 != 0 {
                result_data[i] = -(*val);
            } else {
                result_data[i] = *val;
            }
        }
        Self {
            data: result_data,
            metric: self.metric,
        }
    }
    /// Computes the squared magnitude (squared norm) of the multivector.
    ///
    /// $$ ||A||^2 = \langle A \tilde{A} \rangle_0 $$
    pub(super) fn squared_magnitude_impl(&self) -> T
    where
        T: Num + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        let reverse = self.reversion();
        // We can optimize by only calculating the scalar part of the product
        // But for simplicity/correctness, let's use the full product
        let product = self.clone() * reverse;
        product.data[0] // Scalar part
    }
    /// Computes the inverse of the multivector $A^{-1}$.
    ///
    /// $$ A^{-1} = \frac{\tilde{A}}{A \tilde{A}} $$
    ///
    /// Only valid if $A \tilde{A}$ is a non-zero scalar (Versor).
    pub(super) fn inverse_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Num
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + std::ops::Div<Output = T>
            + PartialEq,
    {
        let sq_mag = self.squared_magnitude();
        if sq_mag == T::zero() {
            return Err(CausalMultiVectorError::zero_magnitude());
        }

        let reverse = self.reversion();
        Ok(reverse / sq_mag)
    }
    /// Computes the dual of the multivector $A^*$.
    ///
    /// $$ A^* = A I^{-1} $$
    /// where $I$ is the pseudoscalar.
    pub(super) fn dual_impl(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Num
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + std::ops::Div<Output = T>
            + PartialEq,
    {
        let pseudo = Self::pseudoscalar(self.metric);
        let pseudo_inv = pseudo.inverse()?;
        Ok(self.clone() * pseudo_inv)
    }

    /// Cyclically shifts the basis coefficients.
    /// This effectively changes the "viewpoint" of the algebra,
    /// making the coefficient at `index` the new scalar (index 0).
    ///
    /// Used for Comonadic 'extend' operations.
    pub(super) fn basis_shift_impl(&self, index: usize) -> Self
    where
        T: Clone,
    {
        // Null check is no needed since length is guaranteed to be >0
        let mut new_data = self.data.clone();
        let shift = index % self.data.len();
        new_data.rotate_left(shift);

        Self {
            data: new_data,
            metric: self.metric,
        }
    }
}
