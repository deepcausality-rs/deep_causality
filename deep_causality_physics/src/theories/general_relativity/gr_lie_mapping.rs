/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lie Algebra ↔ Geometric Tensor Mapping for GR
//!
//! The Lorentz gauge group SO(3,1) has 6 generators, but the geometric Riemann
//! tensor R^ρ_σμν naturally has 4⁴ = 256 components (with 20 independent due to symmetries).
//!
//! This module provides bidirectional mapping between:
//! - **Lie-algebra storage**: `[points, 4, 4, 6]` (used by `GaugeField`)
//! - **Geometric storage**: `[points, 4, 4, 4, 4]` (natural Riemann representation)
//!
//! # Generator Ordering (Lorentz group)
//!
//! | Lie Index | Antisymmetric Pair (μ,ν) | Generator |
//! |-----------|--------------------------|-----------|
//! | 0         | (0, 1)                   | K₁ (boost)|
//! | 1         | (0, 2)                   | K₂ (boost)|
//! | 2         | (0, 3)                   | K₃ (boost)|
//! | 3         | (1, 2)                   | J₃ (rotation)|
//! | 4         | (1, 3)                   | J₂ (rotation)|
//! | 5         | (2, 3)                   | J₁ (rotation)|
use crate::PhysicsError;
use deep_causality_num::{Field, Float};
use deep_causality_tensor::CausalTensor;

/// Maps antisymmetric pair (μ, ν) to Lie algebra index.
///
/// # Ordering
/// (0,1)→0, (0,2)→1, (0,3)→2, (1,2)→3, (1,3)→4, (2,3)→5
///
/// Returns `None` for diagonal (μ=ν) or reverse order (μ>ν).
#[inline]
pub fn pair_to_lie_index(mu: usize, nu: usize) -> Option<usize> {
    if mu >= nu || mu >= 4 || nu >= 4 {
        return None;
    }
    // Triangular indexing: pairs are (0,1), (0,2), (0,3), (1,2), (1,3), (2,3)
    // Index = mu * (7 - mu) / 2 + nu - mu - 1 (simplified formula)
    let idx = match (mu, nu) {
        (0, 1) => 0,
        (0, 2) => 1,
        (0, 3) => 2,
        (1, 2) => 3,
        (1, 3) => 4,
        (2, 3) => 5,
        _ => return None,
    };
    Some(idx)
}

/// Maps Lie algebra index to antisymmetric pair (μ, ν).
///
/// Returns `(mu, nu)` where `mu < nu`.
#[inline]
pub fn lie_index_to_pair(lie_idx: usize) -> Option<(usize, usize)> {
    match lie_idx {
        0 => Some((0, 1)),
        1 => Some((0, 2)),
        2 => Some((0, 3)),
        3 => Some((1, 2)),
        4 => Some((1, 3)),
        5 => Some((2, 3)),
        _ => None,
    }
}

/// Expands Lie-algebra field strength to geometric Riemann tensor.
///
/// # Mathematical Mapping
///
/// ```text
/// R^ρ_σμν = field_strength[p, ρ, σ, lie_index(μ, ν)]   for μ < ν
/// R^ρ_σνμ = -R^ρ_σμν                                    (antisymmetry)
/// R^ρ_σμμ = 0                                           (diagonal)
/// ```
///
/// # Input Shapes
/// - `[4, 4, 6]` → single-point Lie storage
/// - `[N, 4, 4, 6]` → multi-point Lie storage
///
/// # Output Shapes
/// - `[4, 4, 4, 4]` → single-point geometric Riemann
/// - `[N, 4, 4, 4, 4]` → multi-point geometric Riemann
pub fn expand_lie_to_riemann<T>(lie_fs: &CausalTensor<T>) -> Result<CausalTensor<T>, PhysicsError>
where
    T: Field + Float + Copy,
{
    let shape = lie_fs.shape();

    // Validate input shape
    if shape.len() < 3 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Expected at least 3D tensor, got shape {:?}",
            shape
        )));
    }

    let lie_data = lie_fs.as_slice();
    let dim = 4usize;
    let _lie_dim = 6usize;

    // Determine number of points and strides based on shape
    let (num_points, elem_per_lie_point, rho_stride, sigma_stride) = if shape.len() == 3 {
        // [4, 4, 6] - single point
        (
            1,
            shape[0] * shape[1] * shape[2],
            shape[1] * shape[2],
            shape[2],
        )
    } else {
        // [N, 4, 4, 6] - multi-point
        let elem = shape[1] * shape[2] * shape[3];
        (shape[0], elem, shape[2] * shape[3], shape[3])
    };

    // Output: [N, 4, 4, 4, 4] or [4, 4, 4, 4]
    let elem_per_riemann_point = dim * dim * dim * dim; // 256
    let mut riemann = vec![T::zero(); num_points * elem_per_riemann_point];

    // Helper to index into output Riemann[p, rho, sigma, mu, nu]
    let riemann_idx = |p: usize, rho: usize, sigma: usize, mu: usize, nu: usize| {
        p * elem_per_riemann_point + ((rho * dim + sigma) * dim + mu) * dim + nu
    };

    for p in 0..num_points {
        let point_offset = p * elem_per_lie_point;

        for rho in 0..dim {
            for sigma in 0..dim {
                for mu in 0..dim {
                    for nu in 0..dim {
                        let value = if mu == nu {
                            // Diagonal: R^ρ_σμμ = 0
                            T::zero()
                        } else if mu < nu {
                            // Upper triangular: read from Lie storage
                            if let Some(lie_idx) = pair_to_lie_index(mu, nu) {
                                let flat_idx = point_offset
                                    + rho * rho_stride
                                    + sigma * sigma_stride
                                    + lie_idx;
                                lie_data.get(flat_idx).copied().unwrap_or(T::zero())
                            } else {
                                T::zero()
                            }
                        } else {
                            // Lower triangular: antisymmetry R^ρ_σνμ = -R^ρ_σμν
                            if let Some(lie_idx) = pair_to_lie_index(nu, mu) {
                                let flat_idx = point_offset
                                    + rho * rho_stride
                                    + sigma * sigma_stride
                                    + lie_idx;
                                T::zero() - lie_data.get(flat_idx).copied().unwrap_or(T::zero())
                            } else {
                                T::zero()
                            }
                        };

                        riemann[riemann_idx(p, rho, sigma, mu, nu)] = value;
                    }
                }
            }
        }
    }

    // Return shape based on number of points
    let output_shape = if num_points == 1 {
        vec![dim, dim, dim, dim]
    } else {
        vec![num_points, dim, dim, dim, dim]
    };

    Ok(CausalTensor::from_vec(riemann, &output_shape))
}

/// Contracts geometric Riemann `[4, 4, 4, 4]` to Lie-algebra storage `[4, 4, 6]`.
///
/// This is the inverse of `expand_lie_to_riemann`. Only stores the upper-triangular
/// (μ < ν) components since the tensor is antisymmetric in the last two indices.
pub fn contract_riemann_to_lie<T>(
    riemann: &CausalTensor<T>,
) -> Result<CausalTensor<T>, PhysicsError>
where
    T: Field + Float + Copy,
{
    let shape = riemann.shape();

    if shape != [4, 4, 4, 4] {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Expected [4, 4, 4, 4] Riemann tensor, got {:?}",
            shape
        )));
    }

    let r_data = riemann.as_slice();
    let dim = 4usize;
    let lie_dim = 6usize;

    // Output: [4, 4, 6] = 96 elements
    let mut lie_fs = vec![T::zero(); dim * dim * lie_dim];

    // Helper to index Riemann[rho, sigma, mu, nu]
    let riemann_idx = |rho: usize, sigma: usize, mu: usize, nu: usize| {
        ((rho * dim + sigma) * dim + mu) * dim + nu
    };

    for rho in 0..dim {
        for sigma in 0..dim {
            for lie_idx in 0..lie_dim {
                if let Some((mu, nu)) = lie_index_to_pair(lie_idx) {
                    let r_value = r_data[riemann_idx(rho, sigma, mu, nu)];
                    // lie_fs[rho, sigma, lie_idx]
                    lie_fs[rho * dim * lie_dim + sigma * lie_dim + lie_idx] = r_value;
                }
            }
        }
    }

    Ok(CausalTensor::from_vec(lie_fs, &[dim, dim, lie_dim]))
}
