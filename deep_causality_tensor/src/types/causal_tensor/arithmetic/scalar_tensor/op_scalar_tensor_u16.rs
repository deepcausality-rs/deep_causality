/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u16 ---
impl<'a> Add<&'a CpuTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn add(self, rhs: &'a CpuTensor<u16>) -> Self::Output {
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
impl Add<CpuTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn add(self, rhs: CpuTensor<u16>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn sub(self, rhs: &'a CpuTensor<u16>) -> Self::Output {
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
impl Sub<CpuTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn sub(self, rhs: CpuTensor<u16>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn mul(self, rhs: &'a CpuTensor<u16>) -> Self::Output {
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
impl Mul<CpuTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn mul(self, rhs: CpuTensor<u16>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn div(self, rhs: &'a CpuTensor<u16>) -> Self::Output {
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
impl Div<CpuTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Clone,
{
    type Output = CpuTensor<u16>;
    fn div(self, rhs: CpuTensor<u16>) -> Self::Output {
        self.div(&rhs)
    }
}
