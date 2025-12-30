/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for gamma module functions: compute_gamma_element and from_data_helper.

#![allow(clippy::needless_range_loop)]

use deep_causality_metric::Metric;
use deep_causality_multivector::{compute_gamma_element, from_data_helper};
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// compute_gamma_element() tests
// =============================================================================

#[test]
fn test_compute_gamma_element_returns_zero_for_out_of_bounds_row() {
    let metric = Metric::from_signature(3, 0, 0);
    let matrix_dim = 4; // 2^ceil(3/2) = 4

    // Row out of bounds
    let result: f32 = compute_gamma_element(0, matrix_dim, 0, &metric);
    assert_eq!(result, 0.0);
}

#[test]
fn test_compute_gamma_element_returns_zero_for_out_of_bounds_col() {
    let metric = Metric::from_signature(3, 0, 0);
    let matrix_dim = 4;

    // Column out of bounds
    let result: f32 = compute_gamma_element(0, 0, matrix_dim, &metric);
    assert_eq!(result, 0.0);
}

#[test]
fn test_compute_gamma_element_gamma0_structure() {
    // For Cl(2,0,0), gamma_0 should be sigma_x in slot 0
    let metric = Metric::from_signature(2, 0, 0);

    // sigma_x = [[0, 1], [1, 0]]
    let g00: f32 = compute_gamma_element(0, 0, 0, &metric);
    let g01: f32 = compute_gamma_element(0, 0, 1, &metric);
    let g10: f32 = compute_gamma_element(0, 1, 0, &metric);
    let g11: f32 = compute_gamma_element(0, 1, 1, &metric);

    // sigma_x has zeros on diagonal, ones off-diagonal
    assert_eq!(g00, 0.0);
    assert_eq!(g11, 0.0);
    assert_eq!(g01, 1.0);
    assert_eq!(g10, 1.0);
}

#[test]
fn test_compute_gamma_element_gamma1_structure() {
    // For Cl(2,0,0), gamma_1 should be σ_z in the active slot (Euclidean - squares to +1)
    let metric = Metric::from_signature(2, 0, 0);

    // σ_z = [[1, 0], [0, -1]]
    let g00: f32 = compute_gamma_element(1, 0, 0, &metric);
    let g01: f32 = compute_gamma_element(1, 0, 1, &metric);
    let g10: f32 = compute_gamma_element(1, 1, 0, &metric);
    let g11: f32 = compute_gamma_element(1, 1, 1, &metric);

    assert_eq!(g00, 1.0); // diagonal (0,0) = +1
    assert_eq!(g11, -1.0); // diagonal (1,1) = -1
    assert_eq!(g01, 0.0); // off-diagonal = 0
    assert_eq!(g10, 0.0); // off-diagonal = 0
}

#[test]
fn test_compute_gamma_element_clifford_relation_cl2() {
    // Verify γ_i γ_j + γ_j γ_i = 2δ_{ij}I for Cl(2,0,0)
    let metric = Metric::from_signature(2, 0, 0);
    let dim = 2;

    // Compute γ_0 * γ_0 + γ_0 * γ_0 = 2 * I (should equal 2*I)
    let mut gamma0_sq = [[0.0f32; 2]; 2];
    for r in 0..dim {
        for c in 0..dim {
            let mut sum = 0.0f32;
            for k in 0..dim {
                let g0_rk: f32 = compute_gamma_element(0, r, k, &metric);
                let g0_kc: f32 = compute_gamma_element(0, k, c, &metric);
                sum += g0_rk * g0_kc;
            }
            gamma0_sq[r][c] = sum;
        }
    }

    // γ_0^2 = I for Euclidean signature
    assert!((gamma0_sq[0][0] - 1.0).abs() < 1e-6);
    assert!((gamma0_sq[1][1] - 1.0).abs() < 1e-6);
    assert!(gamma0_sq[0][1].abs() < 1e-6);
    assert!(gamma0_sq[1][0].abs() < 1e-6);
}

#[test]
fn test_compute_gamma_element_anticommutator_cl2() {
    // Verify γ_0 γ_1 + γ_1 γ_0 = 0 for i ≠ j in Cl(2,0,0)
    let metric = Metric::from_signature(2, 0, 0);
    let dim = 2;

    // Compute γ_0 * γ_1
    let mut g01 = [[0.0f32; 2]; 2];
    for r in 0..dim {
        for c in 0..dim {
            let mut sum = 0.0f32;
            for k in 0..dim {
                let g0_rk: f32 = compute_gamma_element(0, r, k, &metric);
                let g1_kc: f32 = compute_gamma_element(1, k, c, &metric);
                sum += g0_rk * g1_kc;
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
                let g1_rk: f32 = compute_gamma_element(1, r, k, &metric);
                let g0_kc: f32 = compute_gamma_element(0, k, c, &metric);
                sum += g1_rk * g0_kc;
            }
            g10[r][c] = sum;
        }
    }

    // γ_0 γ_1 + γ_1 γ_0 = 0
    for r in 0..dim {
        for c in 0..dim {
            assert!(
                (g01[r][c] + g10[r][c]).abs() < 1e-6,
                "Anticommutator failed at ({}, {}): {} + {} = {}",
                r,
                c,
                g01[r][c],
                g10[r][c],
                g01[r][c] + g10[r][c]
            );
        }
    }
}

#[test]
fn test_compute_gamma_element_cl3_gamma0() {
    // For Cl(3,0,0), matrix_dim = 4
    let metric = Metric::from_signature(3, 0, 0);

    // Gamma_0 should have non-zero elements
    let mut has_nonzero = false;
    for r in 0..4 {
        for c in 0..4 {
            let val: f32 = compute_gamma_element(0, r, c, &metric);
            if val.abs() > 1e-6 {
                has_nonzero = true;
            }
        }
    }
    assert!(has_nonzero, "Gamma_0 should have non-zero elements");
}

// =============================================================================
// from_data_helper() tests
// =============================================================================

#[test]
fn test_from_data_helper_creates_tensor_with_correct_shape() {
    let data = vec![1.0f32, 2.0, 3.0, 4.0, 5.0, 6.0];
    let shape = [2, 3];

    let tensor = from_data_helper::<CpuBackend, f32>(&data, &shape);
    let result_shape = CpuBackend::shape(&tensor);

    assert_eq!(result_shape, vec![2, 3]);
}

#[test]
fn test_from_data_helper_preserves_values() {
    let data = vec![1.0f32, 2.0, 3.0, 4.0];
    let shape = [2, 2];

    let tensor = from_data_helper::<CpuBackend, f32>(&data, &shape);
    let result: Vec<f32> = CpuBackend::to_vec(&tensor);

    assert_eq!(result, data);
}

#[test]
fn test_from_data_helper_1d_tensor() {
    let data = vec![1.0f32, 2.0, 3.0];
    let shape = [3];

    let tensor = from_data_helper::<CpuBackend, f32>(&data, &shape);
    let result: Vec<f32> = CpuBackend::to_vec(&tensor);

    assert_eq!(result, data);
}

#[test]
fn test_from_data_helper_3d_tensor() {
    let data: Vec<f32> = (0..24).map(|i| i as f32).collect();
    let shape = [2, 3, 4];

    let tensor = from_data_helper::<CpuBackend, f32>(&data, &shape);
    let result_shape = CpuBackend::shape(&tensor);
    let result: Vec<f32> = CpuBackend::to_vec(&tensor);

    assert_eq!(result_shape, vec![2, 3, 4]);
    assert_eq!(result.len(), 24);
}

#[test]
fn test_from_data_helper_pads_with_zeros_if_data_short() {
    // If data is shorter than shape requires, zeros should fill the rest
    let data = vec![1.0f32, 2.0];
    let shape = [2, 2]; // Requires 4 elements

    let tensor = from_data_helper::<CpuBackend, f32>(&data, &shape);
    let result: Vec<f32> = CpuBackend::to_vec(&tensor);

    assert_eq!(result.len(), 4);
    assert_eq!(result[0], 1.0);
    assert_eq!(result[1], 2.0);
    assert_eq!(result[2], 0.0); // Padded with zero
    assert_eq!(result[3], 0.0); // Padded with zero
}
