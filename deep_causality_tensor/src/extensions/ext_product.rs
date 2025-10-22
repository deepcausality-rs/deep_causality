/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalTensor;
use crate::CausalTensorError;

/// An extension trait for computing the tensor product of two tensors.
pub trait CausalTensorProductExt<T> {
    /// Computes the tensor product (outer product) of `self` and `rhs`.
    ///
    /// The tensor product of two tensors `A` and `B` results in a new tensor
    /// whose shape is the concatenation of the shapes of `A` and `B`, and
    /// whose elements are the products of all combinations of elements from `A` and `B`.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand side `CausalTensor` to multiply with.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(CausalTensor<T>)` - The resulting tensor product.
    /// * `Err(CausalTensorError)` - An error if the operation fails (e.g., due to empty tensors).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::{CausalTensor, CausalTensorProductExt};
    ///
    /// let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    /// let b = CausalTensor::new(vec![3, 4], vec![2]).unwrap();
    /// // Tensor product of a (shape [2]) and b (shape [2]) results in a tensor of shape [2, 2]
    /// // [[1*3, 1*4],
    /// //  [2*3, 2*4]]
    /// let product = a.tensor_product(&b).unwrap();
    /// assert_eq!(product.shape(), &[2, 2]);
    /// assert_eq!(product.as_slice(), &[3, 4, 6, 8]);
    ///
    /// let c = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    /// let d = CausalTensor::new(vec![4, 5], vec![2]).unwrap();
    /// // Tensor product of c (shape [3]) and d (shape [2]) results in a tensor of shape [3, 2]
    /// // [[1*4, 1*5],
    /// //  [2*4, 2*5],
    /// //  [3*4, 3*5]]
    /// let product_cd = c.tensor_product(&d).unwrap();
    /// assert_eq!(product_cd.shape(), &[3, 2]);
    /// assert_eq!(product_cd.as_slice(), &[4, 5, 8, 10, 12, 15]);
    /// ```
    fn tensor_product(&self, rhs: &CausalTensor<T>) -> Result<CausalTensor<T>, CausalTensorError>;
}
