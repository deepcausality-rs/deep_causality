/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::impl_tensor_tensor_op;
use crate::{CausalTensor, CausalTensorError};
use std::ops::{Add, Div, Mul, Sub};

// Implement Tensor operations for Add, Sub, Mul, and Div
impl_tensor_tensor_op!(Add, add);
impl_tensor_tensor_op!(Sub, sub);
impl_tensor_tensor_op!(Mul, mul);
impl_tensor_tensor_op!(Div, div);
