/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i32 ---
impl<'a> Add<&'a CpuTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn add(self, rhs: &'a CpuTensor<i32>) -> Self::Output {
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
impl Add<CpuTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn add(self, rhs: CpuTensor<i32>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn sub(self, rhs: &'a CpuTensor<i32>) -> Self::Output {
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
impl Sub<CpuTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn sub(self, rhs: CpuTensor<i32>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn mul(self, rhs: &'a CpuTensor<i32>) -> Self::Output {
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
impl Mul<CpuTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn mul(self, rhs: CpuTensor<i32>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn div(self, rhs: &'a CpuTensor<i32>) -> Self::Output {
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
impl Div<CpuTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Clone,
{
    type Output = CpuTensor<i32>;
    fn div(self, rhs: CpuTensor<i32>) -> Self::Output {
        self.div(&rhs)
    }
}
