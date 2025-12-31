/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u128 ---
impl<'a> Add<&'a InternalCpuTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn add(self, rhs: &'a InternalCpuTensor<u128>) -> Self::Output {
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
impl Add<InternalCpuTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn add(self, rhs: InternalCpuTensor<u128>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn sub(self, rhs: &'a InternalCpuTensor<u128>) -> Self::Output {
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
impl Sub<InternalCpuTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn sub(self, rhs: InternalCpuTensor<u128>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn mul(self, rhs: &'a InternalCpuTensor<u128>) -> Self::Output {
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
impl Mul<InternalCpuTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn mul(self, rhs: InternalCpuTensor<u128>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn div(self, rhs: &'a InternalCpuTensor<u128>) -> Self::Output {
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
impl Div<InternalCpuTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Clone,
{
    type Output = InternalCpuTensor<u128>;
    fn div(self, rhs: InternalCpuTensor<u128>) -> Self::Output {
        self.div(&rhs)
    }
}
