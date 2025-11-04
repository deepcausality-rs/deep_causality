/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError, EinSumAST, EinSumValidationError};
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
        if lhs.num_dim() != 2 || rhs.num_dim() != 2 {
            return Err(CausalTensorError::EinSumError(
                EinSumValidationError::RankMismatch {
                    expected: 2,
                    found: lhs.num_dim(),
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
        let mut new_data = Vec::with_capacity(diag_len);

        for i in 0..diag_len {
            let mut index = vec![0; tensor.num_dim()];
            index[axis1] = i;
            index[axis2] = i;
            new_data.push(tensor.get(&index).unwrap().clone());
        }

        CausalTensor::new(new_data, vec![diag_len])
    }
}
