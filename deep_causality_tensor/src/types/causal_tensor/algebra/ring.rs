/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensor;
use deep_causality_num::{Associative, Distributive, Ring};

// Implement Associative marker trait
impl<T> Associative for CausalTensor<T> where T: Associative + Copy {}

// Implement Distributive marker trait
impl<T> Distributive for CausalTensor<T> where T: Distributive + Copy {}

// Ring is automatically implemented by blanket impl in deep_causality_num
// because CausalTensor implements AbelianGroup, MulMonoid (via One+Associative+Mul), Distributive.

// Module is automatically implemented by blanket impl in deep_causality_num
// because CausalTensor implements AbelianGroup and Mul<S>.

impl<T> CausalTensor<T>
where
    T: Ring + Copy,
{
    /// Element-wise multiplication (Hadamard product).
    pub fn mul(&self, rhs: &Self) -> Self {
        if self.shape() != rhs.shape() {
            panic!(
                "Shape mismatch in multiplication: {:?} vs {:?}",
                self.shape(),
                rhs.shape()
            );
        }

        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a * *b)
            .collect();

        Self::from_vec_and_shape_unchecked(data, self.shape())
    }

    /// Creates a tensor of ones with the specified shape.
    pub fn one(shape: &[usize]) -> Self {
        let size: usize = shape.iter().product();
        let data = vec![T::one(); size];
        Self::from_vec_and_shape_unchecked(data, shape)
    }
}
