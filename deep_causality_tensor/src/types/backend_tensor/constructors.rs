/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor methods for BackendTensor.

use super::BackendTensor;
use crate::CausalTensorError;
use crate::{TensorBackend, TensorData};

impl<T, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a tensor from an owned vector (infallible).
    pub fn from_vec(data: Vec<T>, shape: &[usize]) -> Self {
        Self::from_inner(B::create_from_vec(data, shape))
    }
}

impl<T: Clone, B: TensorBackend> BackendTensor<T, B> {
    /// Creates a new tensor from data with the given shape.
    ///
    /// This is the primary constructor for backward compatibility with CausalTensor.
    /// Returns a Result to match the legacy `InternalCpuTensor::new()` API.
    ///
    /// # Arguments
    /// * `data` - Flat vector of elements in row-major order
    /// * `shape` - Dimensions of the tensor
    ///
    /// # Errors
    /// Returns `CausalTensorError::ShapeMismatch` if `data.len() != shape.iter().product()`
    pub fn new(data: Vec<T>, shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let expected_len: usize = shape.iter().product();
        if data.len() != expected_len {
            return Err(CausalTensorError::ShapeMismatch);
        }
        Ok(Self::from_inner(B::create_from_vec(data, &shape)))
    }

    /// Creates a tensor from a slice (infallible, clones data).
    pub fn from_slice(data: &[T], shape: &[usize]) -> Self {
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

// CPU-backend-specific constructors that require algebraic traits
impl<T> BackendTensor<T, crate::CpuBackend>
where
    T: Clone + Copy + deep_causality_num::Ring,
{
    /// Creates a tensor of ones with the specified shape.
    pub fn one(shape: &[usize]) -> Self {
        Self::from_inner(crate::InternalCpuTensor::one(shape))
    }

    /// Creates an identity matrix (square tensor with 1s on diagonal).
    pub fn identity(shape: &[usize]) -> Result<Self, CausalTensorError> {
        crate::InternalCpuTensor::identity(shape).map(Self::from_inner)
    }
}
