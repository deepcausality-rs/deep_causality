/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensorError;

mod api;
mod display;
mod from;
mod getters;
mod op_scalar_tensor;
pub mod op_tensor_broadcast;
pub mod op_tensor_ein_sum;
mod op_tensor_product;
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

/// `CausalTensor` is a low-dimensional (up to ~5-25 dimensions recommended) tensor
/// backed by a single, contiguous `Vec<T>`. It uses a stride-based memory layout
/// for efficient, cache-friendly access and manipulation.
///
/// **Operation Semantics:**
/// Operations on `CausalTensor` generally fall into two categories regarding data handling:
///
/// 1.  **Metadata-Only Operations (e.g., `reshape`, `permute_axes`, `ravel`):**
///     These operations create a new `CausalTensor` instance but do *not* physically reorder
///     or reallocate the underlying data in memory. Instead, they create a *cloned* copy of the
///     original flat data and only modify the `shape` and `strides` metadata to provide a new
///     logical view of the data. This makes them very efficient as they avoid large data movements.
///
/// 2.  **Data-Copying Operations (e.g., `slice`, binary operations like `add`, `sub`, `mul`, `div`,
///     reduction operations like `sum_axes`, `mean_axes`):**
///     These operations create a new `CausalTensor` instance with newly allocated data.
///     The data is either a subset of the original, a transformation of the original, or a result
///     of combining multiple tensors. These operations involve iterating through and copying/computing
///     values into a new `Vec<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalTensor<T> {
    pub(crate) data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
}

impl<T> CausalTensor<T> {
    ///
    /// This constructor validates that the total number of elements implied by the `shape`
    /// matches the length of the provided `data` vector. It also internally calculates
    /// the strides for efficient memory access based on the given `shape`.
    ///
    /// # Arguments
    ///
    /// * `data` - A `Vec<T>` containing the tensor's elements in row-major order.
    /// * `shape` - A `Vec<usize>` defining the dimensions of the tensor.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(Self)` if the `data` length matches the `shape`'s total elements.
    /// * `Err(CausalTensorError::ShapeMismatch)` if the lengths do not match.
    pub fn new(data: Vec<T>, shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let expected_len: usize = shape.iter().product();
        if data.len() != expected_len {
            return Err(CausalTensorError::ShapeMismatch);
        }

        // Calculate strides internally, which is the core safety improvement.
        let mut strides = vec![0; shape.len()];
        if !shape.is_empty() {
            let mut current_stride = 1;
            // Iterate from the last dimension to the first
            for i in (0..shape.len()).rev() {
                strides[i] = current_stride;
                current_stride *= shape[i];
            }
        }

        Ok(Self {
            data,
            shape,
            strides,
        })
    }

    /// Internal constructor that calculates strides.
    /// Call only when shape is known to be valid.
    pub(super) fn from_vec_and_shape_unchecked(data: Vec<T>, shape: &[usize]) -> Self {
        let mut strides = vec![0; shape.len()];
        if !shape.is_empty() {
            let mut current_stride = 1;
            for i in (0..shape.len()).rev() {
                strides[i] = current_stride;
                current_stride *= shape[i];
            }
        }
        Self {
            data,
            shape: shape.to_vec(),
            strides,
        }
    }
}
