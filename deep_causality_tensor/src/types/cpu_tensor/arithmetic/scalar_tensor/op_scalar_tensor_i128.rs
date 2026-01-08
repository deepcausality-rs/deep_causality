/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i128 ---
impl<'a> Add<&'a InternalCpuTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn add(self, rhs: &'a InternalCpuTensor<i128>) -> Self::Output {
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
impl Add<InternalCpuTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn add(self, rhs: InternalCpuTensor<i128>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn sub(self, rhs: &'a InternalCpuTensor<i128>) -> Self::Output {
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
impl Sub<InternalCpuTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn sub(self, rhs: InternalCpuTensor<i128>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn mul(self, rhs: &'a InternalCpuTensor<i128>) -> Self::Output {
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
impl Mul<InternalCpuTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn mul(self, rhs: InternalCpuTensor<i128>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn div(self, rhs: &'a InternalCpuTensor<i128>) -> Self::Output {
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
impl Div<InternalCpuTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Clone,
{
    type Output = InternalCpuTensor<i128>;
    fn div(self, rhs: InternalCpuTensor<i128>) -> Self::Output {
        self.div(&rhs)
    }
}
