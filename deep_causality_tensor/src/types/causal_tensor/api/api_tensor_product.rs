/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};
use std::ops::Mul;

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd + Mul<Output = T>,
{
    /// Computes the tensor product (also known as the outer product) of two `CausalTensor`s.
    ///
    /// The tensor product combines two tensors into a new tensor whose rank is the sum of
    /// the ranks of the input tensors, and whose shape is the concatenation of their shapes.
    /// Each element of the resulting tensor is the product of an element from the left-hand side
    /// tensor and an element from the right-hand side tensor.
    ///
    /// This operation is fundamental in linear algebra and tensor calculus, effectively
    /// creating all possible pairwise products between elements of the input tensors.
    ///
    /// # Arguments
    ///
    /// * `rhs` - The right-hand side `CausalTensor`.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(CausalTensor<T>)` containing the result of the tensor product.
    /// - `Err(CausalTensorError)` if an error occurs during the operation (e.g., memory allocation).
    ///
    /// # Errors
    ///
    /// This method can return `CausalTensorError` if the underlying `tensor_product_impl`
    /// encounters an issue, such as a failure during new tensor creation.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let lhs = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap(); // Shape [2]
    /// let rhs = CausalTensor::new(vec![3.0, 4.0, 5.0], vec![3]).unwrap(); // Shape [3]
    ///
    /// // Expected result:
    /// // [[1*3, 1*4, 1*5],
    /// //  [2*3, 2*4, 2*5]]
    /// // which is [[3.0, 4.0, 5.0], [6.0, 8.0, 10.0]] with shape [2, 3]
    /// let result = lhs.tensor_product(&rhs).unwrap();
    ///
    /// assert_eq!(result.shape(), &[2, 3]);
    /// assert_eq!(result.as_slice(), &[3.0, 4.0, 5.0, 6.0, 8.0, 10.0]);
    ///
    /// // Tensor product with a scalar
    /// let scalar = CausalTensor::new(vec![10.0], vec![]).unwrap(); // Shape []
    /// let vector = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap(); // Shape [2]
    /// let result_scalar_vec = scalar.tensor_product(&vector).unwrap();
    /// assert_eq!(result_scalar_vec.shape(), &[2]);
    /// assert_eq!(result_scalar_vec.as_slice(), &[10.0, 20.0]);
    /// ```
    pub fn tensor_product(
        &self,
        rhs: &CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        self.tensor_product_impl(rhs)
    }
}
