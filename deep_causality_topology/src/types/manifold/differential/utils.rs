/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::ops::Mul;
use deep_causality_num::{Field,};

/// Helper to apply a sparse matrix operator to a vector
pub(super) fn apply_operator<T>(matrix: &deep_causality_sparse::CsrMatrix<i8>, data: &[T]) -> Vec<T>
where
    T: Field + Copy + std::ops::Neg<Output = T>,
{
    let (rows, cols) = matrix.shape();

    if cols != data.len() {
        // Dimension mismatch, return zeros
        return vec![T::zero(); rows];
    }

    let mut result = vec![T::zero(); rows];

    for row in 0..rows {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx];

            // Convert i8 to T
            let coeff = if val == 0 {
                T::zero()
            } else if val > 0 {
                T::one()
            } else {
                T::zero() - T::one() // -1
            };

            result[row] = result[row] + (coeff * data[col]);
        }
    }

    result
}

/// Helper to apply a sparse matrix operator with f64 values to a vector
pub(super) fn apply_f64_operator<T>(
    matrix: &deep_causality_sparse::CsrMatrix<f64>,
    data: &[T],
) -> Vec<T>
where
    T: Field + Copy + Mul<f64, Output = T>,
{
    let (rows, cols) = matrix.shape();

    if cols != data.len() {
        // Dimension mismatch, return zeros
        return vec![T::zero(); rows];
    }

    let mut result = vec![T::zero(); rows];

    for row in 0..rows {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx];

            result[row] = result[row] + (data[col] * val);
        }
    }

    result
}
