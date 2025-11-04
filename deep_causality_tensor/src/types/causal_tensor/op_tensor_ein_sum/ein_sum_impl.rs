/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalTensor, CausalTensorError, CausalTensorStackExt, EinSumAST, EinSumValidationError,
};
use std::ops::{Add, Mul};

impl<T> CausalTensor<T>
where
    T: Clone + Default + PartialOrd + Add<Output = T> + Mul<Output = T>,
{
    /// Helper to get two operands from the AST children.
    pub(super) fn get_binary_operands(
        children: &[EinSumAST<T>],
    ) -> Result<(CausalTensor<T>, CausalTensor<T>), CausalTensorError> {
        if children.len() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren {
                    expected: 2,
                    found: children.len(),
                },
            ));
        }
        let lhs = CausalTensor::execute_ein_sum(&children[0])?;
        let rhs = CausalTensor::execute_ein_sum(&children[1])?;
        Ok((lhs, rhs))
    }

    /// Helper to get a single operand from the AST children.
    pub(super) fn get_unary_operand(
        children: &[EinSumAST<T>],
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if children.len() != 1 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidNumberOfChildren {
                    expected: 1,
                    found: children.len(),
                },
            ));
        }
        CausalTensor::execute_ein_sum(&children[0])
    }

    /// Private method for generic tensor contraction.
    /// This optimized version uses permutation and reshaping to reduce contraction to matrix multiplication.
    pub(super) fn contract(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
        lhs_contract_axes: &[usize],
        rhs_contract_axes: &[usize],
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        // 1. Validate input axes and shapes
        if lhs_contract_axes.len() != rhs_contract_axes.len() {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::InvalidAxesSpecification {
                    message: "Number of LHS and RHS contraction axes must match".to_string(),
                },
            ));
        }

        for (&lhs_axis, &rhs_axis) in lhs_contract_axes.iter().zip(rhs_contract_axes.iter()) {
            if lhs_axis >= lhs.num_dim() || rhs_axis >= rhs.num_dim() {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::InvalidAxesSpecification {
                        message: format!(
                            "Axis out of bounds: lhs_axis {} (dim {}), rhs_axis {} (dim {})",
                            lhs_axis,
                            lhs.num_dim(),
                            rhs_axis,
                            rhs.num_dim()
                        ),
                    },
                ));
            }
            if lhs.shape[lhs_axis] != rhs.shape[rhs_axis] {
                return Err(CausalTensorError::EinSumError(
                    EinSumValidationError::ShapeMismatch {
                        message: format!(
                            "Contracted axes have mismatched dimensions: lhs_axis {} (dim {}), rhs_axis {} (dim {})",
                            lhs_axis, lhs.shape[lhs_axis], rhs_axis, rhs.shape[rhs_axis]
                        ),
                    },
                ));
            }
        }

        // 2. Identify remaining (uncontracted) axes and build permutation maps
        let lhs_remaining_axes: Vec<usize> = (0..lhs.num_dim())
            .filter(|&i| !lhs_contract_axes.contains(&i))
            .collect();
        let rhs_remaining_axes: Vec<usize> = (0..rhs.num_dim())
            .filter(|&i| !rhs_contract_axes.contains(&i))
            .collect();

        // Create permutation for LHS: (remaining_lhs, contracted_lhs)
        let mut lhs_perm_order = lhs_remaining_axes.clone();
        lhs_perm_order.extend_from_slice(lhs_contract_axes);

        // Create permutation for RHS: (contracted_rhs, remaining_rhs)
        let mut rhs_perm_order = rhs_contract_axes.to_vec();
        rhs_perm_order.extend_from_slice(&rhs_remaining_axes);

        // 3. Permute and reshape tensors
        let permuted_lhs = lhs.permute_axes(&lhs_perm_order)?;
        let permuted_rhs = rhs.permute_axes(&rhs_perm_order)?;

        let contracted_dim_size: usize =
            lhs_contract_axes.iter().map(|&ax| lhs.shape[ax]).product();

        let lhs_rows: usize = lhs_remaining_axes.iter().map(|&ax| lhs.shape[ax]).product();
        let rhs_cols: usize = rhs_remaining_axes.iter().map(|&ax| rhs.shape[ax]).product();

        let reshaped_lhs = permuted_lhs.reshape(&[lhs_rows, contracted_dim_size])?;
        let reshaped_rhs = permuted_rhs.reshape(&[contracted_dim_size, rhs_cols])?;

        // 4. Perform matrix multiplication
        let result_matrix = Self::mat_mul_2d(&reshaped_lhs, &reshaped_rhs)?;

        // 5. Reshape result back to final tensor shape
        let mut final_shape = Vec::new();
        final_shape.extend(lhs_remaining_axes.iter().map(|&ax| lhs.shape[ax]));
        final_shape.extend(rhs_remaining_axes.iter().map(|&ax| rhs.shape[ax]));

        result_matrix.reshape(&final_shape)
    }

    /// Private helper for 2D matrix multiplication.
    pub(super) fn mat_mul_2d(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if lhs.num_dim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 2,
                    found: lhs.num_dim(),
                },
            ));
        }
        if rhs.num_dim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 2,
                    found: rhs.num_dim(),
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

        for i in 0..m {
            for j in 0..n {
                let mut sum = T::default();
                for l in 0..k {
                    let lhs_val = lhs.get(&[i, l]).unwrap().clone();
                    let rhs_val = rhs.get(&[l, j]).unwrap().clone();
                    sum = sum + lhs_val * rhs_val;
                }
                result_data[i * n + j] = sum;
            }
        }

        CausalTensor::new(result_data, vec![m, n])
    }

    /// Private method for element-wise multiplication.
    pub(super) fn element_wise_mul(
        lhs: &CausalTensor<T>,
        rhs: &CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        lhs.broadcast_op(rhs, |a, b| Ok(a * b))
    }

    /// Private method for tracing (summing over diagonal axes).
    pub(super) fn trace(
        tensor: &CausalTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if axis1 >= tensor.num_dim() || axis2 >= tensor.num_dim() || axis1 == axis2 {
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
            let mut total_sum = T::default();
            for i in 0..tensor.shape[axis1] {
                let mut index = vec![0; tensor.num_dim()];
                index[axis1] = i;
                index[axis2] = i;
                total_sum = total_sum + tensor.get(&index).unwrap().clone();
            }
            return CausalTensor::new(vec![total_sum], vec![]);
        }

        let mut result_tensor = CausalTensor::full(&new_shape, T::default());
        let mut current_index = vec![0; tensor.num_dim()];

        for i in 0..tensor.len() {
            if current_index[axis1] == current_index[axis2] {
                let result_index: Vec<usize> = current_index
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i != axis1 && i != axis2)
                    .map(|(_, &val)| val)
                    .collect();

                if let Some(res_val) = result_tensor.get_mut(&result_index) {
                    *res_val = res_val.clone() + tensor.data[i].clone();
                }
            }

            for j in (0..tensor.num_dim()).rev() {
                current_index[j] += 1;
                if current_index[j] < tensor.shape[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        Ok(result_tensor)
    }

    /// Private method for extracting a diagonal.
    pub(super) fn diagonal(
        tensor: &CausalTensor<T>,
        axis1: usize,
        axis2: usize,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if axis1 >= tensor.num_dim() || axis2 >= tensor.num_dim() || axis1 == axis2 {
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

        for i in 0..tensor.num_dim() {
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
                let mut current_full_index = vec![0; tensor.num_dim()];
                current_full_index[axis1] = i;
                current_full_index[axis2] = i;

                // Fill in batch indices
                for (j, &batch_axis) in batch_axes.iter().enumerate() {
                    current_full_index[batch_axis] = current_batch_indices[j];
                }
                result_data.push(tensor.get(&current_full_index).unwrap().clone());
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

        CausalTensor::new(result_data, new_shape)
    }

    pub(super) fn batch_mat_mul(
        lhs: CausalTensor<T>,
        rhs: CausalTensor<T>,
    ) -> Result<CausalTensor<T>, CausalTensorError> {
        if lhs.num_dim() < 3 || rhs.num_dim() < 3 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 3, // At least 3 dimensions for batch matmul (batch, rows, cols)
                    found: lhs.num_dim(),
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
        result_batches.stack(0)
    }
}

mod tests {
    #![allow(unused_imports)]

    use super::*;
    use crate::{EinSumOp, utils_tests};

    #[test]
    fn test_get_binary_operands_success() {
        let lhs_tensor = utils_tests::scalar_tensor(1.0);
        let rhs_tensor = utils_tests::scalar_tensor(2.0);
        let lhs_ast = EinSumOp::tensor_source(lhs_tensor.clone());
        let rhs_ast = EinSumOp::tensor_source(rhs_tensor.clone());
        let children = vec![lhs_ast, rhs_ast];

        let (res_lhs, res_rhs) = CausalTensor::get_binary_operands(&children).unwrap();
        assert_eq!(res_lhs, lhs_tensor);
        assert_eq!(res_rhs, rhs_tensor);
    }

    #[test]
    fn test_get_binary_operands_invalid_children_count() {
        let lhs_tensor = utils_tests::scalar_tensor(1.0);
        let lhs_ast = EinSumOp::tensor_source(lhs_tensor.clone());
        let children = vec![lhs_ast]; // Only one child

        let err = CausalTensor::get_binary_operands(&children).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidNumberOfChildren {
                expected: 2,
                found: 1
            })
        ));
    }

    #[test]
    fn test_get_unary_operand_success() {
        let operand_tensor = utils_tests::scalar_tensor(1.0);
        let operand_ast = EinSumOp::tensor_source(operand_tensor.clone());
        let children = vec![operand_ast];

        let res_operand = CausalTensor::get_unary_operand(&children).unwrap();
        assert_eq!(res_operand, operand_tensor);
    }

    #[test]
    fn test_get_unary_operand_invalid_children_count() {
        let lhs_tensor = utils_tests::scalar_tensor(1.0);
        let rhs_tensor = utils_tests::scalar_tensor(2.0);
        let lhs_ast = EinSumOp::tensor_source(lhs_tensor.clone());
        let rhs_ast = EinSumOp::tensor_source(rhs_tensor.clone());
        let children = vec![lhs_ast, rhs_ast]; // Two children

        let err = CausalTensor::get_unary_operand(&children).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidNumberOfChildren {
                expected: 1,
                found: 2
            })
        ));
    }

    #[test]
    fn test_mat_mul_2d_success() {
        let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
        let expected = utils_tests::matrix_tensor(vec![19.0, 22.0, 43.0, 50.0], 2, 2);

        let result = CausalTensor::mat_mul_2d(&lhs, &rhs).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mat_mul_2d_rank_mismatch() {
        let lhs = utils_tests::vector_tensor(vec![1.0, 2.0]);
        let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);

        let err = CausalTensor::mat_mul_2d(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
                expected: 2,
                found: 1
            })
        ));

        let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let rhs = utils_tests::vector_tensor(vec![5.0, 6.0]);

        let err = CausalTensor::mat_mul_2d(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::RankMismatch {
                expected: 2,
                found: 1
            })
        ));
    }

    #[test]
    fn test_mat_mul_2d_shape_mismatch() {
        let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0], 3, 2);

        let err = CausalTensor::mat_mul_2d(&lhs, &rhs).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
        ));
    }

    #[test]
    fn test_contract_mat_mul_success() {
        let lhs = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let rhs = utils_tests::matrix_tensor(vec![5.0, 6.0, 7.0, 8.0], 2, 2);
        let expected = utils_tests::matrix_tensor(vec![19.0, 22.0, 43.0, 50.0], 2, 2);

        let result = CausalTensor::contract(&lhs, &rhs, &[1], &[0]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_contract_dot_prod_success() {
        let lhs = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
        let rhs = utils_tests::vector_tensor(vec![4.0, 5.0, 6.0]);
        let expected = utils_tests::scalar_tensor(32.0); // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32

        let result = CausalTensor::contract(&lhs, &rhs, &[0], &[0]).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_contract_invalid_axes_len() {
        let lhs = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let rhs = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);

        let err = CausalTensor::contract(&lhs, &rhs, &[0, 1], &[0]).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_contract_axis_out_of_bounds() {
        let lhs = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let rhs = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);

        let err = CausalTensor::contract(&lhs, &rhs, &[0], &[2]).unwrap_err(); // RHS axis 2 is out of bounds
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));

        let err = CausalTensor::contract(&lhs, &rhs, &[2], &[0]).unwrap_err(); // LHS axis 2 is out of bounds
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_contract_shape_mismatch() {
        let lhs = utils_tests::matrix_tensor(vec![1.0; 6], 2, 3);
        let rhs = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);

        let err = CausalTensor::contract(&lhs, &rhs, &[1], &[0]).unwrap_err(); // LHS dim 1 (3) != RHS dim 0 (2)
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
        ));
    }

    #[test]
    fn test_element_wise_mul_success() {
        let lhs = utils_tests::vector_tensor(vec![1.0, 2.0, 3.0]);
        let rhs = utils_tests::vector_tensor(vec![4.0, 5.0, 6.0]);
        let expected = utils_tests::vector_tensor(vec![4.0, 10.0, 18.0]);

        let result = CausalTensor::element_wise_mul(&lhs, &rhs).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trace_success_matrix() {
        let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let expected = utils_tests::scalar_tensor(5.0); // 1.0 + 4.0

        let result = CausalTensor::trace(&operand, 0, 1).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trace_success_3d_tensor() {
        let operand =
            CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], vec![2, 2, 2]).unwrap();
        // Trace over axes 1 and 2 (matrices within the batch)
        // Batch 0: [[1,2],[3,4]] -> 1+4 = 5
        // Batch 1: [[5,6],[7,8]] -> 5+8 = 13
        let expected = utils_tests::vector_tensor(vec![5.0, 13.0]);

        let result = CausalTensor::trace(&operand, 1, 2).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_trace_invalid_axes_out_of_bounds() {
        let operand = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let err = CausalTensor::trace(&operand, 0, 2).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_trace_invalid_axes_identical() {
        let operand = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let err = CausalTensor::trace(&operand, 0, 0).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_trace_shape_mismatch() {
        let operand = CausalTensor::new(vec![1.0; 6], vec![2, 3]).unwrap(); // 2x3 matrix
        let err = CausalTensor::trace(&operand, 0, 1).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
        ));
    }

    #[test]
    fn test_diagonal_success_matrix() {
        let operand = utils_tests::matrix_tensor(vec![1.0, 2.0, 3.0, 4.0], 2, 2);
        let expected = utils_tests::vector_tensor(vec![1.0, 4.0]);

        let result = CausalTensor::diagonal(&operand, 0, 1).unwrap();
        assert_eq!(result, expected);
    }
    #[test]
    fn test_diagonal_success_3d_tensor() {
        let operand =
            CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], vec![2, 2, 2]).unwrap();
        // Extract diagonal over axes 1 and 2 (matrices within the batch)
        // Batch 0: [[1,2],[3,4]] -> [1,4]
        // Batch 1: [[5,6],[7,8]] -> [5,8]
        let expected = CausalTensor::new(vec![1.0, 4.0, 5.0, 8.0], vec![2, 2]).unwrap();

        let result = CausalTensor::diagonal(&operand, 1, 2).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_diagonal_invalid_axes_out_of_bounds() {
        let operand = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let err = CausalTensor::diagonal(&operand, 0, 2).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_diagonal_invalid_axes_identical() {
        let operand = utils_tests::matrix_tensor(vec![1.0; 4], 2, 2);
        let err = CausalTensor::diagonal(&operand, 0, 0).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::InvalidAxesSpecification {
                message: _
            })
        ));
    }

    #[test]
    fn test_diagonal_shape_mismatch() {
        let operand = CausalTensor::new(vec![1.0; 6], vec![2, 3]).unwrap(); // 2x3 matrix
        let err = CausalTensor::diagonal(&operand, 0, 1).unwrap_err();
        assert!(matches!(
            err,
            CausalTensorError::EinSumError(EinSumValidationError::ShapeMismatch { message: _ })
        ));
    }
}
