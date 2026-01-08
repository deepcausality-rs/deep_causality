/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::InternalCpuTensor;
use deep_causality_num::AbelianGroup;

/// `InternalCpuTensor` addition is commutative if T's addition is commutative.
/// This implementation marks `InternalCpuTensor` as an Abelian Group under addition.
/// Note: Group, AddMonoid, InvMonoid are covered by blanket impls in deep_causality_num.
impl<T> AbelianGroup for InternalCpuTensor<T> where T: AbelianGroup + Clone + Default + PartialOrd {}

impl<T> InternalCpuTensor<T>
where
    T: deep_causality_num::AddGroup + Copy + std::ops::Neg<Output = T>,
{
    /// Creates a zero tensor with the specified shape.
    pub fn zero(shape: &[usize]) -> Self {
        let size: usize = shape.iter().product();
        let data = vec![T::zero(); size];
        Self::from_vec_and_shape_unchecked(data, shape)
    }

    /// Element-wise addition.
    pub fn add(&self, rhs: &Self) -> Self {
        if self.shape() != rhs.shape() {
            panic!(
                "Shape mismatch in addition: {:?} vs {:?}",
                self.shape(),
                rhs.shape()
            );
        }

        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        Self::from_vec_and_shape_unchecked(data, self.shape())
    }

    /// Element-wise subtraction.
    pub fn sub(&self, rhs: &Self) -> Self {
        if self.shape() != rhs.shape() {
            panic!(
                "Shape mismatch in subtraction: {:?} vs {:?}",
                self.shape(),
                rhs.shape()
            );
        }

        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a - *b)
            .collect();

        Self::from_vec_and_shape_unchecked(data, self.shape())
    }

    /// Element-wise negation.
    pub fn neg(&self) -> Self {
        let data = self.data.iter().map(|a| -*a).collect();
        Self::from_vec_and_shape_unchecked(data, self.shape())
    }
}
