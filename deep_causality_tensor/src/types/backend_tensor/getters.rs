/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for BackendTensor.

use super::BackendTensor;
use crate::traits::{TensorBackend, TensorData};

impl<T: TensorData, B: TensorBackend> BackendTensor<T, B> {
    /// Returns the shape of the tensor.
    pub fn shape(&self) -> Vec<usize> {
        B::shape(&self.inner)
    }

    /// Returns the number of dimensions.
    pub fn ndim(&self) -> usize {
        B::shape(&self.inner).len()
    }

    /// Returns the total number of elements.
    pub fn len(&self) -> usize {
        B::shape(&self.inner).iter().product()
    }

    /// Returns true if the tensor has no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Downloads tensor data to a host vector.
    pub fn to_vec(&self) -> Vec<T> {
        B::to_vec(&self.inner)
    }

    /// Returns a slice of the data (for CPU-backed tensors).
    ///
    /// Note: This creates a new Vec for non-CPU backends.
    pub fn as_slice(&self) -> Vec<T> {
        B::to_vec(&self.inner)
    }
}
