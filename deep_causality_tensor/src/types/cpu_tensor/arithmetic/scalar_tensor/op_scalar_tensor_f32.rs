/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- f32 ---
// Implementation for `f32 + &InternalCpuTensor<f32>`
impl<'a> Add<&'a InternalCpuTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;

    fn add(self, rhs: &'a InternalCpuTensor<f32>) -> Self::Output {
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

// Implementation for `f32 + InternalCpuTensor<f32>` (consuming)
impl Add<InternalCpuTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;
    fn add(self, rhs: InternalCpuTensor<f32>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- f32 ---
impl<'a> Sub<&'a InternalCpuTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;

    fn sub(self, rhs: &'a InternalCpuTensor<f32>) -> Self::Output {
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

impl Sub<InternalCpuTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;
    fn sub(self, rhs: InternalCpuTensor<f32>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- f32 ---
impl<'a> Mul<&'a InternalCpuTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;

    fn mul(self, rhs: &'a InternalCpuTensor<f32>) -> Self::Output {
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

impl Mul<InternalCpuTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;
    fn mul(self, rhs: InternalCpuTensor<f32>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- f32 ---
impl<'a> Div<&'a InternalCpuTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;

    fn div(self, rhs: &'a InternalCpuTensor<f32>) -> Self::Output {
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

impl Div<InternalCpuTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Clone,
{
    type Output = InternalCpuTensor<f32>;
    fn div(self, rhs: InternalCpuTensor<f32>) -> Self::Output {
        self.div(&rhs)
    }
}
