/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd,
{
    /// Creates a new tensor representing a slice of the original tensor along a specified axis.
    ///
    /// This operation extracts a sub-tensor where one dimension has been fixed to a specific index.
    /// The rank (number of dimensions) of the resulting tensor will be one less than the original.
    ///
    /// **Note:** This is a data-copying operation. It creates a new `CausalTensor` with its
    /// own allocated data. A future optimization could provide a zero-copy, lifetime-bound view.
    ///
    /// # Arguments
    /// * `axis` - The axis to slice along (0-indexed).
    /// * `index` - The index at which to slice the axis.
    ///
    /// # Returns
    /// A `Result` containing the new, sliced `CausalTensor`, or a `CausalTensorError` if
    /// the axis or index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// // Tensor:
    /// // [[1, 2, 3],
    /// //  [4, 5, 6]]
    ///
    /// // Slice along axis 0 at index 0 (first row)
    /// let slice_row0 = tensor.slice(0, 0).unwrap();
    /// assert_eq!(slice_row0.shape(), &[3]);
    /// assert_eq!(slice_row0.as_slice(), &[1, 2, 3]);
    ///
    /// // Slice along axis 0 at index 1 (second row)
    /// let slice_row1 = tensor.slice(0, 1).unwrap();
    /// assert_eq!(slice_row1.shape(), &[3]);
    /// assert_eq!(slice_row1.as_slice(), &[4, 5, 6]);
    ///
    /// // Slice along axis 1 at index 1 (second column)
    /// let slice_col1 = tensor.slice(1, 1).unwrap();
    /// assert_eq!(slice_col1.shape(), &[2]);
    /// assert_eq!(slice_col1.as_slice(), &[2, 5]);
    /// ```
    pub fn slice(&self, axis: usize, index: usize) -> Result<CausalTensor<T>, CausalTensorError> {
        self.slice_impl(axis, index)
    }

    /// Permutes the axes of the tensor according to the given new order.
    ///
    /// This is a metadata-only operation; it creates a new `CausalTensor` with a cloned copy
    /// of the original flat data. The underlying data is *not* physically reordered or reallocated.
    /// Only the `shape` and `strides` are recomputed to reflect the new logical axis order.
    ///
    /// # Arguments
    ///
    /// * `axes` - A slice of `usize` representing the new order of axes.
    ///
    /// The length of the `axes` parameter must be equal to the number of dimensions of the tensor,
    /// and it must contain a permutation of `0..self.num_dim()`.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Self)`: A new `CausalTensor` with permuted axes.
    /// - `Err(CausalTensorError)`: If the `axes` are invalid (e.g., wrong length, not a permutation).
    pub fn permute_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError> {
        self.permute_axes_impl(axes)
    }
}
