/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- f32 ---
// Implementation for `f32 + &CpuTensor<f32>`
impl<'a> Add<&'a CpuTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;

    fn add(self, rhs: &'a CpuTensor<f32>) -> Self::Output {
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

// Implementation for `f32 + CpuTensor<f32>` (consuming)
impl Add<CpuTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;
    fn add(self, rhs: CpuTensor<f32>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- f32 ---
impl<'a> Sub<&'a CpuTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;

    fn sub(self, rhs: &'a CpuTensor<f32>) -> Self::Output {
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

impl Sub<CpuTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;
    fn sub(self, rhs: CpuTensor<f32>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- f32 ---
impl<'a> Mul<&'a CpuTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;

    fn mul(self, rhs: &'a CpuTensor<f32>) -> Self::Output {
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

impl Mul<CpuTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;
    fn mul(self, rhs: CpuTensor<f32>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- f32 ---
impl<'a> Div<&'a CpuTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;

    fn div(self, rhs: &'a CpuTensor<f32>) -> Self::Output {
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

impl Div<CpuTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Clone,
{
    type Output = CpuTensor<f32>;
    fn div(self, rhs: CpuTensor<f32>) -> Self::Output {
        self.div(&rhs)
    }
}
