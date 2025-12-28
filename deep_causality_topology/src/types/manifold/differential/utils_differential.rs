/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Manifold;
use core::ops::Mul;
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;

impl<T> Manifold<T>
where
    T: Field + Default + Copy + PartialEq,
{
    /// Helper to extract data for a specific k-form from the flat storage.
    pub(super) fn get_k_form_data(&self, k: usize) -> Vec<T> {
        let mut offset = 0;
        for i in 0..k {
            if i < self.complex.skeletons().len() {
                offset += self.complex.skeletons()[i].simplices().len();
            }
        }

        let size = if k < self.complex.skeletons().len() {
            self.complex.skeletons()[k].simplices().len()
        } else {
            0
        };

        if offset + size <= self.data().len() {
            self.data().as_slice()[offset..offset + size].to_vec()
        } else {
            // Return zeros if data is missing/mismatched (Graceful degradation)
            vec![T::zero(); size]
        }
    }

    /// Helper to create a temporary manifold holding data for a specific k-form.
    /// Used to chain operators.
    pub(super) fn create_temp_manifold(&self, k: usize, k_data: CausalTensor<T>) -> Self {
        let total_size = self.data().len();
        let mut full_data = vec![T::zero(); total_size];

        let mut offset = 0;
        for i in 0..k {
            offset += self.complex.skeletons()[i].simplices().len();
        }

        let slice = k_data.as_slice();
        if offset + slice.len() <= total_size {
            full_data[offset..offset + slice.len()].copy_from_slice(slice);
        }

        Manifold::new(
            self.complex.clone(),
            CausalTensor::new(full_data, self.data().shape().to_vec()).unwrap(),
            0,
        )
        .unwrap()
    }
}

/// Helper to apply a sparse matrix operator to a vector
pub(super) fn apply_operator<T>(matrix: &deep_causality_sparse::CsrMatrix<i8>, data: &[T]) -> Vec<T>
where
    T: Field + Copy + core::ops::Neg<Output = T>,
{
    let (rows, cols) = matrix.shape();

    if cols != data.len() {
        // Dimension mismatch, return zeros
        return vec![T::zero(); rows];
    }

    let mut result = vec![T::zero(); rows];

    for (row, res_val) in result.iter_mut().enumerate() {
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

            *res_val = *res_val + (coeff * data[col]);
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

    for (row, res_val) in result.iter_mut().enumerate() {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx];

            *res_val = *res_val + (data[col] * val);
        }
    }

    result
}
