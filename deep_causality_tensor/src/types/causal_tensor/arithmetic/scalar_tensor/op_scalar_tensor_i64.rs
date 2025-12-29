/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i64 ---
impl<'a> Add<&'a CpuTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn add(self, rhs: &'a CpuTensor<i64>) -> Self::Output {
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
impl Add<CpuTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn add(self, rhs: CpuTensor<i64>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn sub(self, rhs: &'a CpuTensor<i64>) -> Self::Output {
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
impl Sub<CpuTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn sub(self, rhs: CpuTensor<i64>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn mul(self, rhs: &'a CpuTensor<i64>) -> Self::Output {
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
impl Mul<CpuTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn mul(self, rhs: CpuTensor<i64>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn div(self, rhs: &'a CpuTensor<i64>) -> Self::Output {
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
impl Div<CpuTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Clone,
{
    type Output = CpuTensor<i64>;
    fn div(self, rhs: CpuTensor<i64>) -> Self::Output {
        self.div(&rhs)
    }
}
