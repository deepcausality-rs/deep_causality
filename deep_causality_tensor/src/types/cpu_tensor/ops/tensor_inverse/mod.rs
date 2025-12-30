/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensorError, InternalCpuTensor};
use deep_causality_num::RealField;

use crate::TensorData;

impl<T> InternalCpuTensor<T> {
    pub(in crate::types::cpu_tensor) fn inverse_impl(&self) -> Result<Self, CausalTensorError>
    where
        T: TensorData + RealField,
    {
        let ndim = self.ndim();
        if ndim < 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }

        let rows = self.shape[ndim - 2];
        let cols = self.shape[ndim - 1];

        if rows != cols {
            return Err(CausalTensorError::ShapeMismatch); // Not a square matrix
        }

        if ndim == 2 {
            self.inverse_2d_impl()
        } else {
            // Batched Matrix Inversion

            // Calculate total number of matrices in the batch
            let batch_size: usize = self.shape.iter().take(ndim - 2).product();
            let matrix_size = rows * cols;

            // Result data
            let mut result_data = Vec::with_capacity(self.data.len());

            for b in 0..batch_size {
                let offset = b * matrix_size;
                let batch_slice = &self.data[offset..offset + matrix_size];

                // Construct a temporary 2D tensor for this batch
                let temp_tensor = Self::new(batch_slice.to_vec(), vec![rows, cols])?;
                let inv_tensor = temp_tensor.inverse_2d_impl()?;

                result_data.extend(inv_tensor.data);
            }

            Ok(Self::new(result_data, self.shape.clone())?)
        }
    }

    fn inverse_2d_impl(&self) -> Result<Self, CausalTensorError>
    where
        T: TensorData + RealField,
    {
        let n = self.shape[0];

        // Handle 1x1 matrix case
        if n == 1 {
            let val = self.data[0];
            if val.is_zero() {
                return Err(CausalTensorError::SingularMatrix);
            }
            return Self::new(vec![T::one() / val], vec![1, 1]); // Should not fail for 1x1
        }

        // Create an augmented matrix [A | I]
        let mut augmented_data = Vec::with_capacity(n * 2 * n);
        for r in 0..n {
            for c in 0..n {
                augmented_data.push(self.data[r * n + c]);
            }
            for c in 0..n {
                if r == c {
                    augmented_data.push(T::one());
                } else {
                    augmented_data.push(T::zero());
                }
            }
        }

        let mut augmented_matrix = Self::from_vec_and_shape_unchecked(augmented_data, &[n, 2 * n]);

        // Gaussian Elimination
        for i in 0..n {
            // Estimate scale using max absolute value in column i
            let mut pivot_row = i;
            let mut max_val = T::zero();
            for r in i..n {
                let current_val = augmented_matrix.get(&[r, i]).unwrap().abs();
                if current_val > max_val {
                    max_val = current_val;
                    pivot_row = r;
                }
            }
            // Use relative threshold: tol = epsilon * max(1, max_val)
            let one = T::one();
            let tol = T::epsilon() * if max_val > one { max_val } else { one };
            if max_val <= tol {
                return Err(CausalTensorError::SingularMatrix);
            }

            // Swap pivot row with current row if necessary
            if pivot_row != i {
                for c in 0..2 * n {
                    let a = *augmented_matrix.get(&[i, c]).unwrap();
                    let b = *augmented_matrix.get(&[pivot_row, c]).unwrap();
                    if let Some(val) = augmented_matrix.get_mut(&[i, c]) {
                        *val = b;
                    }
                    if let Some(val) = augmented_matrix.get_mut(&[pivot_row, c]) {
                        *val = a;
                    }
                }
            }

            // Normalize pivot row across ALL columns to make pivot exactly 1
            let pivot_val = *augmented_matrix.get(&[i, i]).unwrap();
            if pivot_val.abs() < T::epsilon() {
                return Err(CausalTensorError::DivisionByZero);
            }
            for c in 0..2 * n {
                let val = *augmented_matrix.get(&[i, c]).unwrap();
                if let Some(dest) = augmented_matrix.get_mut(&[i, c]) {
                    *dest = val / pivot_val;
                }
            }

            // Eliminate other rows
            for r in 0..n {
                if r == i {
                    continue;
                }
                let factor = *augmented_matrix.get(&[r, i]).unwrap();
                if factor.abs() < T::epsilon() {
                    continue;
                }
                for c in 0..2 * n {
                    let v_rc = *augmented_matrix.get(&[r, c]).unwrap();
                    let v_ic = *augmented_matrix.get(&[i, c]).unwrap();
                    if let Some(dest) = augmented_matrix.get_mut(&[r, c]) {
                        *dest = v_rc - factor * v_ic;
                    }
                }
            }
        }

        // Extract the inverse matrix
        let mut inverse_data = Vec::with_capacity(n * n);
        for r in 0..n {
            for c in n..2 * n {
                inverse_data.push(*augmented_matrix.get(&[r, c]).unwrap());
            }
        }

        Ok(Self::from_vec_and_shape_unchecked(inverse_data, &[n, n]))
    }
}
