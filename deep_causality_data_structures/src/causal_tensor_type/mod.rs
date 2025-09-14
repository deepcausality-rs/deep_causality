/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensorError;

mod api;
pub mod errors;
pub mod extensions;
mod operations;

/// A multi-dimensional array designed for causal computation.
///
/// `CausalTensor` is a low-dimensional (up to ~5-25 dimensions recommended) tensor
/// backed by a single, contiguous `Vec<T>`. It uses a stride-based memory layout
/// for efficient, cache-friendly access and manipulation.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalTensor<T> {
    data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
}

impl<T> CausalTensor<T>
where
    T: Copy + Default + PartialOrd,
{
    /// Creates a new `CausalTensor` from a flat `Vec<T>` and a `shape` vector.
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
}

impl<T: std::fmt::Display> std::fmt::Display for CausalTensor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CausalTensor {{ data: [")?;
        for (i, item) in self.data.iter().enumerate() {
            write!(f, "{}", item)?;
            if i < self.data.len() - 1 {
                write!(f, ", ")?;
            }
        }
        write!(
            f,
            "], shape: {:?}, strides: {:?} }}",
            self.shape, self.strides
        )
    }
}
