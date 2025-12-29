/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::InternalCpuTensor;
use deep_causality_num::{Associative, Distributive, Ring};

// Marker traits for algebraic properties.
// Note: MulMonoid and Ring are covered by blanket impls in deep_causality_num when
// the type satisfies the requirements. We only impl the marker traits that aren't
// covered by blankets.

impl<T> Associative for InternalCpuTensor<T> where T: Associative + Copy {}

impl<T> Distributive for InternalCpuTensor<T> where T: Distributive + Copy {}

impl<T> InternalCpuTensor<T>
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

    /// Creates an identity matrix (square tensor with 1s on diagonal).
    pub fn identity(shape: &[usize]) -> Result<Self, crate::CausalTensorError> {
        if shape.len() != 2 {
            return Err(crate::CausalTensorError::DimensionMismatch);
        }
        if shape[0] != shape[1] {
            return Err(crate::CausalTensorError::ShapeMismatch);
        }
        let n = shape[0];
        let mut data = vec![T::zero(); n * n];
        for i in 0..n {
            data[i * n + i] = T::one();
        }
        Ok(Self::from_vec_and_shape_unchecked(data, shape))
    }
}
