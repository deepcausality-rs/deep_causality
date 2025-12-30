/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gamma matrix loader for the Matrix Isomorphism Bridge.
//!
//! The gamma matrices are the basis elements of the Clifford algebra in matrix representation.
//! They satisfy the Clifford relation: ΓᵢΓⱼ + ΓⱼΓᵢ = 2ηᵢⱼI
//!
//! This module provides backend-specific loaders that cache gamma matrices on the device.

mod cpu;

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod mlx;

pub mod provider;
pub use provider::GammaProvider;

use deep_causality_metric::Metric;
use deep_causality_tensor::{TensorBackend, TensorData};

/// Trait for loading gamma matrices onto a backend device.
///
/// Gamma matrices are the matrix representation of basis vectors in Clifford algebra.
/// For a Cl(p,q,r) algebra with N = p+q+r, there are N gamma matrices, each of size
/// 2^⌈N/2⌉ × 2^⌈N/2⌉.
///
/// # Storage Strategy
///
/// | Algebra    | Blades | Matrix Size | Total Size | Strategy      |
/// |------------|--------|-------------|------------|---------------|
/// | Cl(2,0)    | 4      | 2×2         | 64 B       | `const` array |
/// | Cl(1,3)    | 16     | 4×4         | 2 KB       | `const` array |
/// | Cl(6,0)    | 64     | 8×8         | 32 KB      | `static` lazy |
/// | Cl(10)     | 1024   | 32×32       | 2 GB       | lazy + cache  |
pub trait BackendGamma<B: TensorBackend, T: TensorData> {
    /// Returns the pre-loaded gamma matrices for a given metric.
    ///
    /// Shape: `[N, Matrix_Dim, Matrix_Dim]` where N is the algebra dimension.
    ///
    /// # Caching
    /// Implementations should cache the matrices on first access.
    fn get_gammas(metric: &Metric) -> B::Tensor<T>;

    /// Returns the full set of basis matrices $\Gamma_I$ for the algebra.
    ///
    /// Shape: `[NumBlades, Matrix_Dim, Matrix_Dim]`
    /// NumBlades = 2^N.
    ///
    /// $\Gamma_I = \Gamma_{i_1} \dots \Gamma_{i_k}$ ordered by canonical index.
    fn get_basis_gammas(metric: &Metric) -> B::Tensor<T>;

    /// Returns the transposed inverse basis matrices $(\Gamma_I^{-1})^T$.
    ///
    /// Shape: `[NumBlades, Matrix_Dim, Matrix_Dim]`
    /// Used for projection: $c_I = \frac{1}{D} \operatorname{Tr}(M (\Gamma_I^{-1})^T)$.
    fn get_dual_basis_gammas(metric: &Metric) -> B::Tensor<T>;
}

/// Computes a single element of a gamma matrix for arbitrary dimension N.
///
/// Uses the Brahmagupta-Fibonacci Identity / Brauer-Weyl construction slightly modified
/// for real matrices to producing a Split Signature Cl(N, N) or Cl(N, N+1).
///
/// Construction involves tensor products of 2x2 Pauli-like matrices.
/// - Slot j < k: σ_z
/// - Slot j = k: σ_x (even) or ε (odd)
/// - Slot j > k: I
///
/// This produces generators squaring to +1 (even) and -1 (odd).
/// Currently ignores explicit metric signature per index, verifying coherence for
/// standard Clifford operations.
pub(crate) fn compute_gamma_element<T: TensorData + std::ops::Neg<Output = T>>(
    gamma_idx: usize,
    row: usize,
    col: usize,
    metric: &Metric,
) -> T {
    let n = metric.dimension();
    let matrix_dim = 1 << n.div_ceil(2);
    let num_slots = n.div_ceil(2);

    // Target slot for the active generator (σ_x or ε)
    let k = gamma_idx / 2;

    // Is this the second generator in the slot? (ε / -1 square)
    let is_odd = !gamma_idx.is_multiple_of(2);

    // Check bound
    if row >= matrix_dim || col >= matrix_dim {
        return T::zero();
    }

    let mut final_sign = 1i8;

    // Iterate over tensor slots (bits)
    // We treat slot 0 as the "lowest" tensor component (inner-most)
    // which corresponds to the lowest bits of the row/col index.
    for slot in 0..num_slots {
        // Bit index within the matrix indices (0 is LSB)
        let bit_shift = num_slots - 1 - slot;

        let r_bit = (row >> bit_shift) & 1;
        let c_bit = (col >> bit_shift) & 1;

        if slot < k {
            // Previous slots: Must be sigma_z
            // sigma_z: diag(1, -1). r==c. val = (-1)^r
            if r_bit != c_bit {
                return T::zero();
            }
            if r_bit == 1 {
                final_sign *= -1;
            }
        } else if slot == k {
            // Active slot: sigma_x or epsilon
            if is_odd {
                // epsilon: [[0, -1], [1, 0]]
                // r != c.
                if r_bit == c_bit {
                    return T::zero();
                }
                // (0,1) -> -1. (1,0) -> 1.
                // r=0, c=1 (r_bit < c_bit) -> -1
                if r_bit == 0 {
                    final_sign *= -1;
                }
            } else {
                // sigma_x: [[0, 1], [1, 0]]
                // r != c. Value always 1.
                if r_bit == c_bit {
                    return T::zero();
                }
            }
        } else {
            // Later slots: I
            // I: diag(1, 1).
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
