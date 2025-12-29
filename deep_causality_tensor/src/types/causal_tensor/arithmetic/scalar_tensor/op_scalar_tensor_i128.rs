/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i128 ---
impl<'a> Add<&'a CpuTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn add(self, rhs: &'a CpuTensor<i128>) -> Self::Output {
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
impl Add<CpuTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn add(self, rhs: CpuTensor<i128>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn sub(self, rhs: &'a CpuTensor<i128>) -> Self::Output {
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
impl Sub<CpuTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn sub(self, rhs: CpuTensor<i128>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn mul(self, rhs: &'a CpuTensor<i128>) -> Self::Output {
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
impl Mul<CpuTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn mul(self, rhs: CpuTensor<i128>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn div(self, rhs: &'a CpuTensor<i128>) -> Self::Output {
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
impl Div<CpuTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Clone,
{
    type Output = CpuTensor<i128>;
    fn div(self, rhs: CpuTensor<i128>) -> Self::Output {
        self.div(&rhs)
    }
}
