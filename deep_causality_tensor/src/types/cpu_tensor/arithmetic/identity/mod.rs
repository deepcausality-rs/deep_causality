/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::InternalCpuTensor;
use deep_causality_num::{One, Zero};

//
// Implement Zero trait for InternalCpuTensor
//
impl<T> Zero for InternalCpuTensor<T>
where
    T: Zero + Clone + Default + PartialOrd,
{
    fn zero() -> Self {
        // Since we don't know the shape, we can only return an empty tensor
        // or a default 0-dim tensor.
        // Usually `Zero` implies additive identity.
        // For dynamic tensors, `zero` usually requires a shape.
        // This trait method `non-parameterized` is hard for tensors.
        // We will return an empty tensor as a placeholder or panic?
        // Returning empty tensor is safer.
        InternalCpuTensor::new(Vec::new(), Vec::new()).unwrap()
    }

    fn is_zero(&self) -> bool {
        self.data.iter().all(|x| x.is_zero())
    }
}

//
// Implement One trait for InternalCpuTensor
//
impl<T> One for InternalCpuTensor<T>
where
    T: One + Clone + Default + PartialOrd,
{
    fn one() -> Self {
        // Similar to Zero, return empty.
        InternalCpuTensor::new(Vec::new(), Vec::new()).unwrap()
    }

    fn is_one(&self) -> bool {
        self.data.iter().all(|x| x.is_one())
    }
}
