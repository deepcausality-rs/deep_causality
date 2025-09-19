/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::causal_tensor::CausalTensor;
use std::ops::{Add, Div, Mul, Sub};

// ############################################################################
// ############################### ADDITION ###################################
// ############################################################################

// --- f32 ---
// Implementation for `f32 + &CausalTensor<f32>`
impl<'a> Add<&'a CausalTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;

    fn add(self, rhs: &'a CausalTensor<f32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

// Implementation for `f32 + CausalTensor<f32>` (consuming)
impl Add<CausalTensor<f32>> for f32
where
    f32: Add<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;
    fn add(self, rhs: CausalTensor<f32>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- f64 ---
// Implementation for `f64 + &CausalTensor<f64>`
impl<'a> Add<&'a CausalTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;

    fn add(self, rhs: &'a CausalTensor<f64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

// Implementation for `f64 + CausalTensor<f64>` (consuming)
impl Add<CausalTensor<f64>> for f64
where
    f64: Add<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;
    fn add(self, rhs: CausalTensor<f64>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- i8 ---
impl<'a> Add<&'a CausalTensor<i8>> for i8
where
    i8: Add<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;

    fn add(self, rhs: &'a CausalTensor<i8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<i8>> for i8
where
    i8: Add<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;
    fn add(self, rhs: CausalTensor<i8>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- i16 ---
impl<'a> Add<&'a CausalTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;

    fn add(self, rhs: &'a CausalTensor<i16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<i16>> for i16
where
    i16: Add<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;
    fn add(self, rhs: CausalTensor<i16>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- i32 ---
impl<'a> Add<&'a CausalTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;

    fn add(self, rhs: &'a CausalTensor<i32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<i32>> for i32
where
    i32: Add<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;
    fn add(self, rhs: CausalTensor<i32>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- i64 ---
impl<'a> Add<&'a CausalTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;

    fn add(self, rhs: &'a CausalTensor<i64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<i64>> for i64
where
    i64: Add<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;
    fn add(self, rhs: CausalTensor<i64>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- i128 ---
impl<'a> Add<&'a CausalTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;

    fn add(self, rhs: &'a CausalTensor<i128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<i128>> for i128
where
    i128: Add<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;
    fn add(self, rhs: CausalTensor<i128>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- u8 ---
impl<'a> Add<&'a CausalTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;

    fn add(self, rhs: &'a CausalTensor<u8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<u8>> for u8
where
    u8: Add<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;
    fn add(self, rhs: CausalTensor<u8>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- u16 ---
impl<'a> Add<&'a CausalTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;

    fn add(self, rhs: &'a CausalTensor<u16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<u16>> for u16
where
    u16: Add<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;
    fn add(self, rhs: CausalTensor<u16>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- u32 ---
impl<'a> Add<&'a CausalTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;

    fn add(self, rhs: &'a CausalTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<u32>> for u32
where
    u32: Add<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;
    fn add(self, rhs: CausalTensor<u32>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- u64 ---
impl<'a> Add<&'a CausalTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;

    fn add(self, rhs: &'a CausalTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<u64>> for u64
where
    u64: Add<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;
    fn add(self, rhs: CausalTensor<u64>) -> Self::Output {
        self.add(&rhs)
    }
}

// --- u128 ---
impl<'a> Add<&'a CausalTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;

    fn add(self, rhs: &'a CausalTensor<u128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.add(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Add<CausalTensor<u128>> for u128
where
    u128: Add<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;
    fn add(self, rhs: CausalTensor<u128>) -> Self::Output {
        self.add(&rhs)
    }
}

// ############################################################################
// ############################## SUBTRACTION #################################
// ############################################################################

// --- f32 ---
impl<'a> Sub<&'a CausalTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;

    fn sub(self, rhs: &'a CausalTensor<f32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<f32>> for f32
where
    f32: Sub<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;
    fn sub(self, rhs: CausalTensor<f32>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- f64 ---
impl<'a> Sub<&'a CausalTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;

    fn sub(self, rhs: &'a CausalTensor<f64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<f64>> for f64
where
    f64: Sub<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;
    fn sub(self, rhs: CausalTensor<f64>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- i8 ---
impl<'a> Sub<&'a CausalTensor<i8>> for i8
where
    i8: Sub<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;

    fn sub(self, rhs: &'a CausalTensor<i8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<i8>> for i8
where
    i8: Sub<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;
    fn sub(self, rhs: CausalTensor<i8>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- i16 ---
impl<'a> Sub<&'a CausalTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;

    fn sub(self, rhs: &'a CausalTensor<i16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<i16>> for i16
where
    i16: Sub<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;
    fn sub(self, rhs: CausalTensor<i16>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- i32 ---
impl<'a> Sub<&'a CausalTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;

    fn sub(self, rhs: &'a CausalTensor<i32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<i32>> for i32
where
    i32: Sub<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;
    fn sub(self, rhs: CausalTensor<i32>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- i64 ---
impl<'a> Sub<&'a CausalTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;

    fn sub(self, rhs: &'a CausalTensor<i64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<i64>> for i64
where
    i64: Sub<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;
    fn sub(self, rhs: CausalTensor<i64>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- i128 ---
impl<'a> Sub<&'a CausalTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;

    fn sub(self, rhs: &'a CausalTensor<i128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<i128>> for i128
where
    i128: Sub<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;
    fn sub(self, rhs: CausalTensor<i128>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- u8 ---
impl<'a> Sub<&'a CausalTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;

    fn sub(self, rhs: &'a CausalTensor<u8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<u8>> for u8
where
    u8: Sub<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;
    fn sub(self, rhs: CausalTensor<u8>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- u16 ---
impl<'a> Sub<&'a CausalTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;

    fn sub(self, rhs: &'a CausalTensor<u16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<u16>> for u16
where
    u16: Sub<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;
    fn sub(self, rhs: CausalTensor<u16>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- u32 ---
impl<'a> Sub<&'a CausalTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;

    fn sub(self, rhs: &'a CausalTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<u32>> for u32
where
    u32: Sub<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;
    fn sub(self, rhs: CausalTensor<u32>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- u64 ---
impl<'a> Sub<&'a CausalTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;

    fn sub(self, rhs: &'a CausalTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<u64>> for u64
where
    u64: Sub<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;
    fn sub(self, rhs: CausalTensor<u64>) -> Self::Output {
        self.sub(&rhs)
    }
}

// --- u128 ---
impl<'a> Sub<&'a CausalTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;

    fn sub(self, rhs: &'a CausalTensor<u128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.sub(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Sub<CausalTensor<u128>> for u128
where
    u128: Sub<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;
    fn sub(self, rhs: CausalTensor<u128>) -> Self::Output {
        self.sub(&rhs)
    }
}

// ############################################################################
// ############################ MULTIPLICATION ################################
// ############################################################################

// --- f32 ---
impl<'a> Mul<&'a CausalTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;

    fn mul(self, rhs: &'a CausalTensor<f32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<f32>> for f32
where
    f32: Mul<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;
    fn mul(self, rhs: CausalTensor<f32>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- f64 ---
impl<'a> Mul<&'a CausalTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;

    fn mul(self, rhs: &'a CausalTensor<f64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<f64>> for f64
where
    f64: Mul<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;
    fn mul(self, rhs: CausalTensor<f64>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- i8 ---
impl<'a> Mul<&'a CausalTensor<i8>> for i8
where
    i8: Mul<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;

    fn mul(self, rhs: &'a CausalTensor<i8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<i8>> for i8
where
    i8: Mul<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;
    fn mul(self, rhs: CausalTensor<i8>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- i16 ---
impl<'a> Mul<&'a CausalTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;

    fn mul(self, rhs: &'a CausalTensor<i16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<i16>> for i16
where
    i16: Mul<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;
    fn mul(self, rhs: CausalTensor<i16>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- i32 ---
impl<'a> Mul<&'a CausalTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;

    fn mul(self, rhs: &'a CausalTensor<i32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<i32>> for i32
where
    i32: Mul<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;
    fn mul(self, rhs: CausalTensor<i32>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- i64 ---
impl<'a> Mul<&'a CausalTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;

    fn mul(self, rhs: &'a CausalTensor<i64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<i64>> for i64
where
    i64: Mul<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;
    fn mul(self, rhs: CausalTensor<i64>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- i128 ---
impl<'a> Mul<&'a CausalTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;

    fn mul(self, rhs: &'a CausalTensor<i128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<i128>> for i128
where
    i128: Mul<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;
    fn mul(self, rhs: CausalTensor<i128>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- u8 ---
impl<'a> Mul<&'a CausalTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;

    fn mul(self, rhs: &'a CausalTensor<u8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<u8>> for u8
where
    u8: Mul<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;
    fn mul(self, rhs: CausalTensor<u8>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- u16 ---
impl<'a> Mul<&'a CausalTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;

    fn mul(self, rhs: &'a CausalTensor<u16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<u16>> for u16
where
    u16: Mul<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;
    fn mul(self, rhs: CausalTensor<u16>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- u32 ---
impl<'a> Mul<&'a CausalTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;

    fn mul(self, rhs: &'a CausalTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<u32>> for u32
where
    u32: Mul<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;
    fn mul(self, rhs: CausalTensor<u32>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- u64 ---
impl<'a> Mul<&'a CausalTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;

    fn mul(self, rhs: &'a CausalTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<u64>> for u64
where
    u64: Mul<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;
    fn mul(self, rhs: CausalTensor<u64>) -> Self::Output {
        self.mul(&rhs)
    }
}

// --- u128 ---
impl<'a> Mul<&'a CausalTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;

    fn mul(self, rhs: &'a CausalTensor<u128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.mul(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Mul<CausalTensor<u128>> for u128
where
    u128: Mul<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;
    fn mul(self, rhs: CausalTensor<u128>) -> Self::Output {
        self.mul(&rhs)
    }
}

// ############################################################################
// ############################### DIVISION ###################################
// ############################################################################

// --- f32 ---
impl<'a> Div<&'a CausalTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;

    fn div(self, rhs: &'a CausalTensor<f32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<f32>> for f32
where
    f32: Div<f32, Output = f32> + Copy,
{
    type Output = CausalTensor<f32>;
    fn div(self, rhs: CausalTensor<f32>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- f64 ---
impl<'a> Div<&'a CausalTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;

    fn div(self, rhs: &'a CausalTensor<f64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<f64>> for f64
where
    f64: Div<f64, Output = f64> + Copy,
{
    type Output = CausalTensor<f64>;
    fn div(self, rhs: CausalTensor<f64>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- i8 ---
impl<'a> Div<&'a CausalTensor<i8>> for i8
where
    i8: Div<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;

    fn div(self, rhs: &'a CausalTensor<i8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<i8>> for i8
where
    i8: Div<i8, Output = i8> + Copy,
{
    type Output = CausalTensor<i8>;
    fn div(self, rhs: CausalTensor<i8>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- i16 ---
impl<'a> Div<&'a CausalTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;

    fn div(self, rhs: &'a CausalTensor<i16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<i16>> for i16
where
    i16: Div<i16, Output = i16> + Copy,
{
    type Output = CausalTensor<i16>;
    fn div(self, rhs: CausalTensor<i16>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- i32 ---
impl<'a> Div<&'a CausalTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;

    fn div(self, rhs: &'a CausalTensor<i32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<i32>> for i32
where
    i32: Div<i32, Output = i32> + Copy,
{
    type Output = CausalTensor<i32>;
    fn div(self, rhs: CausalTensor<i32>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- i64 ---
impl<'a> Div<&'a CausalTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;

    fn div(self, rhs: &'a CausalTensor<i64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<i64>> for i64
where
    i64: Div<i64, Output = i64> + Copy,
{
    type Output = CausalTensor<i64>;
    fn div(self, rhs: CausalTensor<i64>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- i128 ---
impl<'a> Div<&'a CausalTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;

    fn div(self, rhs: &'a CausalTensor<i128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<i128>> for i128
where
    i128: Div<i128, Output = i128> + Copy,
{
    type Output = CausalTensor<i128>;
    fn div(self, rhs: CausalTensor<i128>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- u8 ---
impl<'a> Div<&'a CausalTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;

    fn div(self, rhs: &'a CausalTensor<u8>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<u8>> for u8
where
    u8: Div<u8, Output = u8> + Copy,
{
    type Output = CausalTensor<u8>;
    fn div(self, rhs: CausalTensor<u8>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- u16 ---
impl<'a> Div<&'a CausalTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;

    fn div(self, rhs: &'a CausalTensor<u16>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<u16>> for u16
where
    u16: Div<u16, Output = u16> + Copy,
{
    type Output = CausalTensor<u16>;
    fn div(self, rhs: CausalTensor<u16>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- u32 ---
impl<'a> Div<&'a CausalTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;

    fn div(self, rhs: &'a CausalTensor<u32>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<u32>> for u32
where
    u32: Div<u32, Output = u32> + Copy,
{
    type Output = CausalTensor<u32>;
    fn div(self, rhs: CausalTensor<u32>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- u64 ---
impl<'a> Div<&'a CausalTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;

    fn div(self, rhs: &'a CausalTensor<u64>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<u64>> for u64
where
    u64: Div<u64, Output = u64> + Copy,
{
    type Output = CausalTensor<u64>;
    fn div(self, rhs: CausalTensor<u64>) -> Self::Output {
        self.div(&rhs)
    }
}

// --- u128 ---
impl<'a> Div<&'a CausalTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;

    fn div(self, rhs: &'a CausalTensor<u128>) -> Self::Output {
        let mut new_data = rhs.data.clone();
        for item in &mut new_data {
            *item = self.div(*item);
        }
        CausalTensor {
            data: new_data,
            shape: rhs.shape.clone(),
            strides: rhs.strides.clone(),
        }
    }
}

impl Div<CausalTensor<u128>> for u128
where
    u128: Div<u128, Output = u128> + Copy,
{
    type Output = CausalTensor<u128>;
    fn div(self, rhs: CausalTensor<u128>) -> Self::Output {
        self.div(&rhs)
    }
}
