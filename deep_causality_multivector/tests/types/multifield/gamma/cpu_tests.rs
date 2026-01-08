/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for CpuGammaLoader implementing BackendGamma trait.

#![allow(clippy::needless_range_loop)] // Matrix indexing is clearer with explicit loops

use deep_causality_metric::Metric;
use deep_causality_multivector::{BackendGamma, CpuGammaLoader};
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// get_gammas() tests
// =============================================================================

#[test]
fn test_get_gammas_shape_cl2() {
    // Cl(2,0,0): N=2, matrix_dim=2
    let metric = Metric::from_signature(2, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let shape = CpuBackend::shape(&gammas);

    // Shape: [N, matrix_dim, matrix_dim] = [2, 2, 2]
    assert_eq!(shape, vec![2, 2, 2]);
}

#[test]
fn test_get_gammas_shape_cl3() {
    // Cl(3,0,0): N=3, matrix_dim=4
    let metric = Metric::from_signature(3, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let shape = CpuBackend::shape(&gammas);

    // Shape: [N, matrix_dim, matrix_dim] = [3, 4, 4]
    assert_eq!(shape, vec![3, 4, 4]);
}

#[test]
fn test_get_gammas_shape_cl4() {
    // Cl(4,0,0): N=4, matrix_dim=4
    let metric = Metric::from_signature(4, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let shape = CpuBackend::shape(&gammas);

    // Shape: [N, matrix_dim, matrix_dim] = [4, 4, 4]
    assert_eq!(shape, vec![4, 4, 4]);
}

#[test]
fn test_get_gammas_clifford_identity_cl2() {
    // Verify γ_i^2 = +1 for Euclidean Cl(2,0,0)
    let metric = Metric::from_signature(2, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let data: Vec<f32> = CpuBackend::to_vec(&gammas);

    let dim = 2;
    // data layout: [gamma_idx, row, col]
    // Stride: gamma_idx * dim * dim + row * dim + col

    for gamma_idx in 0..2 {
        // Compute γ_i^2
        let mut sq = [[0.0f32; 2]; 2];
        for r in 0..dim {
            for c in 0..dim {
                let mut sum = 0.0f32;
                for k in 0..dim {
                    let idx_rk = gamma_idx * dim * dim + r * dim + k;
                    let idx_kc = gamma_idx * dim * dim + k * dim + c;
                    sum += data[idx_rk] * data[idx_kc];
                }
                sq[r][c] = sum;
            }
        }

        // γ_i^2 = I for Euclidean
        assert!(
            (sq[0][0] - 1.0).abs() < 1e-5,
            "γ_{}^2[0,0] = {} (expected 1.0)",
            gamma_idx,
            sq[0][0]
        );
        assert!(
            (sq[1][1] - 1.0).abs() < 1e-5,
            "γ_{}^2[1,1] = {} (expected 1.0)",
            gamma_idx,
            sq[1][1]
        );
        assert!(
            sq[0][1].abs() < 1e-5,
            "γ_{}^2[0,1] = {} (expected 0.0)",
            gamma_idx,
            sq[0][1]
        );
        assert!(
            sq[1][0].abs() < 1e-5,
            "γ_{}^2[1,0] = {} (expected 0.0)",
            gamma_idx,
            sq[1][0]
        );
    }
}

#[test]
fn test_get_gammas_anticommutator_cl2() {
    // Verify γ_0 γ_1 + γ_1 γ_0 = 0 for Cl(2,0,0)
    let metric = Metric::from_signature(2, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let data: Vec<f32> = CpuBackend::to_vec(&gammas);

    let dim = 2;
    let cell_size = dim * dim; // 4

    // Compute γ_0 * γ_1
    let mut g01 = [[0.0f32; 2]; 2];
    for r in 0..dim {
        for c in 0..dim {
            let mut sum = 0.0f32;
            for k in 0..dim {
                // gamma_0 at offset 0, gamma_1 at offset cell_size
                let idx_0rk = r * dim + k;
                let idx_1kc = cell_size + k * dim + c;
                sum += data[idx_0rk] * data[idx_1kc];
            }
            g01[r][c] = sum;
        }
    }

    // Compute γ_1 * γ_0
    let mut g10 = [[0.0f32; 2]; 2];
    for r in 0..dim {
        for c in 0..dim {
            let mut sum = 0.0f32;
            for k in 0..dim {
                let idx_1rk = cell_size + r * dim + k;
                let idx_0kc = k * dim + c;
                sum += data[idx_1rk] * data[idx_0kc];
            }
            g10[r][c] = sum;
        }
    }

    // Anticommutator = 0
    for r in 0..dim {
        for c in 0..dim {
            let anticomm = g01[r][c] + g10[r][c];
            assert!(
                anticomm.abs() < 1e-5,
                "Anticommutator[{},{}] = {} (expected 0)",
                r,
                c,
                anticomm
            );
        }
    }
}

// =============================================================================
// get_basis_gammas() tests
// =============================================================================

#[test]
fn test_get_basis_gammas_shape_cl2() {
    // Cl(2,0,0): 2^2=4 blades, matrix_dim=2
    let metric = Metric::from_signature(2, 0, 0);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let shape = CpuBackend::shape(&basis);

    assert_eq!(shape, vec![4, 2, 2]);
}

#[test]
fn test_get_basis_gammas_shape_cl3() {
    // Cl(3,0,0): 2^3=8 blades, matrix_dim=4
    let metric = Metric::from_signature(3, 0, 0);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let shape = CpuBackend::shape(&basis);

    assert_eq!(shape, vec![8, 4, 4]);
}

#[test]
fn test_get_basis_gammas_identity_blade_is_identity_matrix() {
    // Blade 0 (empty product) should be identity matrix
    let metric = Metric::from_signature(2, 0, 0);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let data: Vec<f32> = CpuBackend::to_vec(&basis);

    // Blade 0 at indices [0, r, c] with dim=2
    // Check diagonal is 1: data[0] = (0,0), data[3] = (1,1)
    assert!((data[0] - 1.0).abs() < 1e-5); // (0,0)
    assert!((data[3] - 1.0).abs() < 1e-5); // (1,1)

    // Check off-diagonal is 0: data[1] = (0,1), data[2] = (1,0)
    assert!(data[1].abs() < 1e-5); // (0,1)
    assert!(data[2].abs() < 1e-5); // (1,0)
}

#[test]
fn test_get_basis_gammas_blade1_equals_gamma0() {
    // Blade 1 (bit pattern 0b01) = γ_0
    let metric = Metric::from_signature(2, 0, 0);
    let gammas = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_gammas(&metric);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);

    let gamma_data: Vec<f32> = CpuBackend::to_vec(&gammas);
    let basis_data: Vec<f32> = CpuBackend::to_vec(&basis);

    let dim = 2;
    let cell_size = dim * dim;

    for r in 0..dim {
        for c in 0..dim {
            // gamma_0 is at offset 0
            let gamma_idx = r * dim + c;
            // blade 1 is at offset cell_size
            let basis_idx = cell_size + r * dim + c;
            assert!(
                (gamma_data[gamma_idx] - basis_data[basis_idx]).abs() < 1e-5,
                "Blade 1 != γ_0 at ({}, {})",
                r,
                c
            );
        }
    }
}

// =============================================================================
// get_dual_basis_gammas() tests
// =============================================================================

#[test]
fn test_get_dual_basis_gammas_shape_cl2() {
    let metric = Metric::from_signature(2, 0, 0);
    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);
    let shape = CpuBackend::shape(&dual);

    assert_eq!(shape, vec![4, 2, 2]);
}

#[test]
fn test_get_dual_basis_gammas_shape_cl3() {
    let metric = Metric::from_signature(3, 0, 0);
    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);
    let shape = CpuBackend::shape(&dual);

    assert_eq!(shape, vec![8, 4, 4]);
}

#[test]
fn test_get_dual_basis_gammas_identity_blade_dual_is_identity() {
    // Dual of identity blade should be identity
    let metric = Metric::from_signature(2, 0, 0);
    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);
    let data: Vec<f32> = CpuBackend::to_vec(&dual);

    // Blade 0 dual: data[0]=(0,0), data[3]=(1,1) should be 1
    // data[1]=(0,1), data[2]=(1,0) should be 0
    assert!((data[0] - 1.0).abs() < 1e-5);
    assert!((data[3] - 1.0).abs() < 1e-5);
    assert!(data[1].abs() < 1e-5);
    assert!(data[2].abs() < 1e-5);
}

#[test]
fn test_dual_basis_orthogonality_cl2() {
    // Tr(Γ_I Γ_J^{-1}) = D δ_{IJ}
    let metric = Metric::from_signature(2, 0, 0);
    let basis = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_basis_gammas(&metric);
    let dual = <CpuGammaLoader as BackendGamma<CpuBackend, f32>>::get_dual_basis_gammas(&metric);

    let basis_data: Vec<f32> = CpuBackend::to_vec(&basis);
    let dual_data: Vec<f32> = CpuBackend::to_vec(&dual);

    let dim = 2;
    let cell_size = dim * dim;
    let num_blades = 4;

    for i in 0..num_blades {
        for j in 0..num_blades {
            // Compute Tr(Γ_I * Dual_J)
            let mut trace = 0.0f32;
            for r in 0..dim {
                for k in 0..dim {
                    let basis_idx = i * cell_size + r * dim + k;
                    let dual_idx = j * cell_size + r * dim + k;
                    trace += basis_data[basis_idx] * dual_data[dual_idx];
                }
            }

            if i == j {
                assert!(
                    (trace - dim as f32).abs() < 1e-4,
                    "Tr(Γ_{} Γ_{}^{{-1}}) = {} (expected {})",
                    i,
                    j,
                    trace,
                    dim
                );
            } else {
                assert!(
                    trace.abs() < 1e-4,
                    "Tr(Γ_{} Γ_{}^{{-1}}) = {} (expected 0)",
                    i,
                    j,
                    trace
                );
            }
        }
    }
}
