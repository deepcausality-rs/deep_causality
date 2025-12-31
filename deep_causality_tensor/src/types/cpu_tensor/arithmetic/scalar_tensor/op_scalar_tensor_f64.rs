/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- f64 ---
// Implementation for `f64 + &InternalCpuTensor<f64>`
impl<'a> Add<&'a InternalCpuTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;

    fn add(self, rhs: &'a InternalCpuTensor<f64>) -> Self::Output {
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

// Implementation for `f64 + InternalCpuTensor<f64>` (consuming)
impl Add<InternalCpuTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;
    fn add(self, rhs: InternalCpuTensor<f64>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- f64 ---
impl<'a> Sub<&'a InternalCpuTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;

    fn sub(self, rhs: &'a InternalCpuTensor<f64>) -> Self::Output {
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

impl Sub<InternalCpuTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;
    fn sub(self, rhs: InternalCpuTensor<f64>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- f64 ---
impl<'a> Mul<&'a InternalCpuTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;

    fn mul(self, rhs: &'a InternalCpuTensor<f64>) -> Self::Output {
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

impl Mul<InternalCpuTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;
    fn mul(self, rhs: InternalCpuTensor<f64>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- f64 ---
impl<'a> Div<&'a InternalCpuTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;

    fn div(self, rhs: &'a InternalCpuTensor<f64>) -> Self::Output {
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

impl Div<InternalCpuTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Clone,
{
    type Output = InternalCpuTensor<f64>;
    fn div(self, rhs: InternalCpuTensor<f64>) -> Self::Output {
        self.div(&rhs)
    }
}
