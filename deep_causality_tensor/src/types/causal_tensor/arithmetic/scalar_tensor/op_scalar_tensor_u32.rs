/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u32 ---
impl<'a> Add<&'a CpuTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn add(self, rhs: &'a CpuTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Add<CpuTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn add(self, rhs: CpuTensor<u32>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn sub(self, rhs: &'a CpuTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Sub<CpuTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn sub(self, rhs: CpuTensor<u32>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn mul(self, rhs: &'a CpuTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Mul<CpuTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn mul(self, rhs: CpuTensor<u32>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn div(self, rhs: &'a CpuTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CpuTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}
impl Div<CpuTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Clone,
{
    type Output = CpuTensor<u32>;
    fn div(self, rhs: CpuTensor<u32>) -> Self::Output {
        self.div(&rhs)
    }
}
