/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    all_structure_constants, confinement_potential_kernel, covariant_derivative_kernel,
    gell_mann_matrices, running_coupling_kernel, structure_constant, wilson_loop_kernel,
};

// =============================================================================
// Gell-Mann Matrices Tests
// =============================================================================

#[test]
fn test_gell_mann_matrices_count() {
    let matrices = gell_mann_matrices();
    assert_eq!(matrices.len(), 8, "Should have 8 Gell-Mann matrices");
}

#[test]
fn test_gell_mann_matrices_size() {
    for (i, matrix) in gell_mann_matrices().iter().enumerate() {
        assert_eq!(matrix.len(), 9, "Matrix {} should have 9 elements (3x3)", i);
    }
}

#[test]
fn test_gell_mann_lambda3_diagonal() {
    // λ_3 = diag(1, -1, 0)
    let matrices = gell_mann_matrices();
    let l3 = matrices[2]; // Index 2 for λ_3

    assert!((l3[0] - 1.0).abs() < 1e-10, "λ_3[0,0] should be 1");
    assert!((l3[4] - (-1.0)).abs() < 1e-10, "λ_3[1,1] should be -1");
    assert!((l3[8] - 0.0).abs() < 1e-10, "λ_3[2,2] should be 0");
}

#[test]
fn test_gell_mann_lambda8_diagonal() {
    // λ_8 = diag(1, 1, -2) / sqrt(3)
    let matrices = gell_mann_matrices();
    let l8 = matrices[7]; // Index 7 for λ_8

    let inv_sqrt3 = 1.0 / 3.0_f64.sqrt();
    assert!((l8[0] - inv_sqrt3).abs() < 1e-10);
    assert!((l8[4] - inv_sqrt3).abs() < 1e-10);
    assert!((l8[8] - (-2.0 * inv_sqrt3)).abs() < 1e-10);
}

// =============================================================================
// Structure Constants Tests
// =============================================================================

#[test]
fn test_structure_constant_f123() {
    // f^123 = 1
    assert!((structure_constant(1, 2, 3) - 1.0).abs() < 1e-10);
}

#[test]
fn test_structure_constant_antisymmetry() {
    // f^abc = -f^bac
    let f123 = structure_constant(1, 2, 3);
    let f213 = structure_constant(2, 1, 3);
    assert!(
        (f123 + f213).abs() < 1e-10,
        "Should be antisymmetric: f123={}, f213={}",
        f123,
        f213
    );
}

#[test]
fn test_structure_constant_cyclic() {
    // f^abc = f^bca for cyclic permutations (accounting for antisymmetry)
    let f123 = structure_constant(1, 2, 3);
    let f231 = structure_constant(2, 3, 1);
    let f312 = structure_constant(3, 1, 2);

    // All cyclic permutations should have same magnitude
    assert!((f123.abs() - f231.abs()).abs() < 1e-10);
    assert!((f231.abs() - f312.abs()).abs() < 1e-10);
}

#[test]
fn test_structure_constant_zero_for_invalid() {
    // Non-existent combinations should be 0
    assert_eq!(structure_constant(1, 1, 1), 0.0);
    assert_eq!(structure_constant(1, 1, 2), 0.0);
    assert_eq!(structure_constant(1, 2, 2), 0.0);
}

#[test]
fn test_structure_constant_f458() {
    // f^458 = √3/2
    let sqrt3_half = 3.0_f64.sqrt() * 0.5;
    assert!((structure_constant(4, 5, 8) - sqrt3_half).abs() < 1e-10);
}

#[test]
fn test_all_structure_constants_non_empty() {
    let all = all_structure_constants();
    assert!(!all.is_empty(), "Should have non-zero structure constants");
    assert!(all.len() >= 9, "Should have at least 9 non-zero entries");
}

// =============================================================================
// Covariant Derivative Tests
// =============================================================================

#[test]
fn test_covariant_derivative_pure_gauge() {
    // With zero gluon field, covariant derivative = ordinary derivative
    let psi = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // Unit color state
    let psi_gradient = vec![0.1; 24]; // Some gradient
    let gluon_field = vec![0.0; 32]; // Zero gauge field
    let coupling = 1.0;

    let result = covariant_derivative_kernel(&psi, &psi_gradient, &gluon_field, coupling);
    assert!(result.is_ok());

    let d_psi = result.unwrap();
    // Should match ordinary derivative since A = 0
    for (i, &val) in d_psi.iter().enumerate() {
        assert!(
            (val - psi_gradient[i]).abs() < 1e-10,
            "Index {}: expected {}, got {}",
            i,
            psi_gradient[i],
            val
        );
    }
}

#[test]
fn test_covariant_derivative_dimension_error_psi() {
    let psi = vec![1.0, 0.0]; // Wrong size
    let psi_gradient = vec![0.0; 24];
    let gluon_field = vec![0.0; 32];

    let result = covariant_derivative_kernel(&psi, &psi_gradient, &gluon_field, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_covariant_derivative_dimension_error_gradient() {
    let psi = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let psi_gradient = vec![0.0; 12]; // Wrong size
    let gluon_field = vec![0.0; 32];

    let result = covariant_derivative_kernel(&psi, &psi_gradient, &gluon_field, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_covariant_derivative_dimension_error_gluon() {
    let psi = vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let psi_gradient = vec![0.0; 24];
    let gluon_field = vec![0.0; 16]; // Wrong size

    let result = covariant_derivative_kernel(&psi, &psi_gradient, &gluon_field, 1.0);
    assert!(result.is_err());
}

// =============================================================================
// Wilson Loop Tests
// =============================================================================

#[test]
fn test_wilson_loop_trivial_path() {
    // Empty path should give Tr(I) = 3
    let gluon_values: Vec<f64> = vec![];
    let path_lengths: Vec<f64> = vec![];

    let result = wilson_loop_kernel(&gluon_values, &path_lengths, 1.0);
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.0).abs() < 1e-10);
}

#[test]
fn test_wilson_loop_zero_field() {
    // Zero gluon field should give W ≈ 3
    let gluon_values = vec![0.0; 16]; // 2 segments × 8 colors
    let path_lengths = vec![1.0, 1.0];

    let result = wilson_loop_kernel(&gluon_values, &path_lengths, 1.0);
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.0).abs() < 1e-10);
}

#[test]
fn test_wilson_loop_confinement_decay() {
    // Non-zero field should cause W < 3 (area law suppression)
    let gluon_values = vec![1.0; 8]; // 1 segment, unit field
    let path_lengths = vec![1.0];

    let result = wilson_loop_kernel(&gluon_values, &path_lengths, 1.0);
    assert!(result.is_ok());

    let w = result.unwrap();
    assert!(w < 3.0, "Wilson loop should decay: W = {}", w);
    assert!(w > 0.0, "Wilson loop should remain positive: W = {}", w);
}

#[test]
fn test_wilson_loop_dimension_error() {
    let gluon_values = vec![0.0; 8]; // 1 segment
    let path_lengths = vec![1.0, 2.0]; // 2 segments - mismatch

    let result = wilson_loop_kernel(&gluon_values, &path_lengths, 1.0);
    assert!(result.is_err());
}

// =============================================================================
// Confinement Potential Tests
// =============================================================================

#[test]
fn test_confinement_potential_linear() {
    // V(r) = σr for pure linear confinement
    let sigma = 0.18; // String tension in GeV²
    let r = 2.0;

    let result = confinement_potential_kernel(r, sigma, None);
    assert!(result.is_ok());

    let v = result.unwrap();
    assert!((v - 0.36).abs() < 1e-10, "Expected V = 0.36, got {}", v);
}

#[test]
fn test_confinement_potential_with_coulomb() {
    // V(r) = σr - α/r
    let sigma = 0.18;
    let alpha = 0.3;
    let r = 1.0;

    let result = confinement_potential_kernel(r, sigma, Some(alpha));
    assert!(result.is_ok());

    let v = result.unwrap();
    let expected = sigma * r - alpha / r;
    assert!(
        (v - expected).abs() < 1e-10,
        "Expected V = {}, got {}",
        expected,
        v
    );
}

#[test]
fn test_confinement_potential_zero_distance_error() {
    let result = confinement_potential_kernel(0.0, 0.18, None);
    assert!(result.is_err());
}

#[test]
fn test_confinement_potential_negative_distance_error() {
    let result = confinement_potential_kernel(-1.0, 0.18, None);
    assert!(result.is_err());
}

// =============================================================================
// Running Coupling (Asymptotic Freedom) Tests
// =============================================================================

#[test]
fn test_running_coupling_high_q2() {
    // At high Q², α_s should be small (asymptotic freedom)
    let q2 = 100.0; // 100 GeV²
    let lambda = 0.2; // Λ_QCD ≈ 200 MeV
    let nf = 3; // 3 light flavors

    let result = running_coupling_kernel(q2, lambda, nf);
    assert!(result.is_ok());

    let alpha_s = result.unwrap();
    assert!(alpha_s < 1.0, "α_s should be < 1 at high Q²: {}", alpha_s);
    assert!(alpha_s > 0.0, "α_s should be positive: {}", alpha_s);
}

#[test]
fn test_running_coupling_decreases_with_q2() {
    // α_s(Q1²) > α_s(Q2²) when Q1² < Q2² (asymptotic freedom)
    let lambda = 0.2;
    let nf = 3;

    let alpha_low = running_coupling_kernel(10.0, lambda, nf).unwrap();
    let alpha_high = running_coupling_kernel(100.0, lambda, nf).unwrap();

    assert!(
        alpha_low > alpha_high,
        "α_s should decrease with Q²: α_s(10) = {}, α_s(100) = {}",
        alpha_low,
        alpha_high
    );
}

#[test]
fn test_running_coupling_q2_below_lambda_error() {
    // Q² ≤ Λ² should error (non-perturbative regime)
    let result = running_coupling_kernel(0.01, 0.2, 3);
    assert!(result.is_err());
}

#[test]
fn test_running_coupling_zero_q2_error() {
    let result = running_coupling_kernel(0.0, 0.2, 3);
    assert!(result.is_err());
}

#[test]
fn test_running_coupling_zero_lambda_error() {
    let result = running_coupling_kernel(100.0, 0.0, 3);
    assert!(result.is_err());
}

#[test]
fn test_running_coupling_too_many_flavors_error() {
    // nf = 17 would make b0 = 11 - 34/3 < 0
    let result = running_coupling_kernel(100.0, 0.2, 17);
    assert!(result.is_err());
}
