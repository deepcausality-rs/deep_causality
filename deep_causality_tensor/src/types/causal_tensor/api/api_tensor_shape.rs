/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

impl<T> CausalTensor<T>
where
    T: Clone,
{
    /// Returns a new tensor with the same data but a different shape.
    ///
    /// This is a metadata-only operation; it creates a new `CausalTensor` with a cloned copy
    /// of the original flat data. The underlying data is *not* physically reordered or reallocated.
    /// Only the `shape` and `strides` are recomputed to reflect the new logical view.
    /// The total number of elements implied by the `new_shape` must be equal to the total number of
    /// elements in the original tensor (`self.len()`).
    ///
    /// # Arguments
    ///
    /// * `new_shape` - A slice representing the desired new shape.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Self)`: A new `CausalTensor` with the updated shape.
    /// - `Err(CausalTensorError)`: If the `new_shape` is incompatible (e.g., total elements don't match).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    ///
    /// // Reshape to 3x2
    /// let reshaped = tensor.reshape(&[3, 2]).unwrap();
    /// assert_eq!(reshaped.shape(), &[3, 2]);
    /// assert_eq!(reshaped.as_slice(), &[1, 2, 3, 4, 5, 6]); // Data remains the same
    ///
    /// // Reshape to 1D vector
    /// let raveled = tensor.reshape(&[6]).unwrap();
    /// assert_eq!(raveled.shape(), &[6]);
    ///
    /// // Incompatible shape (total elements don't match)
    /// let err = tensor.reshape(&[2, 2]);
    /// assert!(err.is_err());
    /// ```
    pub fn reshape(&self, new_shape: &[usize]) -> Result<Self, CausalTensorError> {
        self.reshape_impl(new_shape)
    }

    /// Flattens the tensor into a 1-dimensional tensor (vector).
    ///
    /// This is a metadata-only operation; it does not copy or reallocate the underlying data.
    /// The resulting tensor will have a shape of `[self.len()]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// let raveled_tensor = tensor.ravel();
    /// assert_eq!(raveled_tensor.shape(), &[6]);
    /// assert_eq!(raveled_tensor.as_slice(), &[1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn ravel(self) -> Self {
        self.ravel_impl()
    }
}
