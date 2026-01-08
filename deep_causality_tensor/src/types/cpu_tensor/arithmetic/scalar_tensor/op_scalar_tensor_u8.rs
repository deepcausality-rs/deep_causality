/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u8 ---
impl<'a> Add<&'a InternalCpuTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn add(self, rhs: &'a InternalCpuTensor<u8>) -> Self::Output {
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
impl Add<InternalCpuTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn add(self, rhs: InternalCpuTensor<u8>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a InternalCpuTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn sub(self, rhs: &'a InternalCpuTensor<u8>) -> Self::Output {
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
impl Sub<InternalCpuTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn sub(self, rhs: InternalCpuTensor<u8>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a InternalCpuTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn mul(self, rhs: &'a InternalCpuTensor<u8>) -> Self::Output {
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
impl Mul<InternalCpuTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn mul(self, rhs: InternalCpuTensor<u8>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a InternalCpuTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn div(self, rhs: &'a InternalCpuTensor<u8>) -> Self::Output {
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
impl Div<InternalCpuTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Clone,
{
    type Output = InternalCpuTensor<u8>;
    fn div(self, rhs: InternalCpuTensor<u8>) -> Self::Output {
        self.div(&rhs)
    }
}
