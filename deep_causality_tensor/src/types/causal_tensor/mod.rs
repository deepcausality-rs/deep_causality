/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensorError;
use std::ops::{Add, Div};

mod op_scalar_tensor;
pub mod op_tensor_ein_sum;
mod op_tensor_product;
/// Defines tensor reduction operations (e.g., sum, mean).
mod op_tensor_reduction;
/// Defines operations between a tensor and a scalar (e.g., `tensor + scalar`).
mod op_tensor_scalar;
/// Defines tensor shape manipulation operations (e.g., reshape, ravel).
mod op_tensor_shape;
/// Defines operations between two tensors (e.g., `tensor_a + tensor_b`).
mod op_tensor_tensor;
/// Defines tensor view operations (e.g., slicing).
mod op_view;
/// Utility functions for tensor operations.
pub(crate) mod utils;

/// `CausalTensor` is a low-dimensional (up to ~5-25 dimensions recommended) tensor
/// backed by a single, contiguous `Vec<T>`. It uses a stride-based memory layout
/// for efficient, cache-friendly access and manipulation.
///
/// **Operation Semantics:**
/// Operations on `CausalTensor` generally fall into two categories regarding data handling:
///
/// 1.  **Metadata-Only Operations (e.g., `reshape`, `permute_axes`, `ravel`):**
///     These operations create a new `CausalTensor` instance but do *not* physically reorder
///     or reallocate the underlying data in memory. Instead, they create a *cloned* copy of the
///     original flat data and only modify the `shape` and `strides` metadata to provide a new
///     logical view of the data. This makes them very efficient as they avoid large data movements.
///
/// 2.  **Data-Copying Operations (e.g., `slice`, binary operations like `add`, `sub`, `mul`, `div`,
///     reduction operations like `sum_axes`, `mean_axes`):**
///     These operations create a new `CausalTensor` instance with newly allocated data.
///     The data is either a subset of the original, a transformation of the original, or a result
///     of combining multiple tensors. These operations involve iterating through and copying/computing
///     values into a new `Vec<T>`.
#[derive(Debug, Clone, PartialEq)]
pub struct CausalTensor<T> {
    pub(crate) data: Vec<T>,
    shape: Vec<usize>,
    strides: Vec<usize>,
}

impl<T> CausalTensor<T> {
    ///
    /// This constructor validates that the total number of elements implied by the `shape`
    /// matches the length of the provided `data` vector. It also internally calculates
    /// the strides for efficient memory access based on the given `shape`.
    ///
    /// # Arguments
    ///
    /// * `data` - A `Vec<T>` containing the tensor's elements in row-major order.
    /// * `shape` - A `Vec<usize>` defining the dimensions of the tensor.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// * `Ok(Self)` if the `data` length matches the `shape`'s total elements.
    /// * `Err(CausalTensorError::ShapeMismatch)` if the lengths do not match.
    pub fn new(data: Vec<T>, shape: Vec<usize>) -> Result<Self, CausalTensorError> {
        let expected_len: usize = shape.iter().product();
        if data.len() != expected_len {
            return Err(CausalTensorError::ShapeMismatch);
        }

        // Calculate strides internally, which is the core safety improvement.
        let mut strides = vec![0; shape.len()];
        if !shape.is_empty() {
            let mut current_stride = 1;
            // Iterate from the last dimension to the first
            for i in (0..shape.len()).rev() {
                strides[i] = current_stride;
                current_stride *= shape[i];
            }
        }

        Ok(Self {
            data,
            shape,
            strides,
        })
    }

    /// Returns a reference to the underlying `Vec<T>` that stores the tensor's data.
    ///
    /// The data is stored in row-major order.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// assert_eq!(tensor.data(), &vec![1, 2, 3, 4, 5, 6]);
    ///
    /// let empty_tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    /// assert!(empty_tensor.data().is_empty());
    /// ```
    pub fn data(&self) -> &Vec<T> {
        &self.data
    }

    /// Returns a slice representing the shape (dimensions) of the tensor.
    ///
    /// The elements of the slice indicate the size of each dimension.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// assert_eq!(tensor.shape(), &[2, 3]);
    /// ```
    pub fn shape(&self) -> &[usize] {
        &self.shape
    }

    // --- Inspectors ---

    /// Returns `true` if the tensor contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let empty_tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    /// assert!(empty_tensor.is_empty());
    ///
    /// let non_empty_tensor: CausalTensor<f64> = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    /// assert!(!non_empty_tensor.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the number of dimensions (rank) of the tensor.
    ///
    /// This is equivalent to `self.shape().len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// assert_eq!(tensor.num_dim(), 2);
    ///
    /// let scalar = CausalTensor::new(vec![42], vec![]).   unwrap();
    /// assert_eq!(scalar.num_dim(), 0); // A scalar has 0 dimensions
    /// ```
    pub fn num_dim(&self) -> usize {
        self.shape.len()
    }

    /// Returns the total number of elements in the tensor.
    ///
    /// This is equivalent to `self.as_slice().len()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// assert_eq!(tensor.len(), 6);
    ///
    /// let empty_tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![0]).unwrap();
    /// assert_eq!(empty_tensor.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns a slice to the underlying contiguous data storage of the tensor.
    ///
    /// The data is stored in row-major order.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// assert_eq!(tensor.as_slice(), &[1, 2, 3, 4, 5, 6]);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns an optional reference to the element at the specified multi-dimensional index.
    ///
    /// Returns `None` if the provided `index` is out of bounds or has an incorrect number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `index` - A slice representing the multi-dimensional coordinates (e.g., `&[row, col]`).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    ///
    /// assert_eq!(tensor.get(&[0, 0]), Some(&1));
    /// assert_eq!(tensor.get(&[0, 1]), Some(&2));
    /// assert_eq!(tensor.get(&[1, 2]), Some(&6));
    ///
    /// // Out of bounds
    /// assert_eq!(tensor.get(&[2, 0]), None);
    /// assert_eq!(tensor.get(&[0, 3]), None);
    ///
    /// // Incorrect number of dimensions
    /// assert_eq!(tensor.get(&[0]), None);
    /// assert_eq!(tensor.get(&[0, 0, 0]), None);
    /// ```
    pub fn get(&self, index: &[usize]) -> Option<&T> {
        let flat_index = self.get_flat_index(index)?;
        self.data.get(flat_index)
    }

    /// Returns an optional mutable reference to the element at the specified multi-dimensional index.
    ///
    /// Returns `None` if the provided `index` is out of bounds or has an incorrect number of dimensions.
    ///
    /// # Arguments
    ///
    /// * `index` - A slice representing the multi-dimensional coordinates (e.g., `&[row, col]`).
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// let mut tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    ///
    /// if let Some(val) = tensor.get_mut(&[0, 0]) {
    ///     *val = 10;
    /// }
    /// assert_eq!(tensor.as_slice(), &[10, 2, 3, 4, 5, 6]);
    ///
    /// if let Some(val) = tensor.get_mut(&[1, 2]) {
    ///     *val = 60;
    /// }
    /// assert_eq!(tensor.as_slice(), &[10, 2, 3, 4, 5, 60]);
    ///
    /// // Out of bounds
    /// assert_eq!(tensor.get_mut(&[2, 0]), None);
    /// ```
    pub fn get_mut(&mut self, index: &[usize]) -> Option<&mut T> {
        let flat_index = self.get_flat_index(index)?;
        self.data.get_mut(flat_index)
    }

    /// Calculates the flat index into the `data` vector from a multi-dimensional index.
    /// This is the core of the stride-based memory layout.
    pub(crate) fn get_flat_index(&self, index: &[usize]) -> Option<usize> {
        if index.len() != self.num_dim() {
            return None;
        }

        let mut flat_index = 0;
        for (i, &dim_index) in index.iter().enumerate() {
            if dim_index >= self.shape[i] {
                return None; // Index is out of bounds for this dimension
            }
            flat_index += dim_index * self.strides[i];
        }
        Some(flat_index)
    }
}

impl<T: std::fmt::Display> std::fmt::Display for CausalTensor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CausalTensor {{ data: [")?;
        let max_items = 10;
        for (i, item) in self.data.iter().take(max_items).enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        if self.data.len() > max_items {
            write!(f, ", ...")?;
        }
        write!(
            f,
            "], shape: {:?}, strides: {:?} }}",
            self.shape, self.strides
        )
    }
}

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd,
{
    // --- Shape Manipulation ---

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

    // --- Reduction Operations ---

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

    // --- View Operations ---

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
