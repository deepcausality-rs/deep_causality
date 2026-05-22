/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::chain_complex::ChainComplex;
use crate::types::manifold::Manifold;
use core::ops::Mul;
use deep_causality_num::{Field, RealField};
use deep_causality_tensor::CausalTensor;

impl<K, D> Manifold<K, D>
where
    K: ChainComplex,
    D: RealField + Default + PartialEq,
{
    /// Extract the slice of data corresponding to grade-k forms from the flat
    /// per-grade storage carried by `self.data`. Generic over the chain complex
    /// backend via `ChainComplex::num_cells`.
    pub(super) fn get_k_form_data(&self, k: usize) -> Vec<D> {
        let max_dim = self.complex.max_dim();

        let mut offset = 0usize;
        for i in 0..k {
            if i <= max_dim {
                offset += self.complex.num_cells(i);
            }
        }

        let size = if k <= max_dim {
            self.complex.num_cells(k)
        } else {
            0
        };

        if offset + size <= self.data().len() {
            self.data().as_slice()[offset..offset + size].to_vec()
        } else {
            // Graceful degradation: return zeros if data is missing/mismatched.
            vec![D::zero(); size]
        }
    }
}

impl<K, D> Manifold<K, D>
where
    K: ChainComplex + Clone,
    K::Metric: Clone,
    D: RealField + Default + PartialEq,
{
    /// Create a temporary manifold holding data for grade `k` only, used to
    /// chain differential operators (e.g. compute `d δ ω`). Generic over the
    /// chain complex backend.
    pub(super) fn create_temp_manifold(&self, k: usize, k_data: CausalTensor<D>) -> Self {
        let total_size = self.data().len();
        let mut full_data = vec![D::zero(); total_size];

        let mut offset = 0usize;
        for i in 0..k {
            offset += self.complex.num_cells(i);
        }

        let slice = k_data.as_slice();
        if offset + slice.len() <= total_size {
            full_data[offset..offset + slice.len()].copy_from_slice(slice);
        }

        Manifold {
            complex: self.complex.clone(),
            data: CausalTensor::new(full_data, self.data().shape().to_vec()).unwrap(),
            metric: self.metric.clone(),
            cursor: 0,
        }
    }
}

/// Helper to apply a sparse matrix operator to a vector
pub(super) fn apply_operator<D>(matrix: &deep_causality_sparse::CsrMatrix<i8>, data: &[D]) -> Vec<D>
where
    D: Field + Copy + core::ops::Neg<Output = D>,
{
    let (rows, cols) = matrix.shape();

    if cols != data.len() {
        // Dimension mismatch, return zeros
        return vec![D::zero(); rows];
    }

    let mut result = vec![D::zero(); rows];

    for (row, res_val) in result.iter_mut().enumerate() {
        let row_start = matrix.row_indices()[row];
        let row_end = matrix.row_indices()[row + 1];

        for idx in row_start..row_end {
            let col = matrix.col_indices()[idx];
            let val = matrix.values()[idx];

            // Convert i8 to D
            let coeff = if val == 0 {
                D::zero()
            } else if val > 0 {
                D::one()
            } else {
                D::zero() - D::one() // -1
            };

            *res_val = *res_val + (coeff * data[col]);
        }
    }

    result
}

/// Helper to apply a sparse matrix operator with C values to a vector of D
pub(super) fn apply_metric_operator<C, D>(
    matrix: &deep_causality_sparse::CsrMatrix<C>,
    data: &[D],
) -> Vec<D>
where
    C: Copy,
    D: Field + Copy + Mul<C, Output = D>,
{
    let (rows, cols) = matrix.shape();

    if cols != data.len() {
        // Dimension mismatch, return zeros
        return vec![D::zero(); rows];
    }

    let mut result = vec![D::zero(); rows];

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
