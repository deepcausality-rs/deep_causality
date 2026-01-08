/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod algebra;
mod api;
mod arithmetic;
mod display;
mod from;
mod getters;
pub mod ops;

pub use ops::tensor_ein_sum::ein_sum_op::{EinSumAST, EinSumOp};

/// `CpuTensor` is a low-dimensional (up to ~5-25 dimensions recommended) tensor
/// backed by a single, contiguous `Vec<T>`. It uses a stride-based memory layout
/// for efficient, cache-friendly access and manipulation.
///
/// **Operation Semantics:**
/// Operations on `CpuTensor` generally fall into two categories regarding data handling:
///
/// 1.  **Metadata-Only Operations (e.g., `reshape`, `permute_axes`, `ravel`):**
///     These operations create a new `CpuTensor` instance but do *not* physically reorder
///     or reallocate the underlying data in memory. Instead, they create a *cloned* copy of the
///     original flat data and only modify the `shape` and `strides` metadata to provide a new
///     logical view of the data. This makes them very efficient as they avoid large data movements.
///
/// 2.  **Data-Copying Operations (e.g., `slice`, binary operations like `add`, `sub`, `mul`, `div`,
///     reduction operations like `sum_axes`, `mean_axes`):**
///     These operations create a new `CpuTensor` instance with newly allocated data.
///     The data is either a subset of the original, a transformation of the original, or a result
///     of combining multiple tensors. These operations involve iterating through and copying/computing
///     values into a new `Vec<T>`.
///
/// This serves as the default backend implementation.
#[derive(Clone, PartialEq, Debug)]
pub struct InternalCpuTensor<T> {
    pub(crate) data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
}

impl<T> InternalCpuTensor<T> {
    /// Consumes the tensor and returns the underlying data.
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Internal constructor that calculates strides.
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

    /// Creates a new `CpuTensor`.
    ///
    /// This constructor validates that the total number of elements implied by the `shape`
    /// matches the length of the provided `data` vector. It also internally calculates
    /// the strides for efficient memory access based on the given `shape`.
    pub fn new(data: Vec<T>, shape: Vec<usize>) -> Result<Self, crate::CausalTensorError> {
        let expected_len: usize = shape.iter().product();
        if data.len() != expected_len {
            return Err(crate::CausalTensorError::ShapeMismatch);
        }

        // Calculate strides internally.
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
}

impl<T> InternalCpuTensor<T> where T: Clone {}
