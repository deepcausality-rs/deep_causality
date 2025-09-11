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
    /// Creates a new tensor from a flat vector and a logical shape.
    ///
    /// This is the primary, safe constructor. It takes ownership of the data and
    /// calculates the correct strides internally, guaranteeing that the tensor
    /// is always in a valid state.
    ///
    /// # Errors
    /// Returns `CausalTensorError::ShapeMismatch` if the number of elements in `data`
    /// does not match the product of the dimensions in `shape`.
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
