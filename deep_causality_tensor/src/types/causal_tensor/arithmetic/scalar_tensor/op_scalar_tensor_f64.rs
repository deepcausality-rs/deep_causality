/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- f64 ---
// Implementation for `f64 + &CpuTensor<f64>`
impl<'a> Add<&'a CpuTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;

    fn add(self, rhs: &'a CpuTensor<f64>) -> Self::Output {
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

// Implementation for `f64 + CpuTensor<f64>` (consuming)
impl Add<CpuTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;
    fn add(self, rhs: CpuTensor<f64>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- f64 ---
impl<'a> Sub<&'a CpuTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;

    fn sub(self, rhs: &'a CpuTensor<f64>) -> Self::Output {
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

impl Sub<CpuTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;
    fn sub(self, rhs: CpuTensor<f64>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- f64 ---
impl<'a> Mul<&'a CpuTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;

    fn mul(self, rhs: &'a CpuTensor<f64>) -> Self::Output {
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

impl Mul<CpuTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;
    fn mul(self, rhs: CpuTensor<f64>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- f64 ---
impl<'a> Div<&'a CpuTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;

    fn div(self, rhs: &'a CpuTensor<f64>) -> Self::Output {
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

impl Div<CpuTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Clone,
{
    type Output = CpuTensor<f64>;
    fn div(self, rhs: CpuTensor<f64>) -> Self::Output {
        self.div(&rhs)
    }
}
