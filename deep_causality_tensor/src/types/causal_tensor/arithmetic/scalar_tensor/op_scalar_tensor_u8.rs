/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u8 ---
impl<'a> Add<&'a CpuTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn add(self, rhs: &'a CpuTensor<u8>) -> Self::Output {
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
impl Add<CpuTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn add(self, rhs: CpuTensor<u8>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn sub(self, rhs: &'a CpuTensor<u8>) -> Self::Output {
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
impl Sub<CpuTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn sub(self, rhs: CpuTensor<u8>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn mul(self, rhs: &'a CpuTensor<u8>) -> Self::Output {
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
impl Mul<CpuTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn mul(self, rhs: CpuTensor<u8>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn div(self, rhs: &'a CpuTensor<u8>) -> Self::Output {
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
impl Div<CpuTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Clone,
{
    type Output = CpuTensor<u8>;
    fn div(self, rhs: CpuTensor<u8>) -> Self::Output {
        self.div(&rhs)
    }
}
