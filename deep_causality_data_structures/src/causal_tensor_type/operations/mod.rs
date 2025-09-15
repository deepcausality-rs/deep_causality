/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines various tensor operations, including arithmetic operations
//! with scalars, reductions, shape manipulations, and tensor-tensor operations.

/// Defines operations between a scalar and a tensor (e.g., `scalar + tensor`).
mod op_scalar_tensor;
/// Defines tensor reduction operations (e.g., sum, mean).
mod op_tensor_reduction;
/// Defines operations between a tensor and a scalar (e.g., `tensor + scalar`).
mod op_tensor_scalar;
/// Defines tensor shape manipulation operations (e.g., reshape, ravel).
mod op_tensor_shape;
/// Defines operations between two tensors (e.g., `tensor_a + tensor_b`).
mod op_tensor_tensor;
/// Defines tensor view operations (e.g., slicing).
mod op_view;
/// Utility functions for tensor operations.
pub(super) mod utils;
