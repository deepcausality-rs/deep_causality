/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::BackendTensor;
use crate::{TensorBackend, TensorData};
use core::ops::Neg;
use core::ops::Sub;
use deep_causality_num::{Associative, Commutative, Distributive, One, Zero};

// --- Identity Elements (Zero, One) ---

impl<T, B> Zero for BackendTensor<T, B>
where
    T: TensorData, // Use TensorData to ensure inner elements have properties
    B: TensorBackend,
    B::Tensor<T>: Zero,
{
    fn zero() -> Self {
        Self::from_inner(B::Tensor::<T>::zero())
    }

    fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }
}

impl<T, B> One for BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend,
    B::Tensor<T>: One,
{
    fn one() -> Self {
        Self::from_inner(B::Tensor::<T>::one())
    }

    fn is_one(&self) -> bool {
        self.inner.is_one()
    }
}

// --- Negation ---

impl<T, B> Neg for BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend,
    Self: Sub<Output = Self> + Zero,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        // 0 - x
        Self::zero() - self
    }
}

// --- Algebraic Markers ---

impl<T, B> Associative for BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend,
    B::Tensor<T>: Associative,
{
}

impl<T, B> Distributive for BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend,
    B::Tensor<T>: Distributive,
{
}

impl<T, B> Commutative for BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend,
    B::Tensor<T>: Commutative,
{
}
