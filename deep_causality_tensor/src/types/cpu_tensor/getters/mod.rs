/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::InternalCpuTensor;

impl<T> InternalCpuTensor<T> {
    /// Returns a reference to the underlying `Vec<T>` that stores the tensor's data.
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Returns a slice representing the shape (dimensions) of the tensor.
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    /// Returns a slice representing the strides of the tensor.
    pub fn strides(&self) -> &[usize] {
        &self.strides
    }

    /// Returns `true` if the tensor contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of dimensions (rank) of the tensor.
    /// Alias for `num_dim()` for API consistency with BackendTensor.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Returns the number of dimensions (rank) of the tensor.
    pub fn num_dim(&self) -> usize {
        self.shape.len()
    }

    /// Returns the total number of elements in the tensor.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice to the underlying contiguous data storage of the tensor.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns an optional reference to the element at the specified multi-dimensional index.
    pub fn get(&self, index: &[usize]) -> Option<&T> {
        let flat_index = self.get_flat_index(index)?;
        self.data.get(flat_index)
    }

    /// Returns an optional mutable reference to the element at the specified multi-dimensional index.
    pub fn get_mut(&mut self, index: &[usize]) -> Option<&mut T> {
        let flat_index = self.get_flat_index(index)?;
        self.data.get_mut(flat_index)
    }

    /// Calculates the flat index into the `data` vector from a multi-dimensional index.
    pub(crate) fn get_flat_index(&self, index: &[usize]) -> Option<usize> {
        if index.len() != self.ndim() {
            return None;
        }

        let mut flat_index = 0;
        for (i, &dim_index) in index.iter().enumerate() {
            if dim_index >= self.shape[i] {
                return None;
            }
            flat_index += dim_index * self.strides[i];
        }
        Some(flat_index)
    }
}
