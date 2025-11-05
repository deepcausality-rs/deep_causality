/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};
use std::ops::{Add, Div};

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd,
{
    /// Sums the elements along one or more specified axes.
    ///
    /// The dimensions corresponding to the `axes` provided will be removed from the
    /// shape of the resulting tensor. If `axes` is empty, the sum of all elements
    /// in the tensor is returned as a 0-dimensional (scalar) tensor.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Must implement `Add<T, Output = T>` for summation.
    ///
    /// # Arguments
    ///
    /// * `axes` - A slice of `usize` indicating the axes along which to sum. Axes are 0-indexed.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Self)`: A new `CausalTensor` containing the sums.
    /// - `Err(CausalTensorError)`: If an invalid axis is specified.
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
    /// // Sum along axis 0 (columns): [1+4, 2+5, 3+6] = [5, 7, 9]
    /// let sum_axis0 = tensor.sum_axes(&[0]).unwrap();
    /// assert_eq!(sum_axis0.shape(), &[3]);
    /// assert_eq!(sum_axis0.as_slice(), &[5, 7, 9]);
    ///
    /// // Sum along axis 1 (rows): [1+2+3, 4+5+6] = [6, 15]
    /// let sum_axis1 = tensor.sum_axes(&[1]).unwrap();
    /// assert_eq!(sum_axis1.shape(), &[2]);
    /// assert_eq!(sum_axis1.as_slice(), &[6, 15]);
    ///
    /// // Sum all elements: 1+2+3+4+5+6 = 21
    /// let sum_all = tensor.sum_axes(&[]).unwrap();
    /// assert_eq!(sum_all.shape(), &[]); // Scalar result with shape [] still has one element.
    /// assert_eq!(sum_all.as_slice(), &[21]);
    ///
    /// // Invalid axis
    /// let err = tensor.sum_axes(&[2]);
    /// assert!(err.is_err());
    /// ```
    pub fn sum_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Add<T, Output = T>,
    {
        self.sum_axes_impl(axes)
    }

    /// Calculates the mean (average) of the elements along one or more specified axes.
    ///
    /// The dimensions corresponding to the `axes` provided will be removed from the
    /// shape of the resulting tensor. If `axes` is empty, the mean of all elements
    /// in the tensor is returned as a 0-dimensional (scalar) tensor.
    ///
    /// # Type Parameters
    ///
    /// * `T`: Must implement `Div<T, Output = T>` for division.
    /// * `T`: Must implement `From<u32>` to convert counts to the numeric type
    /// * `T`: Must implement `Add<T, Output = T>` for summation.
    ///
    /// # Arguments
    ///
    /// * `axes` - A slice of `usize` indicating the axes along which to calculate the mean. Axes are 0-indexed.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Self)`: A new `CausalTensor` containing the means.
    /// - `Err(CausalTensorError)`: If an invalid axis is specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    /// // Tensor:
    /// // [[1.0, 2.0, 3.0],
    /// //  [4.0, 5.0, 6.0]]
    ///
    /// // Mean along axis 0 (columns): [(1+4)/2, (2+5)/2, (3+6)/2] = [2.5, 3.5, 4.5]
    /// let mean_axis0 = tensor.mean_axes(&[0]).unwrap();
    /// assert_eq!(mean_axis0.shape(), &[3]);
    /// assert_eq!(mean_axis0.as_slice(), &[2.5, 3.5, 4.5]);
    ///
    /// // Mean along axis 1 (rows): [(1+2+3)/3, (4+5+6)/3] = [2.0, 5.0]
    /// let mean_axis1 = tensor.mean_axes(&[1]).unwrap();
    /// assert_eq!(mean_axis1.shape(), &[2]);
    /// assert_eq!(mean_axis1.as_slice(), &[2.0, 5.0]);
    ///
    /// // Mean of all elements: (1+2+3+4+5+6)/6 = 3.5
    /// let mean_all = tensor.mean_axes(&[]).unwrap();
    /// assert_eq!(mean_all.shape(), &[]); // Scalar result
    /// assert_eq!(mean_all.as_slice(), &[3.5]);
    /// ```
    pub fn mean_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Div<T, Output = T> + From<u32> + Add<T, Output = T>,
    {
        self.mean_axes_impl(axes)
    }

    /// Computes the indices that would sort a 1-dimensional tensor (vector).
    ///
    /// This method is only valid for tensors with `ndim() == 1`. It returns a vector
    /// of indices such that applying these indices to the original tensor would yield
    /// a sorted version of the tensor. The sorting is stable.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(Vec<usize>)`: A vector of indices that sort the tensor.
    /// - `Err(CausalTensorError)`: If the tensor is not 1-dimensional.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![3, 1, 4, 1, 5, 9, 2, 6], vec![8]).unwrap();
    /// let sorted_indices = tensor.arg_sort().unwrap();
    /// assert_eq!(sorted_indices, vec![1, 3, 6, 0, 2, 4, 7, 5]);
    ///
    /// // Verify sorting
    /// let sorted_data: Vec<_> = sorted_indices.iter().map(|&i| tensor.as_slice()[i]).collect();
    /// assert_eq!(sorted_data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    ///
    /// // Attempting to sort a 2D tensor will result in an error
    /// let multi_dim_tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    /// assert!(multi_dim_tensor.arg_sort().is_err());
    /// ```
    pub fn arg_sort(&self) -> Result<Vec<usize>, CausalTensorError> {
        self.arg_sort_impl()
    }
}
