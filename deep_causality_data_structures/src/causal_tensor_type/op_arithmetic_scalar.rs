/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;
use crate::{impl_scalar_tensor_op, impl_tensor_scalar_op};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// --- Element-wise Operations ---
// Implemented via `std::ops` traits for ergonomic use (`+`, `-`, `*`, `/`).
// The initial implementation will focus on the most critical
// operations between a Tensor and a Scalar. Tensor-Tensor operations with
// full broadcasting are a potential future extension.

// Instantiate the macro for Add, Sub, Mul, and Div
impl_tensor_scalar_op!(Add, add, AddAssign, add_assign);
impl_tensor_scalar_op!(Sub, sub, SubAssign, sub_assign);
impl_tensor_scalar_op!(Mul, mul, MulAssign, mul_assign);
impl_tensor_scalar_op!(Div, div, DivAssign, div_assign);

// Instantiate for f32
impl_scalar_tensor_op!(Add, add, f32);
impl_scalar_tensor_op!(Sub, sub, f32);
impl_scalar_tensor_op!(Mul, mul, f32);
impl_scalar_tensor_op!(Div, div, f32);

// Instantiate for f64
impl_scalar_tensor_op!(Add, add, f64);
impl_scalar_tensor_op!(Sub, sub, f64);
impl_scalar_tensor_op!(Mul, mul, f64);
impl_scalar_tensor_op!(Div, div, f64);

// Instantiate for i8
impl_scalar_tensor_op!(Add, add, i8);
impl_scalar_tensor_op!(Sub, sub, i8);
impl_scalar_tensor_op!(Mul, mul, i8);
impl_scalar_tensor_op!(Div, div, i8);

// Instantiate for i16
impl_scalar_tensor_op!(Add, add, i16);
impl_scalar_tensor_op!(Sub, sub, i16);
impl_scalar_tensor_op!(Mul, mul, i16);
impl_scalar_tensor_op!(Div, div, i16);

// Instantiate for i32
impl_scalar_tensor_op!(Add, add, i32);
impl_scalar_tensor_op!(Sub, sub, i32);
impl_scalar_tensor_op!(Mul, mul, i32);
impl_scalar_tensor_op!(Div, div, i32);

// Instantiate for i64
impl_scalar_tensor_op!(Add, add, i64);
impl_scalar_tensor_op!(Sub, sub, i64);
impl_scalar_tensor_op!(Mul, mul, i64);
impl_scalar_tensor_op!(Div, div, i64);

// Instantiate for i128
impl_scalar_tensor_op!(Add, add, i128);
impl_scalar_tensor_op!(Sub, sub, i128);
impl_scalar_tensor_op!(Mul, mul, i128);
impl_scalar_tensor_op!(Div, div, i128);

// Instantiate for u8
impl_scalar_tensor_op!(Add, add, u8);
impl_scalar_tensor_op!(Sub, sub, u8);
impl_scalar_tensor_op!(Mul, mul, u8);
impl_scalar_tensor_op!(Div, div, u8);

// Instantiate for u16
impl_scalar_tensor_op!(Add, add, u16);
impl_scalar_tensor_op!(Sub, sub, u16);
impl_scalar_tensor_op!(Mul, mul, u16);
impl_scalar_tensor_op!(Div, div, u16);

// Instantiate for u32
impl_scalar_tensor_op!(Add, add, u32);
impl_scalar_tensor_op!(Sub, sub, u32);
impl_scalar_tensor_op!(Mul, mul, u32);
impl_scalar_tensor_op!(Div, div, u32);

// Instantiate for u64
impl_scalar_tensor_op!(Add, add, u64);
impl_scalar_tensor_op!(Sub, sub, u64);
impl_scalar_tensor_op!(Mul, mul, u64);
impl_scalar_tensor_op!(Div, div, u64);

// Instantiate for u128
impl_scalar_tensor_op!(Add, add, u128);
impl_scalar_tensor_op!(Sub, sub, u128);
impl_scalar_tensor_op!(Mul, mul, u128);
impl_scalar_tensor_op!(Div, div, u128);
