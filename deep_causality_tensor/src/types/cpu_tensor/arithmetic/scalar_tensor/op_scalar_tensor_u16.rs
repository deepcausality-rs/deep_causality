/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u16 ---
impl<'a> Add<&'a InternalCpuTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn add(self, rhs: &'a InternalCpuTensor<u16>) -> Self::Output {
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
impl Add<InternalCpuTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn add(self, rhs: InternalCpuTensor<u16>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn sub(self, rhs: &'a InternalCpuTensor<u16>) -> Self::Output {
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
impl Sub<InternalCpuTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn sub(self, rhs: InternalCpuTensor<u16>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn mul(self, rhs: &'a InternalCpuTensor<u16>) -> Self::Output {
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
impl Mul<InternalCpuTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn mul(self, rhs: InternalCpuTensor<u16>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn div(self, rhs: &'a InternalCpuTensor<u16>) -> Self::Output {
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
impl Div<InternalCpuTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Clone,
{
    type Output = InternalCpuTensor<u16>;
    fn div(self, rhs: InternalCpuTensor<u16>) -> Self::Output {
        self.div(&rhs)
    }
}
