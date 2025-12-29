/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i16 ---
impl<'a> Add<&'a InternalCpuTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn add(self, rhs: &'a InternalCpuTensor<i16>) -> Self::Output {
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
impl Add<InternalCpuTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn add(self, rhs: InternalCpuTensor<i16>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn sub(self, rhs: &'a InternalCpuTensor<i16>) -> Self::Output {
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
impl Sub<InternalCpuTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn sub(self, rhs: InternalCpuTensor<i16>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn mul(self, rhs: &'a InternalCpuTensor<i16>) -> Self::Output {
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
impl Mul<InternalCpuTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn mul(self, rhs: InternalCpuTensor<i16>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn div(self, rhs: &'a InternalCpuTensor<i16>) -> Self::Output {
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
impl Div<InternalCpuTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Clone,
{
    type Output = InternalCpuTensor<i16>;
    fn div(self, rhs: InternalCpuTensor<i16>) -> Self::Output {
        self.div(&rhs)
    }
}
