/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor methods for BackendTensor.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};

impl<T, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a tensor from an owned vector.
    pub fn from_vec(data: Vec<T>, shape: &[usize]) -> Self {
        Self::from_inner(B::create_from_vec(data, shape))
    }
}

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a tensor from data with the given shape.
    ///
    /// # Arguments
    /// * `data` - Flat array of elements in row-major order
    /// * `shape` - Dimensions of the tensor
    ///
    /// # Panics
    /// Panics if `data.len() != shape.iter().product()`
    pub fn new(data: &[T], shape: &[usize]) -> Self {
        Self::from_inner(B::create(data, shape))
    }

    /// Creates a tensor from a function applied to each index.
    ///
    /// # Arguments
    /// * `shape` - Dimensions of the tensor
    /// * `f` - Function mapping indices to values
    pub fn from_shape_fn<F>(shape: &[usize], f: F) -> Self
    where
        F: FnMut(&[usize]) -> T,
    {
        Self::from_inner(B::from_shape_fn(shape, f))
    }
}

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a tensor filled with zeros.
    pub fn zeros(shape: &[usize]) -> Self {
        Self::from_inner(B::zeros(shape))
    }

    /// Creates a tensor filled with ones.
    pub fn ones(shape: &[usize]) -> Self {
        Self::from_inner(B::ones(shape))
    }
}
