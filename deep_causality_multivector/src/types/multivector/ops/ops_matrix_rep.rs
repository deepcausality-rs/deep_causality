/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! MatrixRep trait implementation for CausalMultiVector.
//!
//! Enables conversion between coefficient representation and matrix representation
//! for hardware-accelerated Clifford algebra operations.

use crate::CausalMultiVector;
use crate::types::multifield::gamma::{BackendGamma, GammaProvider};
use deep_causality_metric::Metric;
use deep_causality_tensor::TensorData;
use std::ops::Neg;

impl<T> CausalMultiVector<T>
where
    T: TensorData + Clone + Neg<Output = T>,
{
    pub fn to_matrix_on_backend<B>(&self) -> B::Tensor<T>
    where
        B: GammaProvider<T>,
    {
        let n = self.metric.dimension();
        let num_blades = 1usize << n;
        let matrix_dim = 1usize << n.div_ceil(2);

        // 1. Get coefficients as a tensor [num_blades, 1, 1]
        // This allows broadcasting with [num_blades, D, D]
        let coeffs_tensor = B::create(&self.data, &[num_blades, 1, 1]);

        // 2. Get batch of basis gammas [num_blades, matrix_dim, matrix_dim]
        let basis_gammas = B::GammaLoader::get_basis_gammas(&self.metric);

        // 3. Multiply: Coeffs * Basis (broadcasting handles [B,1,1] * [B,D,D])
        let scaled = B::mul(&coeffs_tensor, &basis_gammas);

        // 4. Sum along the batch axis (0) to get result matrix [matrix_dim, matrix_dim]
        let sum = B::sum(&scaled, &[0]);

        // Reshape result to [D, D]
        B::reshape(&sum, &[matrix_dim, matrix_dim])
    }

    /// Converts Matrix Representation back to coefficients.
    ///
    /// Uses trace projection: aᵢ = Tr(M · Γᵢ†) / matrix_dim
    pub fn from_matrix_on_backend<B>(matrix: B::Tensor<T>, metric: Metric) -> Self
    where
        B: GammaProvider<T>,
        T: Default,
    {
        let n = metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);

        // 1. Get dual basis gammas [num_blades, D, D]
        let dual_basis = B::GammaLoader::get_dual_basis_gammas(&metric);

        // 2. Prepare matrix for batch contraction [1, D, D]
        let matrix_batch = B::reshape(&matrix, &[1, matrix_dim, matrix_dim]);

        // 3. Trace projection via element-wise product and sum
        // Tr(M * G) = sum_{r,c} M_rc * (G^T)_rc
        // dual_basis[i] already stores (Gamma_i^{-1})^T
        let product = B::mul(&matrix_batch, &dual_basis);

        // 4. Sum along R and C axes (1, 2) to get [num_blades]
        let sum = B::sum(&product, &[1, 2]);

        // 5. Normalization: divide by matrix_dim
        // In the Brauer-Weyl construction, Tr(I) = matrix_dim
        let mut dim_t = T::zero();
        for _ in 0..matrix_dim {
            dim_t = dim_t + T::one();
        }

        let normalized = if !dim_t.is_zero() {
            let scale = B::from_shape_fn(&[1], |_| T::one() / dim_t);
            B::mul(&sum, &scale)
        } else {
            sum
        };

        let coeffs = B::to_vec(&normalized);
        Self::unchecked(coeffs, metric)
    }

    /// Gets the gamma matrix for a specific blade index.
    pub fn get_gamma_matrix<B>(&self, blade_idx: usize) -> B::Tensor<T>
    where
        B: GammaProvider<T>,
    {
        let n = self.metric.dimension();
        let matrix_dim = 1usize << n.div_ceil(2);

        // We can leverage the batch loader and slice it
        let basis_gammas = B::GammaLoader::get_basis_gammas(&self.metric);

        // Slince the [num_blades, D, D] tensor to get [1, D, D] at blade_idx
        let start = blade_idx;
        let end = blade_idx + 1;
        let slice = B::slice(&basis_gammas, &[start..end, 0..matrix_dim, 0..matrix_dim]);

        B::reshape(&slice, &[matrix_dim, matrix_dim])
    }
}
