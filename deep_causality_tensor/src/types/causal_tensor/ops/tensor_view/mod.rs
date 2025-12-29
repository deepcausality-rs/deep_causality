/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};
use core::ops::Range;

impl<T> CausalTensor<T>
where
    T: Clone,
{
    pub(in crate::types::causal_tensor) fn slice_impl(
        &self,
        axis: usize,
        index: usize,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        // --- 1. Input Validation ---
        if axis >= self.num_dim() {
            return Err(CausalTensorError::AxisOutOfBounds);
        }
        if index >= self.shape()[axis] {
            // Using a specific error for this would be even better, but this is clear enough.
            return Err(CausalTensorError::AxisOutOfBounds);
        }

        // --- 2. Calculate the shape of the new, sliced tensor ---
        let mut new_shape: Vec<usize> = self.shape().to_vec();
        new_shape.remove(axis);

        let new_len: usize = new_shape.iter().product();
        let mut new_data = Vec::with_capacity(new_len);

        // --- 3. Iterate and copy the relevant data ---
        // This is a robust, safe way to iterate through all elements of the slice.
        let mut current_index = vec![0; self.num_dim()];
        for _ in 0..self.len() {
            // If the current multi-dimensional index is on the desired slice...
            if current_index[axis] == index {
                // ...get the value at that position and push it to our new data buffer.
                // .unwrap() is safe because we are iterating within the bounds of self.len().
                let flat_index = self.get_flat_index(&current_index).unwrap();
                new_data.push(self.as_slice()[flat_index].clone());
            }

            // Increment the multi-dimensional index to the next position.
            for j in (0..self.num_dim()).rev() {
                current_index[j] += 1;
                if current_index[j] < self.shape()[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        // --- 4. Construct and return the new tensor ---
        CausalTensor::new(new_data, new_shape)
    }

    pub(in crate::types::causal_tensor) fn shifted_view_impl(&self, flat_index: usize) -> Self
    where
        T: Clone,
    {
        let len = self.data.len();
        let shift = flat_index % len; // Safety check

        // 1. Clone the data (Since we are currently owning)
        let mut new_data = self.data.clone();

        // 2. Rotate the vector
        // Rust's native rotate_left handles the wrapping logic efficiently.
        // [A, B, C, D].rotate_left(1) -> [B, C, D, A]
        // Now index 0 holds 'B', which was previously at index 1.
        new_data.rotate_left(shift);

        // 3. Return new Tensor (Shape and Strides remain identical)
        Self {
            data: new_data,
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }

    /// Extracts a sub-tensor using range-based indexing for each dimension.
    ///
    /// This is a data-copying operation that creates a new tensor containing
    /// elements from the specified ranges along each dimension.
    ///
    /// # Arguments
    ///
    /// * `ranges` - A slice of `Range<usize>`, one per dimension. Each range
    ///   specifies `start..end` indices for that dimension.
    ///
    /// # Returns
    ///
    /// A new `CausalTensor` with shape determined by the range sizes.
    ///
    /// # Errors
    ///
    /// - `DimensionMismatch` if `ranges.len() != self.num_dim()`
    /// - `IndexOutOfBounds` if any range exceeds the dimension bounds
    pub fn range_slice_impl(
        &self,
        ranges: &[Range<usize>],
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        let shape = self.shape();
        let ndim = self.num_dim();

        // --- 1. Input Validation ---
        if ranges.len() != ndim {
            return Err(CausalTensorError::DimensionMismatch);
        }

        for (dim, range) in ranges.iter().enumerate() {
            if range.start > range.end {
                return Err(CausalTensorError::IndexOutOfBounds);
            }
            if range.end > shape[dim] {
                return Err(CausalTensorError::IndexOutOfBounds);
            }
        }

        // --- 2. Calculate the new shape ---
        let new_shape: Vec<usize> = ranges.iter().map(|r| r.end - r.start).collect();
        let new_len: usize = new_shape.iter().product();

        if new_len == 0 {
            // Handle empty slice case
            return CausalTensor::new(vec![], new_shape);
        }

        // --- 3. Pre-allocate output buffer ---
        let mut new_data = Vec::with_capacity(new_len);

        // --- 4. Iterate through all positions in the output tensor ---
        // We iterate in row-major order through the output shape,
        // mapping each output index back to the source tensor.
        let mut output_indices = vec![0usize; ndim];

        for _ in 0..new_len {
            // Map output indices to source indices by adding range starts
            let source_indices: Vec<usize> = output_indices
                .iter()
                .zip(ranges.iter())
                .map(|(&out_idx, range)| range.start + out_idx)
                .collect();

            // Get value from source using existing infrastructure
            if let Some(val) = self.get(&source_indices) {
                new_data.push(val.clone());
            } else {
                // This should not happen if validation is correct
                return Err(CausalTensorError::IndexOutOfBounds);
            }

            // Increment output indices in row-major order
            for dim in (0..ndim).rev() {
                output_indices[dim] += 1;
                if output_indices[dim] < new_shape[dim] {
                    break;
                }
                output_indices[dim] = 0;
            }
        }

        // --- 5. Construct and return the new tensor ---
        CausalTensor::new(new_data, new_shape)
    }
}
