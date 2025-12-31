/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gamma matrix generation for Clifford algebra representations.

pub mod cpu;
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub mod mlx;
pub mod provider;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
pub use mlx::*;
pub use provider::*;

use deep_causality_metric::Metric;
use deep_causality_tensor::{TensorBackend, TensorData};

/// Trait for gamma matrix loading per backend.
///
/// Gamma matrices provide the Matrix Isomorphism of the Clifford Algebra.
/// Each blade $e_{i_1...i_k}$ is represented by a $D \times D$ matrix,
/// where $D = 2^{\lceil N/2 \rceil}$.
pub trait BackendGamma<B: TensorBackend, T: TensorData> {
    /// Returns the N generator gamma matrices: $\gamma_0, \ldots, \gamma_{N-1}$.
    ///
    /// Shape: [N, D, D]
    fn get_gammas(metric: &Metric) -> B::Tensor<T>;

    /// Returns the $2^N$ basis blade matrices: $\Gamma_0 = I, \Gamma_1 = \gamma_0, \ldots$
    ///
    /// These are formed by products of gamma matrices according to the blade's
    /// bitmask index.
    ///
    /// Shape: [2^N, D, D]
    fn get_basis_gammas(metric: &Metric) -> B::Tensor<T>;

    /// Returns the dual (inverse) basis matrices used for coefficient extraction.
    ///
    /// Shape: [2^N, D, D]
    fn get_dual_basis_gammas(metric: &Metric) -> B::Tensor<T>;
}

/// Computes a single element of a gamma matrix for arbitrary dimension N.
///
/// Uses the Brauer-Weyl construction with tensor products of 2x2 Pauli-like matrices.
/// The construction respects the metric signature.
///
/// For a generator at position `gamma_idx`:
/// - Slot position: k = gamma_idx / 2
/// - Position within slot: is_second = (gamma_idx % 2) != 0
///
/// Construction at each slot:
/// - Slot j < k: σ_z (for anticommutation with previous slots)
/// - Slot j = k:
///     - First generator: σ_x (off-diagonal)
///     - Second generator: σ_y if signature requires -1, else σ_z if +1
/// - Slot j > k: I (identity)
///
/// This ensures:
/// - γ_i² = metric.sign_of_sq(i) * I
/// - γ_i γ_j + γ_j γ_i = 0 for i ≠ j
pub fn compute_gamma_element<T: TensorData + std::ops::Neg<Output = T>>(
    gamma_idx: usize,
    row: usize,
    col: usize,
    metric: &Metric,
) -> T {
    let n = metric.dimension();
    let matrix_dim = 1 << n.div_ceil(2);
    let num_slots = n.div_ceil(2);

    // Target slot for this generator
    let k = gamma_idx / 2;

    // Is this the second generator in the slot?
    let is_second = !gamma_idx.is_multiple_of(2);

    // What does this generator square to?
    let sign_sq = metric.sign_of_sq(gamma_idx);

    // Check bounds
    if row >= matrix_dim || col >= matrix_dim {
        return T::zero();
    }

    let mut final_sign = 1i8;

    // Iterate over tensor slots (bits)
    for slot in 0..num_slots {
        // Bit index within the matrix indices (0 is LSB)
        let bit_shift = num_slots - 1 - slot;

        let r_bit = (row >> bit_shift) & 1;
        let c_bit = (col >> bit_shift) & 1;

        if slot < k {
            // Previous slots: Must be sigma_z for anticommutation
            // sigma_z: diag(1, -1). r==c. val = (-1)^r
            if r_bit != c_bit {
                return T::zero();
            }
            if r_bit == 1 {
                final_sign *= -1;
            }
        } else if slot == k {
            if !is_second {
                // First generator in slot: sigma_x = [[0, 1], [1, 0]]
                // r != c, always value 1
                if r_bit == c_bit {
                    return T::zero();
                }
                // σ_x² = I (always +1)
            } else {
                // Second generator in slot
                if sign_sq == 1 {
                    // Need generator that squares to +1 and anticommutes with σ_x
                    // Use σ_z = diag(1, -1)
                    // σ_z anticommutes with σ_x, and σ_z² = I
                    if r_bit != c_bit {
                        return T::zero();
                    }
                    if r_bit == 1 {
                        final_sign *= -1;
                    }
                } else {
                    // Need generator that squares to -1 and anticommutes with σ_x
                    // Use ε = [[0, -1], [1, 0]] = i*σ_y (symplectic)
                    // ε² = -I
                    if r_bit == c_bit {
                        return T::zero();
                    }
                    // (0,1) -> -1, (1,0) -> 1
                    if r_bit == 0 {
                        final_sign *= -1;
                    }
                }
            }
        } else {
            // Later slots: I (identity)
            if r_bit != c_bit {
                return T::zero();
            }
        }
    }

    if final_sign > 0 { T::one() } else { -T::one() }
}

/// Helper to create a tensor from linear data.
///
/// Replaces missing `from_data` using `from_shape_fn`.
pub fn from_data_helper<B, T>(data: &[T], shape: &[usize]) -> B::Tensor<T>
where
    B: TensorBackend,
    T: TensorData + Clone,
{
    B::from_shape_fn(shape, |idx| {
        // Compute linear index for row-major layout
        let mut linear_idx = 0;
        let mut stride = 1;

        // Iterate reversed to calculate offset
        for (i, &dim) in shape.iter().enumerate().rev() {
            linear_idx += idx[i] * stride;
            stride *= dim;
        }

        if linear_idx < data.len() {
            data[linear_idx]
        } else {
            T::zero() // Should not happen if shape matches data
        }
    })
}
