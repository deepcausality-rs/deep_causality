/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensorError, InternalCpuTensor, Tensor};
use deep_causality_num::RealField;

use crate::backend::TensorData;

impl<T> InternalCpuTensor<T>
where
    T: TensorData + RealField,
{
    pub(in crate::types::cpu_tensor) fn solve_least_squares_cholsky_impl(
        a: &Self, // Design matrix (m x n)
        b: &Self, // Observation vector (m x 1)
    ) -> Result<Self, CausalTensorError> {
        // Input validation
        if a.ndim() != 2 || b.ndim() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = a.shape()[0];
        let n = a.shape()[1]; // Number of parameters
        if b.shape()[0] != m || b.shape()[1] != 1 {
            return Err(CausalTensorError::ShapeMismatch); // b must be a column vector
        }

        // 1. Calculate A^T using a strided view via permute_axes
        let a_t = a.permute_axes(&[1, 0])?;

        // 2. Calculate M = A^T A
        let m_matrix = InternalCpuTensor::mat_mul_2d(&a_t, a)?;

        // 3. Calculate y = A^T b
        let y_vector = InternalCpuTensor::mat_mul_2d(&a_t, b)?;

        // 4. Cholesky decomposition of M: M = LL^T
        let l_matrix = m_matrix.cholesky_decomposition_impl()?;

        // 5. Solve Lz = y for z (forward substitution)
        let z_data = vec![T::zero(); n];
        let mut z_vector = InternalCpuTensor::from_vec_and_shape_unchecked(z_data, &[n, 1]);

        for i in 0..n {
            let mut sum = T::zero();
            for j in 0..i {
                let l_val = *l_matrix
                    .get(&[i, j])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                let z_val = *z_vector
                    .get(&[j, 0])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                sum += l_val * z_val;
            }
            let y_val = *y_vector
                .get(&[i, 0])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let l_diag = *l_matrix
                .get(&[i, i])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let val = (y_val - sum) / l_diag;

            if let Some(z_ref) = z_vector.get_mut(&[i, 0]) {
                *z_ref = val;
            } else {
                return Err(CausalTensorError::IndexOutOfBounds);
            }
        }

        // 6. Solve L^T x = z for x (backward substitution)
        // Here we CAN use permute_axes because we access elements via get, which respects strides.
        let l_t_matrix = l_matrix.permute_axes(&[1, 0])?; // L^T

        let x_data = vec![T::zero(); n];
        let mut x_vector = InternalCpuTensor::from_vec_and_shape_unchecked(x_data, &[n, 1]);

        for i in (0..n).rev() {
            let mut sum = T::zero();
            for j in i + 1..n {
                let lt_val = *l_t_matrix
                    .get(&[i, j])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                let x_val = *x_vector
                    .get(&[j, 0])
                    .ok_or(CausalTensorError::IndexOutOfBounds)?;
                sum += lt_val * x_val;
            }
            let z_val = *z_vector
                .get(&[i, 0])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let lt_diag = *l_t_matrix
                .get(&[i, i])
                .ok_or(CausalTensorError::IndexOutOfBounds)?;
            let val = (z_val - sum) / lt_diag;

            if let Some(x_ref) = x_vector.get_mut(&[i, 0]) {
                *x_ref = val;
            } else {
                return Err(CausalTensorError::IndexOutOfBounds);
            }
        }
        Ok(x_vector)
    }

    pub(in crate::types::cpu_tensor) fn cholesky_decomposition_impl(
        &self,
    ) -> Result<Self, CausalTensorError> {
        // Input validation: Must be a square matrix
        let ndim = self.ndim();
        if ndim != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let n = self.shape()[0];
        if n != self.shape()[1] {
            return Err(CausalTensorError::ShapeMismatch);
        }

        let l_data = vec![T::zero(); n * n];
        let mut l_matrix = InternalCpuTensor::from_vec_and_shape_unchecked(l_data, &[n, n]);

        for i in 0..n {
            for j in 0..i + 1 {
                // Iterate up to and including the diagonal
                let mut sum = T::zero();
                for k in 0..j {
                    let l_ik = *l_matrix
                        .get(&[i, k])
                        .ok_or(CausalTensorError::IndexOutOfBounds)?;
                    let l_jk = *l_matrix
                        .get(&[j, k])
                        .ok_or(CausalTensorError::IndexOutOfBounds)?;
                    sum += l_ik * l_jk;
                }

                if i == j {
                    // Diagonal elements
                    let s_ii = *self
                        .get(&[i, i])
                        .ok_or(CausalTensorError::IndexOutOfBounds)?;
                    let val = s_ii - sum;
                    if val <= T::zero() {
                        // Check for positive definiteness
                        return Err(CausalTensorError::SingularMatrix); // Not positive definite
                    }
                    let sqrt_val = val.sqrt(); // Need a sqrt method on T

                    if let Some(l_ref) = l_matrix.get_mut(&[i, j]) {
                        *l_ref = sqrt_val;
                    }
                } else {
                    // Off-diagonal elements
                    let s_ij = *self
                        .get(&[i, j])
                        .ok_or(CausalTensorError::IndexOutOfBounds)?;
                    let l_jj = *l_matrix
                        .get(&[j, j])
                        .ok_or(CausalTensorError::IndexOutOfBounds)?;
                    let val = (s_ij - sum) / l_jj;

                    if let Some(l_ref) = l_matrix.get_mut(&[i, j]) {
                        *l_ref = val;
                    }
                }
            }
        }
        Ok(l_matrix)
    }
}
