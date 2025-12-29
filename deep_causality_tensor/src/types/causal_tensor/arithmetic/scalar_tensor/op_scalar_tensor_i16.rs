/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- i16 ---
impl<'a> Add<&'a CpuTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn add(self, rhs: &'a CpuTensor<i16>) -> Self::Output {
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
impl Add<CpuTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn add(self, rhs: CpuTensor<i16>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn sub(self, rhs: &'a CpuTensor<i16>) -> Self::Output {
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
impl Sub<CpuTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn sub(self, rhs: CpuTensor<i16>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn mul(self, rhs: &'a CpuTensor<i16>) -> Self::Output {
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
impl Mul<CpuTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn mul(self, rhs: CpuTensor<i16>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn div(self, rhs: &'a CpuTensor<i16>) -> Self::Output {
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
impl Div<CpuTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Clone,
{
    type Output = CpuTensor<i16>;
    fn div(self, rhs: CpuTensor<i16>) -> Self::Output {
        self.div(&rhs)
    }
}
