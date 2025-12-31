/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i32 ---
impl<'a> Add<&'a InternalCpuTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn add(self, rhs: &'a InternalCpuTensor<i32>) -> Self::Output {
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
impl Add<InternalCpuTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn add(self, rhs: InternalCpuTensor<i32>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn sub(self, rhs: &'a InternalCpuTensor<i32>) -> Self::Output {
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
impl Sub<InternalCpuTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn sub(self, rhs: InternalCpuTensor<i32>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn mul(self, rhs: &'a InternalCpuTensor<i32>) -> Self::Output {
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
impl Mul<InternalCpuTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn mul(self, rhs: InternalCpuTensor<i32>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn div(self, rhs: &'a InternalCpuTensor<i32>) -> Self::Output {
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
impl Div<InternalCpuTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Clone,
{
    type Output = InternalCpuTensor<i32>;
    fn div(self, rhs: InternalCpuTensor<i32>) -> Self::Output {
        self.div(&rhs)
    }
}
