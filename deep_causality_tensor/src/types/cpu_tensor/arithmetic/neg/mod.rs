/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::InternalCpuTensor;
use std::ops::Neg;

//
// Implement Neg trait for InternalCpuTensor
//
impl<T> Neg for InternalCpuTensor<T>
where
    T: Neg<Output = T> + Clone + Default + PartialOrd,
{
    type Output = InternalCpuTensor<T>;

    fn neg(self) -> Self::Output {
        self.unary_op(|a| Ok(-a)).expect("Unary negation failed")
    }
}

impl<T> Neg for &InternalCpuTensor<T>
where
    T: Neg<Output = T> + Clone + Default + PartialOrd,
{
    type Output = InternalCpuTensor<T>;

    fn neg(self) -> Self::Output {
        self.unary_op(|a| Ok(-a)).expect("Unary negation failed")
    }
}
