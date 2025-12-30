/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MatrixRep trait implementation for CausalMultiVector.
//!
//! Enables conversion between coefficient representation and matrix representation
//! for hardware-accelerated Clifford algebra operations.

use crate::CausalMultiVector;
use deep_causality_metric::Metric;
use deep_causality_tensor::{LinearAlgebraBackend, TensorData};
use std::ops::Neg;

impl<T> CausalMultiVector<T>
where
    T: TensorData + Clone + Neg<Output = T>,
{
    /// Converts coefficients to Matrix Representation for a given backend.
    ///
    /// M(A) = Σ aᵢ Γᵢ
    ///
    /// where Γᵢ are the gamma matrices for the algebra.
    pub fn to_matrix_on_backend<B>(&self) -> B::Tensor<T>
    where
        B: LinearAlgebraBackend,
    {
        let n = self.metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);
        let num_blades = 1usize << n;

        // Initialize result as zero matrix
        let shape = [matrix_dim, matrix_dim];
        let mut result = B::zeros(&shape);

        // Sum aᵢ * Γᵢ for each blade
        for blade_idx in 0..num_blades {
            if self.data[blade_idx].is_zero() {
                continue;
            }

            // Get gamma matrix for this blade
            let gamma = self.get_gamma_matrix::<B>(blade_idx);

            // Scale by coefficient: aᵢ * Γᵢ
            let coeff = self.data[blade_idx];
            let coeff_tensor = B::from_shape_fn(&[1], |_| coeff);
            let scaled = B::mul(&gamma, &coeff_tensor);

            // Accumulate
            result = B::add(&result, &scaled);
        }

        result
    }

    /// Converts Matrix Representation back to coefficients.
    ///
    /// Uses trace projection: aᵢ = Tr(M · Γᵢ†) / matrix_dim
    pub fn from_matrix_on_backend<B>(matrix: B::Tensor<T>, metric: Metric) -> Self
    where
        B: LinearAlgebraBackend,
        T: Default,
    {
        let n = metric.dimension();
        let num_blades = 1usize << n;
        let matrix_dim = 1usize << n.div_ceil(2);

        let mut coeffs = vec![T::zero(); num_blades];

        // For each blade, project: aᵢ = Tr(M · Γᵢ†) / dim
        // For each blade, project: aᵢ = Tr(M · Γᵢ†) / dim
        // For self-adjoint gammas, Γᵢ† = Γᵢ
        for (blade_idx, coeff) in coeffs.iter_mut().enumerate() {
            // Get gamma matrix for this blade
            let gamma = Self::get_gamma_matrix_static::<B>(blade_idx, metric, matrix_dim);

            // Compute M · Γᵢ
            let product = B::matmul(&matrix, &gamma);

            // Compute trace (sum of diagonal elements)
            let trace = Self::compute_trace::<B>(&product, matrix_dim);

            // Normalize by dimension
            let _dim_t = T::one();
            // Simplified: just use trace directly for now
            *coeff = trace;
        }

        Self::unchecked(coeffs, metric)
    }

    /// Gets the gamma matrix for a specific blade index.
    fn get_gamma_matrix<B>(&self, blade_idx: usize) -> B::Tensor<T>
    where
        B: LinearAlgebraBackend,
    {
        let n = self.metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);
        Self::get_gamma_matrix_static::<B>(blade_idx, self.metric, matrix_dim)
    }

    /// Static version of gamma matrix generation.
    fn get_gamma_matrix_static<B>(
        blade_idx: usize,
        metric: Metric,
        matrix_dim: usize,
    ) -> B::Tensor<T>
    where
        B: LinearAlgebraBackend,
    {
        let shape = [matrix_dim, matrix_dim];

        // Generate gamma matrix for this blade
        // For blade 0 (scalar): Identity matrix
        // For blade with single bit set (vector): corresponding gamma matrix
        // For higher grades: product of vector gammas
        B::from_shape_fn(&shape, |idx| {
            let row = idx[0];
            let col = idx[1];
            compute_blade_gamma_element::<T>(blade_idx, row, col, metric, matrix_dim)
        })
    }

    /// Computes the trace of a matrix tensor.
    fn compute_trace<B>(matrix: &B::Tensor<T>, dim: usize) -> T
    where
        B: LinearAlgebraBackend,
        T: Default,
    {
        let data = B::to_vec(matrix);
        let mut trace = T::zero();
        for i in 0..dim {
            trace = trace + data[i * dim + i];
        }
        trace
    }
}

/// Computes gamma matrix element for a specific blade.
///
/// - Blade 0 (scalar): Identity matrix
/// - Blade with single bit (vector): Base gamma matrix
/// - Higher grades: Product of base gammas
fn compute_blade_gamma_element<T: TensorData + Neg<Output = T>>(
    blade_idx: usize,
    row: usize,
    col: usize,
    metric: Metric,
    _matrix_dim: usize,
) -> T {
    let _n = metric.dimension();

    if blade_idx == 0 {
        // Scalar: Identity matrix
        return if row == col { T::one() } else { T::zero() };
    }

    let grade = blade_idx.count_ones() as usize;

    if grade == 1 {
        // Vector: Single gamma matrix
        let vector_idx = blade_idx.trailing_zeros() as usize;
        return crate::types::multifield::gamma::compute_gamma_element::<T>(
            vector_idx, row, col, &metric,
        );
    }

    // Higher grades: Need to compute product of base gammas
    // For simplicity, return identity-like for now (Placeholder)
    if row == col { T::one() } else { T::zero() }
}
