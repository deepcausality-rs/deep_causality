/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u64 ---
impl<'a> Add<&'a InternalCpuTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn add(self, rhs: &'a InternalCpuTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        InternalCpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Add<InternalCpuTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn add(self, rhs: InternalCpuTensor<u64>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn sub(self, rhs: &'a InternalCpuTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        InternalCpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Sub<InternalCpuTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn sub(self, rhs: InternalCpuTensor<u64>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn mul(self, rhs: &'a InternalCpuTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        InternalCpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Mul<InternalCpuTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn mul(self, rhs: InternalCpuTensor<u64>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn div(self, rhs: &'a InternalCpuTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        InternalCpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Div<InternalCpuTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Clone,
{
    type Output = InternalCpuTensor<u64>;
    fn div(self, rhs: InternalCpuTensor<u64>) -> Self::Output {
        self.div(&rhs)
    }
}
