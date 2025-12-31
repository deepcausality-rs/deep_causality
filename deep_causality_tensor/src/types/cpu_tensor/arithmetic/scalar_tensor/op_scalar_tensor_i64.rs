/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i64 ---
impl<'a> Add<&'a InternalCpuTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn add(self, rhs: &'a InternalCpuTensor<i64>) -> Self::Output {
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
impl Add<InternalCpuTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn add(self, rhs: InternalCpuTensor<i64>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn sub(self, rhs: &'a InternalCpuTensor<i64>) -> Self::Output {
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
impl Sub<InternalCpuTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn sub(self, rhs: InternalCpuTensor<i64>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn mul(self, rhs: &'a InternalCpuTensor<i64>) -> Self::Output {
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
impl Mul<InternalCpuTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn mul(self, rhs: InternalCpuTensor<i64>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn div(self, rhs: &'a InternalCpuTensor<i64>) -> Self::Output {
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
impl Div<InternalCpuTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Clone,
{
    type Output = InternalCpuTensor<i64>;
    fn div(self, rhs: InternalCpuTensor<i64>) -> Self::Output {
        self.div(&rhs)
    }
}
