/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Tensor;
use crate::types::cpu_tensor::{EinSumAST, EinSumOp};
use crate::{CausalTensorError, EinSumValidationError, InternalCpuTensor};

impl<T> InternalCpuTensor<T>
where
    T: Clone,
{
    pub(crate) fn get_binary_operands(
        children: &[EinSumAST<InternalCpuTensor<T>>],
    ) -> Result<(InternalCpuTensor<T>, InternalCpuTensor<T>), CausalTensorError> {
        if children.len() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren {
                    expected: 2,
                    found: children.len(),
                },
            ));
        }

        let lhs = match children[0].value() {
            EinSumOp::TensorSource { tensor } => tensor.clone(),
            _ => {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::InvalidASTStructure {
                        message: "Expected TensorSource node for binary operand".to_string(),
                    },
                ));
            }
        };

        let rhs = match children[1].value() {
            EinSumOp::TensorSource { tensor } => tensor.clone(),
            _ => {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::InvalidASTStructure {
                        message: "Expected TensorSource node for binary operand".to_string(),
                    },
                ));
            }
        };

        Ok((lhs, rhs))
    }

    pub(crate) fn get_unary_operand(
        children: &[EinSumAST<InternalCpuTensor<T>>],
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if children.len() != 1 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren {
                    expected: 1,
                    found: children.len(),
                },
            ));
        }

        match children[0].value() {
            EinSumOp::TensorSource { tensor } => Ok(tensor.clone()),
            _ => Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidASTStructure {
                    message: "Expected TensorSource node for unary operand".to_string(),
                },
            )),
        }
    }
}

use crate::TensorData;

// Specialized constructions
impl<T> InternalCpuTensor<T>
where
    T: TensorData,
{
    pub(crate) fn mat_mul_2d(
        lhs: &InternalCpuTensor<T>,
        rhs: &InternalCpuTensor<T>,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if lhs.ndim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 2,
                    found: lhs.ndim(),
                },
            ));
        }
        if rhs.ndim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 2,
                    found: rhs.ndim(),
                },
            ));
        }
        if lhs.shape[1] != rhs.shape[0] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Matrix dimensions mismatch for multiplication: {} vs {}",
                        lhs.shape[1], rhs.shape[0]
                    ),
                },
            ));
        }

        let m = lhs.shape[0];
        let k = lhs.shape[1]; // Also rhs.shape[0]
        let n = rhs.shape[1];

        let mut result_data = vec![T::default(); m * n];

        let lhs_row_stride_i = lhs.strides[0];
        let lhs_col_stride = lhs.strides[1];
        let rhs_row_stride = rhs.strides[0];
        let rhs_col_stride_j = rhs.strides[1];

        for i in 0..m {
            let lhs_row_base = i * lhs_row_stride_i;
            for j in 0..n {
                let rhs_col_base = j * rhs_col_stride_j;
                let mut sum = T::default();
                for l in 0..k {
                    let lhs_idx = lhs_row_base + l * lhs_col_stride;
                    let rhs_idx = l * rhs_row_stride + rhs_col_base;
                    let lhs_val = lhs.data[lhs_idx];
                    let rhs_val = rhs.data[rhs_idx];
                    sum = sum + lhs_val * rhs_val;
                }
                result_data[i * n + j] = sum;
            }
        }

        InternalCpuTensor::new(result_data, vec![m, n])
    }

    /// Performs tensor contraction over specified axes.
    ///
    /// Contracts `lhs` and `rhs` along the specified axes, summing over the
    /// contracted dimensions. This is a generalization of matrix multiplication.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left-hand side tensor.
    /// * `rhs` - The right-hand side tensor.
    /// * `lhs_axes` - The axes of `lhs` to contract over.
    /// * `rhs_axes` - The axes of `rhs` to contract over.
    ///
    /// # Returns
    ///
    /// The result of contracting the tensors along the specified axes.
    ///
    /// # Errors
    ///
    /// Returns an error if the axes are invalid or dimensions don't match.
    pub(crate) fn contract(
        lhs: &InternalCpuTensor<T>,
        rhs: &InternalCpuTensor<T>,
        lhs_axes: &[usize],
        rhs_axes: &[usize],
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        // Validate axes lengths match
        if lhs_axes.len() != rhs_axes.len() {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: format!(
                        "Contraction axes length mismatch: lhs has {} axes, rhs has {}",
                        lhs_axes.len(),
                        rhs_axes.len()
                    ),
                },
            ));
        }

        // Validate axes are in bounds
        for &ax in lhs_axes {
            if ax >= lhs.ndim() {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::InvalidAxesSpecification {
                        message: format!("LHS axis {} out of bounds (ndim={})", ax, lhs.ndim()),
                    },
                ));
            }
        }
        for &ax in rhs_axes {
            if ax >= rhs.ndim() {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::InvalidAxesSpecification {
                        message: format!("RHS axis {} out of bounds (ndim={})", ax, rhs.ndim()),
                    },
                ));
            }
        }

        // Validate contracted dimensions match
        for (&lhs_ax, &rhs_ax) in lhs_axes.iter().zip(rhs_axes.iter()) {
            if lhs.shape[lhs_ax] != rhs.shape[rhs_ax] {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::ShapeMismatch {
                        message: format!(
                            "Contraction dimension mismatch: lhs axis {} (dim {}) vs rhs axis {} (dim {})",
                            lhs_ax, lhs.shape[lhs_ax], rhs_ax, rhs.shape[rhs_ax]
                        ),
                    },
                ));
            }
        }

        // Compute result shape: non-contracted axes of lhs + non-contracted axes of rhs
        let lhs_free: Vec<usize> = (0..lhs.ndim())
            .filter(|ax| !lhs_axes.contains(ax))
            .collect();
        let rhs_free: Vec<usize> = (0..rhs.ndim())
            .filter(|ax| !rhs_axes.contains(ax))
            .collect();

        let mut result_shape: Vec<usize> = lhs_free.iter().map(|&ax| lhs.shape[ax]).collect();
        result_shape.extend(rhs_free.iter().map(|&ax| rhs.shape[ax]));

        // Handle scalar result case
        if result_shape.is_empty() {
            result_shape.push(1); // Scalar represented as 1-element tensor
        }

        let result_len: usize = result_shape.iter().product();
        let mut result_data = vec![T::default(); result_len];

        // Contracted dimension product
        let contract_size: usize = lhs_axes.iter().map(|&ax| lhs.shape[ax]).product();

        // Iterate over all combinations of free indices
        let lhs_free_sizes: Vec<usize> = lhs_free.iter().map(|&ax| lhs.shape[ax]).collect();
        let rhs_free_sizes: Vec<usize> = rhs_free.iter().map(|&ax| rhs.shape[ax]).collect();

        let lhs_free_count: usize = lhs_free_sizes.iter().product::<usize>().max(1);
        let rhs_free_count: usize = rhs_free_sizes.iter().product::<usize>().max(1);

        for lhs_free_idx in 0..lhs_free_count {
            for rhs_free_idx in 0..rhs_free_count {
                let mut sum = T::default();

                for contract_idx in 0..contract_size {
                    // Build lhs index
                    let mut lhs_index = vec![0; lhs.ndim()];
                    let mut lhs_free_remaining = lhs_free_idx;
                    for (i, &ax) in lhs_free.iter().enumerate() {
                        let dim = lhs_free_sizes[i];
                        if dim > 0 {
                            lhs_index[ax] = lhs_free_remaining % dim;
                            lhs_free_remaining /= dim;
                        }
                    }
                    let mut contract_remaining = contract_idx;
                    for &ax in lhs_axes.iter() {
                        let dim = lhs.shape[ax];
                        if dim > 0 {
                            lhs_index[ax] = contract_remaining % dim;
                            contract_remaining /= dim;
                        }
                    }

                    // Build rhs index
                    let mut rhs_index = vec![0; rhs.ndim()];
                    let mut rhs_free_remaining = rhs_free_idx;
                    for (i, &ax) in rhs_free.iter().enumerate() {
                        let dim = rhs_free_sizes[i];
                        if dim > 0 {
                            rhs_index[ax] = rhs_free_remaining % dim;
                            rhs_free_remaining /= dim;
                        }
                    }
                    contract_remaining = contract_idx;
                    for &ax in rhs_axes.iter() {
                        let dim = rhs.shape[ax];
                        if dim > 0 {
                            rhs_index[ax] = contract_remaining % dim;
                            contract_remaining /= dim;
                        }
                    }

                    // Get values and accumulate - use Result for proper error handling
                    let lhs_val = lhs.get(&lhs_index).ok_or_else(|| {
                        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                            message: "Internal error: lhs index out of bounds in contraction".to_string(),
                        })
                    })?;
                    let rhs_val = rhs.get(&rhs_index).ok_or_else(|| {
                        CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                            message: "Internal error: rhs index out of bounds in contraction".to_string(),
                        })
                    })?;
                    sum = sum + *lhs_val * *rhs_val;
                }

                // Store result
                let result_idx = lhs_free_idx * rhs_free_count + rhs_free_idx;
                if result_idx < result_data.len() {
                    result_data[result_idx] = sum;
                }
            }
        }

        InternalCpuTensor::new(result_data, result_shape)
    }

    /// Performs element-wise multiplication (Hadamard product) between two `InternalCpuTensor`s.
    ///
    /// This private method multiplies corresponding elements of two tensors. It leverages
    /// the `broadcast_op` method to handle broadcasting rules if the tensor shapes are not identical.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left-hand side `InternalCpuTensor`.
    /// * `rhs` - The right-hand side `InternalCpuTensor`.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(InternalCpuTensor<T>)` containing the result of the element-wise multiplication.
    /// - `Err(CausalTensorError)` if broadcasting fails or an element-wise operation encounters an error.
    ///
    /// # Errors
    ///
    /// Returns errors propagated from `InternalCpuTensor::broadcast_op`.
    pub(in crate::types::cpu_tensor) fn element_wise_mul(
        lhs: &InternalCpuTensor<T>,
        rhs: &InternalCpuTensor<T>,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        lhs.broadcast_op(rhs, |a, b| Ok(a * b))
    }

    /// Computes the trace of a `InternalCpuTensor` over two specified axes.
    ///
    /// The trace operation sums the elements along the diagonal of a 2D slice
    /// defined by `axis1` and `axis2`. If the tensor has more than two dimensions,
    /// this operation is applied to all 2D slices formed by the specified axes,
    /// effectively reducing the tensor's rank by two.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `InternalCpuTensor`.
    /// * `axis1` - The first axis to trace over.
    /// * `axis2` - The second axis to trace over.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(InternalCpuTensor<T>)` containing the result of the trace operation.
    /// - `Err(CausalTensorError)` if validation fails.
    ///
    /// # Errors
    ///
    /// Returns `CausalTensorError::EinSumError` if:
    /// - `axis1` or `axis2` are out of bounds for the tensor's dimensions.
    /// - `axis1` and `axis2` are identical.
    /// - The dimensions of `axis1` and `axis2` are not equal (shape mismatch).
    pub(in crate::types::cpu_tensor) fn trace(
        tensor: &InternalCpuTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if axis1 >= tensor.ndim() || axis2 >= tensor.ndim() || axis1 == axis2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: format!("Invalid trace axes: {}, {}", axis1, axis2),
                },
            ));
        }
        if tensor.shape[axis1] != tensor.shape[axis2] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Trace axes have mismatched dimensions: axis {} (dim {}), axis {} (dim {})",
                        axis1, tensor.shape[axis1], axis2, tensor.shape[axis2]
                    ),
                },
            ));
        }

        let new_shape: Vec<usize> = tensor
            .shape
            .iter()
            .enumerate()
            .filter(|&(i, _)| i != axis1 && i != axis2)
            .map(|(_, &dim)| dim)
            .collect();

        if new_shape.is_empty() {
            // This is a 2D tensor trace, resulting in a scalar.
            let mut total_sum = T::default();
            let dim = tensor.shape[axis1];
            let stride1 = tensor.strides[axis1];
            let stride2 = tensor.strides[axis2];

            for i in 0..dim {
                let flat_index = i * stride1 + i * stride2;
                total_sum = total_sum + tensor.data[flat_index];
            }
            return InternalCpuTensor::new(vec![total_sum], vec![]);
        }

        let mut result_tensor = InternalCpuTensor::full(&new_shape, T::default());
        let diag_len = tensor.shape[axis1];

        let mut batch_axes = Vec::new();
        for i in 0..tensor.ndim() {
            if i != axis1 && i != axis2 {
                batch_axes.push(i);
            }
        }

        let num_batch_elements: usize = batch_axes.iter().map(|&ax| tensor.shape[ax]).product();
        let mut current_batch_indices = vec![0; batch_axes.len()];

        for _ in 0..num_batch_elements {
            let result_index = current_batch_indices.clone();
            if let Some(res_val) = result_tensor.get_mut(&result_index) {
                let mut batch_offset = 0;
                for (k, &batch_axis) in batch_axes.iter().enumerate() {
                    batch_offset += current_batch_indices[k] * tensor.strides[batch_axis];
                }

                let mut diag_sum = T::default();
                for i in 0..diag_len {
                    let flat_index =
                        batch_offset + i * tensor.strides[axis1] + i * tensor.strides[axis2];
                    diag_sum = diag_sum + tensor.data[flat_index];
                }
                *res_val = diag_sum;
            }

            // Increment batch indices
            let mut k = batch_axes.len();
            while k > 0 {
                k -= 1;
                current_batch_indices[k] += 1;
                if current_batch_indices[k] < tensor.shape[batch_axes[k]] {
                    break;
                }
                current_batch_indices[k] = 0;
            }
        }

        Ok(result_tensor)
    }

    /// Extracts the diagonal elements of a `InternalCpuTensor` over two specified axes.
    ///
    /// This private method extracts the diagonal elements from 2D slices of the input tensor
    /// defined by `axis1` and `axis2`. The resulting tensor will have a rank reduced by one
    /// compared to the input tensor, with the new last dimension representing the diagonal.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `InternalCpuTensor`.
    /// * `axis1` - The first axis defining the 2D plane from which to extract the diagonal.
    /// * `axis2` - The second axis defining the 2D plane from which to extract the diagonal.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(InternalCpuTensor<T>)` containing the tensor with extracted diagonal elements.
    /// - `Err(CausalTensorError)` if validation fails.
    ///
    /// # Errors
    ///
    /// Returns `CausalTensorError::EinSumError` if:
    /// - `axis1` or `axis2` are out of bounds for the tensor's dimensions.
    /// - `axis1` and `axis2` are identical.
    /// - The dimensions of `axis1` and `axis2` are not equal (shape mismatch).
    pub(in crate::types::cpu_tensor) fn diagonal(
        tensor: &InternalCpuTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if axis1 >= tensor.ndim() || axis2 >= tensor.ndim() || axis1 == axis2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: format!("Invalid diagonal axes: {}, {}", axis1, axis2),
                },
            ));
        }
        if tensor.shape[axis1] != tensor.shape[axis2] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Diagonal axes have mismatched dimensions: axis {} (dim {}), axis {} (dim {})",
                        axis1, tensor.shape[axis1], axis2, tensor.shape[axis2]
                    ),
                },
            ));
        }

        let diag_len = tensor.shape[axis1];
        let mut new_shape = Vec::new();
        let mut batch_axes = Vec::new();

        for i in 0..tensor.ndim() {
            if i != axis1 && i != axis2 {
                new_shape.push(tensor.shape[i]);
                batch_axes.push(i);
            }
        }
        new_shape.push(diag_len); // Add the diagonal dimension

        let mut result_data = Vec::with_capacity(new_shape.iter().product());
        let mut current_batch_indices = vec![0; batch_axes.len()];

        // Iterate over all combinations of batch indices
        let num_batch_elements: usize = batch_axes.iter().map(|&ax| tensor.shape[ax]).product();

        for _ in 0..num_batch_elements {
            for i in 0..diag_len {
                let mut current_full_index = vec![0; tensor.ndim()];
                current_full_index[axis1] = i;
                current_full_index[axis2] = i;

                // Fill in batch indices
                for (j, &batch_axis) in batch_axes.iter().enumerate() {
                    current_full_index[batch_axis] = current_batch_indices[j];
                }
                result_data.push(*tensor.get(&current_full_index).unwrap());
            }

            // Increment batch indices
            let mut k = batch_axes.len();
            while k > 0 {
                k -= 1;
                current_batch_indices[k] += 1;
                if current_batch_indices[k] < tensor.shape[batch_axes[k]] {
                    break;
                }
                current_batch_indices[k] = 0;
            }
        }

        InternalCpuTensor::new(result_data, new_shape)
    }

    /// Performs batch matrix multiplication between two `InternalCpuTensor`s.
    ///
    /// This method expects both input tensors to have at least 3 dimensions, where the first
    /// dimension represents the batch size. It performs a 2D matrix multiplication for each
    /// corresponding pair of matrices within the batch and stacks the results.
    ///
    /// # Arguments
    ///
    /// * `lhs` - The left-hand side `InternalCpuTensor` with a batch dimension.
    /// * `rhs` - The right-hand side `InternalCpuTensor` with a batch dimension.
    ///
    /// # Returns
    ///
    /// A `Result` which is:
    /// - `Ok(InternalCpuTensor<T>)` containing the result of the batch matrix multiplication.
    /// - `Err(CausalTensorError)` if validation fails or an underlying operation encounters an error.
    ///
    /// # Errors
    ///
    /// Returns `CausalTensorError::EinSumError` if:
    /// - Either `lhs` or `rhs` has fewer than 3 dimensions (rank mismatch).
    /// - The batch sizes of `lhs` and `rhs` do not match (shape mismatch).
    /// - Errors are propagated from `slice`, `mat_mul_2d`, or `stack`.
    pub(in crate::types::cpu_tensor) fn batch_mat_mul(
        lhs: InternalCpuTensor<T>,
        rhs: InternalCpuTensor<T>,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if lhs.ndim() < 3 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 3,
                    found: lhs.ndim(),
                },
            ));
        }
        if rhs.ndim() < 3 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 3,
                    found: rhs.ndim(),
                },
            ));
        }

        let batch_size = lhs.shape()[0];
        if batch_size != rhs.shape()[0] {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::ShapeMismatch {
                    message: format!(
                        "Batch dimensions mismatch: lhs batch {} vs rhs batch {}",
                        batch_size,
                        rhs.shape()[0]
                    ),
                },
            ));
        }

        let mut result_batches = Vec::with_capacity(batch_size);

        for i in 0..batch_size {
            let lhs_batch = lhs.slice(0, i)?;
            let rhs_batch = rhs.slice(0, i)?;
            let result_batch = Self::mat_mul_2d(&lhs_batch, &rhs_batch)?;
            result_batches.push(result_batch);
        }

        // Stack the results back into a single tensor
        InternalCpuTensor::stack_impl(&result_batches, 0)
    }
}
