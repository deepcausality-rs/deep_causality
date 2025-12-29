/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u128 ---
impl<'a> Add<&'a CpuTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn add(self, rhs: &'a CpuTensor<u128>) -> Self::Output {
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
impl Add<CpuTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn add(self, rhs: CpuTensor<u128>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn sub(self, rhs: &'a CpuTensor<u128>) -> Self::Output {
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
impl Sub<CpuTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn sub(self, rhs: CpuTensor<u128>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn mul(self, rhs: &'a CpuTensor<u128>) -> Self::Output {
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
impl Mul<CpuTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn mul(self, rhs: CpuTensor<u128>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn div(self, rhs: &'a CpuTensor<u128>) -> Self::Output {
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
impl Div<CpuTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Clone,
{
    type Output = CpuTensor<u128>;
    fn div(self, rhs: CpuTensor<u128>) -> Self::Output {
        self.div(&rhs)
    }
}
