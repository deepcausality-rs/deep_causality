/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! QR Decomposition using Householder reflections.
//!
//! The QR decomposition factors a matrix A into A = QR where:
//! - Q is an orthogonal matrix (Q^T Q = I)
//! - R is an upper triangular matrix

use crate::{CausalTensor, CausalTensorError};
use core::iter::Sum;
use deep_causality_num::{One, RealField, Zero};

impl<T: Default> CausalTensor<T> {
    /// Computes the QR decomposition of a matrix using Householder reflections.
    ///
    /// For a matrix A of shape (m, n), returns (Q, R) where:
    /// - Q has shape (m, m) and is orthogonal
    /// - R has shape (m, n) and is upper triangular
    /// - A = Q * R
    ///
    /// # Algorithm
    ///
    /// Uses Householder reflections to zero out elements below the diagonal.
    /// For each column k, we compute a Householder vector v such that
    /// H = I - 2*v*v^T reflects the column to have zeros below the diagonal.
    ///
    /// # Errors
    ///
    /// - `DimensionMismatch` if the tensor is not 2-dimensional
    pub(in crate::types::causal_tensor) fn qr_impl(
        &self,
    ) -> Result<(Self, Self), CausalTensorError>
    where
        T: Clone + RealField + Zero + One + Sum + PartialEq,
    {
        if self.num_dim() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }

        let m = self.shape()[0]; // rows
        let n = self.shape()[1]; // cols
        let k = m.min(n); // number of Householder transformations

        // Initialize R as a copy of A
        let mut r_data: Vec<T> = self.as_slice().to_vec();

        // Initialize Q as identity matrix (m x m)
        let mut q_data: Vec<T> = vec![T::zero(); m * m];
        for i in 0..m {
            q_data[i * m + i] = T::one();
        }

        // Helper to get/set elements in row-major order
        let get_r = |data: &[T], row: usize, col: usize| data[row * n + col];
        let set_r = |data: &mut [T], row: usize, col: usize, val: T| {
            data[row * n + col] = val;
        };
        let get_q = |data: &[T], row: usize, col: usize| data[row * m + col];
        let set_q = |data: &mut [T], row: usize, col: usize, val: T| {
            data[row * m + col] = val;
        };

        // Apply Householder reflections
        for j in 0..k {
            // Extract the column vector from R[j:m, j]
            let mut v: Vec<T> = (j..m).map(|i| get_r(&r_data, i, j)).collect();

            // Compute the norm of v
            let norm: T = v.iter().map(|&x| x * x).sum::<T>().sqrt();

            if norm == T::zero() {
                continue; // Skip if column is already zero
            }

            // Compute Householder vector: v[0] += sign(v[0]) * norm
            let sign = if v[0] >= T::zero() {
                T::one()
            } else {
                -T::one()
            };
            v[0] = v[0] + sign * norm;

            // Normalize v
            let v_norm: T = v.iter().map(|&x| x * x).sum::<T>().sqrt();
            if v_norm == T::zero() {
                continue;
            }
            for x in &mut v {
                *x = *x / v_norm;
            }

            // Apply Householder reflection to R: R[j:m, j:n] = R - 2*v*(v^T * R)
            // For each column c from j to n-1
            for c in j..n {
                // Compute v^T * R[:, c] for rows j..m
                let dot: T = (0..v.len()).map(|i| v[i] * get_r(&r_data, j + i, c)).sum();

                // Update R[:, c] -= 2 * dot * v
                let two = T::one() + T::one();
                for i in 0..v.len() {
                    let old = get_r(&r_data, j + i, c);
                    set_r(&mut r_data, j + i, c, old - two * dot * v[i]);
                }
            }

            // Apply Householder reflection to Q: Q[:, j:m] = Q - 2*(Q*v)*v^T
            // For each row r from 0 to m-1
            for r in 0..m {
                // Compute Q[r, j:m] * v
                let dot: T = (0..v.len()).map(|i| get_q(&q_data, r, j + i) * v[i]).sum();

                // Update Q[r, j:m] -= 2 * dot * v
                let two = T::one() + T::one();
                for i in 0..v.len() {
                    let old = get_q(&q_data, r, j + i);
                    set_q(&mut q_data, r, j + i, old - two * dot * v[i]);
                }
            }
        }

        let q = CausalTensor::new(q_data, vec![m, m])?;
        let r = CausalTensor::new(r_data, vec![m, n])?;

        Ok((q, r))
    }
}
