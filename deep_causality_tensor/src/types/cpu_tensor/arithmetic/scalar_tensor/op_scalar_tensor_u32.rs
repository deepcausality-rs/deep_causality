/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u32 ---
impl<'a> Add<&'a InternalCpuTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn add(self, rhs: &'a InternalCpuTensor<u32>) -> Self::Output {
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
impl Add<InternalCpuTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn add(self, rhs: InternalCpuTensor<u32>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn sub(self, rhs: &'a InternalCpuTensor<u32>) -> Self::Output {
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
impl Sub<InternalCpuTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn sub(self, rhs: InternalCpuTensor<u32>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn mul(self, rhs: &'a InternalCpuTensor<u32>) -> Self::Output {
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
impl Mul<InternalCpuTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn mul(self, rhs: InternalCpuTensor<u32>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn div(self, rhs: &'a InternalCpuTensor<u32>) -> Self::Output {
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
impl Div<InternalCpuTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Clone,
{
    type Output = InternalCpuTensor<u32>;
    fn div(self, rhs: InternalCpuTensor<u32>) -> Self::Output {
        self.div(&rhs)
    }
}
