/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

/// An extension trait for collections of tensors.
pub trait CausalTensorCollectionExt<T> {
    /// Stacks a sequence of tensors along a new axis to form a single, higher-dimensional tensor.
    ///
    /// This method takes a slice of `CausalTensor`s and combines them into a new `CausalTensor`
    /// with an additional dimension. The new dimension is inserted at the specified `axis`.
    ///
    /// # Arguments
    ///
    /// * `axis` - The axis along which to stack the tensors. This must be a valid axis
    ///   within the dimensions of the input tensors, or one greater than the
    ///   maximum existing axis to append a new dimension.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(CausalTensor<T>)` - The new stacked tensor if the operation was successful.
    /// * `Err(CausalTensorError)` - An error if the input tensors are empty, have mismatched shapes,
    ///   or if the specified `axis` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_data_structures::{CausalTensor, CausalTensorCollectionExt};
    /// let t1 = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    /// let t2 = CausalTensor::new(vec![3, 4], vec![2]).unwrap();
    /// let stacked_tensor = [t1, t2].stack(0).unwrap();
    /// assert_eq!(stacked_tensor.shape(), &[2, 2]);
    /// ```
    fn stack(&self, axis: usize) -> Result<CausalTensor<T>, CausalTensorError>;
}
