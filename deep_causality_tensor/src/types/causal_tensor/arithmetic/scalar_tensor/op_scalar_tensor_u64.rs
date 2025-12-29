/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// --- u64 ---
impl<'a> Add<&'a CpuTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn add(self, rhs: &'a CpuTensor<u64>) -> Self::Output {
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
impl Add<CpuTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn add(self, rhs: CpuTensor<u64>) -> Self::Output {
        self.add(&rhs)
    }
}
impl<'a> Sub<&'a CpuTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn sub(self, rhs: &'a CpuTensor<u64>) -> Self::Output {
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
impl Sub<CpuTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn sub(self, rhs: CpuTensor<u64>) -> Self::Output {
        self.sub(&rhs)
    }
}
impl<'a> Mul<&'a CpuTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn mul(self, rhs: &'a CpuTensor<u64>) -> Self::Output {
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
impl Mul<CpuTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn mul(self, rhs: CpuTensor<u64>) -> Self::Output {
        self.mul(&rhs)
    }
}
impl<'a> Div<&'a CpuTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn div(self, rhs: &'a CpuTensor<u64>) -> Self::Output {
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
impl Div<CpuTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Clone,
{
    type Output = CpuTensor<u64>;
    fn div(self, rhs: CpuTensor<u64>) -> Self::Output {
        self.div(&rhs)
    }
}
