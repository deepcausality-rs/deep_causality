/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::InternalCpuTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- i8 ---
impl<'a> Add<&'a InternalCpuTensor<i8>> for i8
where
    i8: Add<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;

    fn add(self, rhs: &'a InternalCpuTensor<i8>) -> Self::Output {
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

impl Add<InternalCpuTensor<i8>> for i8
where
    i8: Add<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;
    fn add(self, rhs: InternalCpuTensor<i8>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- i8 ---
impl<'a> Sub<&'a InternalCpuTensor<i8>> for i8
where
    i8: Sub<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;

    fn sub(self, rhs: &'a InternalCpuTensor<i8>) -> Self::Output {
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

impl Sub<InternalCpuTensor<i8>> for i8
where
    i8: Sub<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;
    fn sub(self, rhs: InternalCpuTensor<i8>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- i8 ---
impl<'a> Mul<&'a InternalCpuTensor<i8>> for i8
where
    i8: Mul<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;

    fn mul(self, rhs: &'a InternalCpuTensor<i8>) -> Self::Output {
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

impl Mul<InternalCpuTensor<i8>> for i8
where
    i8: Mul<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;
    fn mul(self, rhs: InternalCpuTensor<i8>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- i8 ---
impl<'a> Div<&'a InternalCpuTensor<i8>> for i8
where
    i8: Div<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;

    fn div(self, rhs: &'a InternalCpuTensor<i8>) -> Self::Output {
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

impl Div<InternalCpuTensor<i8>> for i8
where
    i8: Div<i8, Output = i8> + Clone,
{
    type Output = InternalCpuTensor<i8>;
    fn div(self, rhs: InternalCpuTensor<i8>) -> Self::Output {
        self.div(&rhs)
    }
}
