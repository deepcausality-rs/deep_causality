/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gamma matrices for Clifford algebras.
//!
//! This module provides generator matrices (gamma matrices) for matrix representations
//! of Clifford algebras. These are used to convert between coefficient and matrix forms.

use deep_causality_metric::Metric;
use deep_causality_num::Field;
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Helper Functions
// =============================================================================

/// Computes the matrix dimension for a given algebra dimension.
pub fn matrix_dim(n: usize) -> usize {
    1 << n.div_ceil(2)
}

/// Returns the number of basis blades for dimension n.
pub fn num_blades(n: usize) -> usize {
    1 << n
}

/// Computes a single element of a gamma matrix.
///
/// Uses the Brauer-Weyl construction for Clifford algebra representations.
pub fn compute_gamma_element<T>(gamma_idx: usize, row: usize, col: usize, metric: &Metric) -> T
where
    T: Field + std::ops::Neg<Output = T>,
{
    let n = metric.dimension();
    let dim = matrix_dim(n);

    if row >= dim || col >= dim {
        return T::zero();
    }

    // Brauer-Weyl construction
    let block_size = 1 << (gamma_idx / 2);
    let block_row = row / block_size;
    let block_col = col / block_size;
    let inner_row = row % block_size;
    let inner_col = col % block_size;

    // Check if we're on a valid block
    if inner_row != inner_col {
        return T::zero();
    }

    // Even generators: use sigma_x pattern
    // Odd generators: use sigma_z pattern
    let is_even = gamma_idx.is_multiple_of(2);

    let value = if is_even {
        // sigma_x: off-diagonal
        if block_row + block_col == 1 {
            T::one()
        } else {
            T::zero()
        }
    } else {
        // sigma_z: diagonal
        if block_row == block_col {
            if block_row == 0 { T::one() } else { -T::one() }
        } else {
            T::zero()
        }
    };

    // Apply metric signature
    let sig = metric.sign_of_sq(gamma_idx);
    if sig < 0 {
        // Negative signature: multiply by i (for real types, use -1)
        -value
    } else if sig == 0 {
        // Null signature
        T::zero()
    } else {
        value
    }
}

// =============================================================================
// Public API - CausalTensor-based functions
// =============================================================================

/// Gets gamma matrices for all generators.
///
/// Returns a tensor of shape [N, D, D] where N is the algebra dimension
/// and D is the matrix dimension.
pub fn get_gammas<T>(metric: &Metric) -> CausalTensor<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + std::ops::Neg<Output = T>,
{
    let n = metric.dimension();
    let dim = matrix_dim(n);
    let shape = [n, dim, dim];

    CausalTensor::from_shape_fn(&shape, |idx| {
        compute_gamma_element::<T>(idx[0], idx[1], idx[2], metric)
    })
}

/// Gets basis blade matrices (gamma products for all blades).
///
/// Returns a tensor of shape [2^N, D, D] containing the matrix for each blade.
pub fn get_basis_gammas<T>(metric: &Metric) -> CausalTensor<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + std::ops::Neg<Output = T>,
{
    let n = metric.dimension();
    let num_blades = num_blades(n);
    let dim = matrix_dim(n);
    let shape = [num_blades, dim, dim];

    // Get individual gamma matrices
    let gammas = get_gammas::<T>(metric);
    let gamma_data = gammas.to_vec();

    let mut result_data = vec![T::zero(); num_blades * dim * dim];

    for blade_idx in 0..num_blades {
        if blade_idx == 0 {
            // Identity blade
            for i in 0..dim {
                result_data[i * dim + i] = T::one();
            }
        } else {
            // Compute product of gammas for this blade
            let mut matrix = vec![T::zero(); dim * dim];

            // Start with identity
            for i in 0..dim {
                matrix[i * dim + i] = T::one();
            }

            // Multiply by each gamma in the blade
            for gamma_idx in 0..n {
                if blade_idx & (1 << gamma_idx) != 0 {
                    let mut new_matrix = vec![T::zero(); dim * dim];
                    for i in 0..dim {
                        for j in 0..dim {
                            let mut sum = T::zero();
                            for k in 0..dim {
                                let m_ik = matrix[i * dim + k];
                                let g_kj = gamma_data[gamma_idx * dim * dim + k * dim + j];
                                sum = sum + m_ik * g_kj;
                            }
                            new_matrix[i * dim + j] = sum;
                        }
                    }
                    matrix = new_matrix;
                }
            }

            // Copy to result
            for i in 0..dim * dim {
                result_data[blade_idx * dim * dim + i] = matrix[i];
            }
        }
    }

    CausalTensor::from_slice(&result_data, &shape)
}

/// Gets dual basis gamma matrices (for trace projection).
///
/// Returns a tensor of shape [2^N, D, D] containing the dual (inverse transpose) of each blade matrix.
pub fn get_dual_basis_gammas<T>(metric: &Metric) -> CausalTensor<T>
where
    T: Field + Copy + Default + PartialOrd + Send + Sync + std::ops::Neg<Output = T>,
{
    let n = metric.dimension();
    let num_blades = num_blades(n);
    let dim = matrix_dim(n);
    let shape = [num_blades, dim, dim];

    let basis = get_basis_gammas::<T>(metric);
    let basis_data = basis.to_vec();

    let mut result_data = vec![T::zero(); num_blades * dim * dim];

    for blade_idx in 0..num_blades {
        // For orthogonal matrices, inverse = transpose
        // Dual basis is transposed for trace projection
        for i in 0..dim {
            for j in 0..dim {
                // Transpose: result[i,j] = basis[j,i]
                result_data[blade_idx * dim * dim + i * dim + j] =
                    basis_data[blade_idx * dim * dim + j * dim + i];
            }
        }
    }

    CausalTensor::from_slice(&result_data, &shape)
}
