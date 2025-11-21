/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError, EinSumAST};
use std::ops::{Add, Div, Mul};

pub trait Tensor<T> {
    /// Public API for Einstein summation.
    ///
    /// This method serves as the entry point for performing Einstein summation operations
    /// on `CausalTensor`s. It takes an `EinSumAST` (Abstract Syntax Tree) as input,
    /// which defines the sequence of tensor operations to be executed.
    ///
    /// # Arguments
    ///
    /// * `ast` - A reference to the `EinSumAST` that describes the Einstein summation operation.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(CausalTensor<T>)` containing the result of the Einstein summation.
    /// - `Err(CausalTensorError)` if any error occurs during the execution of the AST.
    ///
    /// # Errors
    ///
    /// Returns errors propagated from `execute_ein_sum`.
    fn ein_sum(ast: &EinSumAST<T>) -> Result<CausalTensor<T>, CausalTensorError>
    where
        T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn tensor_product(&self, rhs: &CausalTensor<T>) -> Result<CausalTensor<T>, CausalTensorError>
    where
        T: Clone + Mul<Output = T>;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn sum_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd,
        T: Add<T, Output = T>,
        Self: Sized;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn mean_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone + Default + PartialOrd,
        T: Div<T, Output = T> + From<u32> + Add<T, Output = T>,
        Self: Sized;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn arg_sort(&self) -> Result<Vec<usize>, CausalTensorError>
    where
        T: Clone + Default + PartialOrd;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn reshape(&self, new_shape: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
        Self: Sized;
    /// Flattens the tensor into a 1-dimensional tensor (vector).
    ///
    /// This is a metadata-only operation; it does not copy or reallocate the underlying data.
    /// The resulting tensor will have a shape of `[self.len()]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_tensor::{CausalTensor, Tensor};
    ///
    /// let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    /// let raveled_tensor = tensor.ravel();
    /// assert_eq!(raveled_tensor.shape(), &[6]);
    /// assert_eq!(raveled_tensor.as_slice(), &[1, 2, 3, 4, 5, 6]);
    /// ```
    fn ravel(self) -> Self
    where
        T: Clone;
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
    /// use deep_causality_tensor::{CausalTensor, Tensor};
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
    fn slice(&self, axis: usize, index: usize) -> Result<CausalTensor<T>, CausalTensorError>
    where
        T: Clone;
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
    fn permute_axes(&self, axes: &[usize]) -> Result<Self, CausalTensorError>
    where
        T: Clone,
        Self: Sized;
    /// Creates a new tensor that is cyclically shifted (rolled) so that the
    /// element at `flat_index` moves to position 0.
    ///
    /// This is essential for Comonadic `extend` operations, allowing a local
    /// physics function to always treat the "current" pixel/particle as the
    /// origin (index 0) of the coordinate system.
    ///
    /// # Arguments
    /// * `flat_index` - The flat index in the data vector that should become the new origin.
    ///
    /// # Returns
    /// A new `CausalTensor` with the same shape, but rotated data.
    ///
    /// # Physics Note
    /// This implements Periodic Boundary Conditions (Topology of a Torus).
    /// If you shift off the edge, you wrap around to the other side.
    fn shifted_view(&self, flat_index: usize) -> Self
    where
        T: Clone;
}
