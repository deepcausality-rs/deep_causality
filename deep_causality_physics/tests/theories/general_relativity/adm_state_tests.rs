/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for adm_state.rs - ADM State structure and operations

use deep_causality_physics::theories::general_relativity::{AdmOps, AdmState};
use deep_causality_tensor::CausalTensor;
use std::f64::consts::PI;

// ============================================================================
// ADM State Construction and Accessors
// ============================================================================

#[test]
fn test_adm_structures() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma.clone(), k, alpha.clone(), beta.clone(), 0.0);

    // Test hamiltonian constraint interface
    let h = state.hamiltonian_constraint(None).unwrap();
    assert_eq!(h.shape(), &[1]);
    // Expect 0 for flat slice with K=0
    assert_eq!(h.as_slice()[0], 0.0);

    // Test Case 2: Non-zero expansion (K = I)
    // Tr K = 3, K_ij K^ij = 3 => H = 3^2 - 3 = 6
    let k_expanding = CausalTensor::identity(&[3, 3]).unwrap();
    let state_expanding =
        AdmState::new(gamma.clone(), k_expanding, alpha.clone(), beta.clone(), 0.0);
    let h_expanding = state_expanding.hamiltonian_constraint(None).unwrap();
    assert!(
        (h_expanding.as_slice()[0] - 6.0f64).abs() < 1e-10,
        "H should be 6 for isotropic expansion with R=0"
    );

    // Test momentum constraint interface (Expect Error due to missing derivatives)
    assert!(
        state.momentum_constraint(None).is_err(),
        "Momentum constraint should error without spatial derivatives"
    );
}

#[test]
fn test_adm_with_christoffel() {
    // Test that ADM momentum constraint works when Christoffel symbols are provided
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    // Zero Christoffel symbols (flat space)
    let christoffel = CausalTensor::zeros(&[3, 3, 3]);

    let state = AdmState::with_christoffel(
        gamma,
        k,
        alpha,
        beta,
        0.0, // R = 0 for flat
        christoffel,
    );

    // Momentum constraint should now return Ok, not Err
    let m = state.momentum_constraint(None);
    assert!(
        m.is_ok(),
        "Momentum constraint should succeed with Christoffel symbols: {:?}",
        m.err()
    );

    // For flat space with K=0, momentum constraint should be zero
    let m_vec: CausalTensor<f64> = m.unwrap();
    assert_eq!(m_vec.shape(), &[3]);
    for (i, &val) in m_vec.as_slice().iter().enumerate() {
        assert!(
            val.abs() < 1e-12,
            "M_{} should be 0 for flat space with K=0, got {}",
            i,
            val
        );
    }
}

#[test]
fn test_adm_state_default() {
    let state = AdmState::<f64>::default();

    // Default should have identity spatial metric
    let gamma = state.spatial_metric();
    assert_eq!(gamma.shape(), &[3, 3]);

    // Default should have zero extrinsic curvature
    let k = state.extrinsic_curvature();
    assert_eq!(k.shape(), &[3, 3]);

    // Default should have unit lapse
    let alpha = state.lapse();
    assert_eq!(alpha.as_slice()[0], 1.0);

    // Default should have zero shift
    let beta = state.shift();
    assert_eq!(beta.shape(), &[3]);

    // Default should have zero Ricci scalar
    assert_eq!(state.spatial_ricci_scalar(), 0.0);

    // Default should have no Christoffel symbols
    assert!(state.spatial_christoffel().is_none());
}

// ============================================================================
// Mean Curvature
// ============================================================================

#[test]
fn test_adm_mean_curvature_flat() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma, k, alpha, beta, 0.0);
    let mean_k = state.mean_curvature().unwrap();

    assert_eq!(mean_k.shape(), &[1]);
    let k_val: f64 = mean_k.as_slice()[0];
    assert!(k_val.abs() < 1e-12, "Mean curvature should be 0 for K=0");
}

#[test]
fn test_adm_mean_curvature_nonzero() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    // K_ij = diag(1, 2, 3) -> K = γ^ij K_ij = 1 + 2 + 3 = 6
    let k_data = vec![1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0];
    let k = CausalTensor::from_vec(k_data, &[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma, k, alpha, beta, 0.0);
    let mean_k = state.mean_curvature().unwrap();

    let k_val: f64 = mean_k.as_slice()[0];
    assert!(
        (k_val - 6.0).abs() < 1e-10,
        "Mean curvature should be 6, got {}",
        k_val
    );
}

// ============================================================================
// Hamiltonian Constraint
// ============================================================================

#[test]
fn test_adm_hamiltonian_with_matter() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma, k, alpha, beta, 0.0);

    // With matter density ρ = 1
    let rho = CausalTensor::from_vec(vec![1.0], &[1]);
    let h = state.hamiltonian_constraint(Some(&rho)).unwrap();

    // H = R + K² - K_ij K^ij - 16πρ = 0 + 0 - 0 - 16π ≈ -50.265
    let expected = -16.0 * PI;
    assert!(
        (h.as_slice()[0] - expected).abs() < 1e-10,
        "H with matter should be -16π, got {}",
        h.as_slice()[0]
    );
}

#[test]
fn test_adm_hamiltonian_constraint_singular_metric() {
    // Create a singular (zero) spatial metric
    let gamma = CausalTensor::zeros(&[3, 3]);
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma, k, alpha, beta, 0.0);

    let result = state.hamiltonian_constraint(None);
    assert!(result.is_err(), "Singular metric should return error");
}

// ============================================================================
// Momentum Constraint
// ============================================================================

#[test]
fn test_adm_momentum_with_matter() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);
    let christoffel = CausalTensor::zeros(&[3, 3, 3]);

    let state = AdmState::with_christoffel(gamma, k, alpha, beta, 0.0, christoffel);

    // With matter momentum j_i = (1, 0, 0)
    let j = CausalTensor::from_vec(vec![1.0, 0.0, 0.0], &[3]);
    let m = state.momentum_constraint(Some(&j)).unwrap();

    // M_i = connection_terms - 8π j_i
    // For flat space with K=0 and zero Christoffel, M_0 = -8π
    let expected_m0 = -8.0 * PI;
    assert!(
        (m.as_slice()[0] - expected_m0).abs() < 1e-10,
        "M_0 should be -8π, got {}",
        m.as_slice()[0]
    );
    assert!(m.as_slice()[1].abs() < 1e-10, "M_1 should be 0");
    assert!(m.as_slice()[2].abs() < 1e-10, "M_2 should be 0");
}

#[test]
fn test_adm_momentum_constraint_wrong_christoffel_shape() {
    let gamma = CausalTensor::identity(&[3, 3]).unwrap();
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    // Wrong shape: 8 elements instead of 27
    let bad_christoffel = CausalTensor::zeros(&[2, 2, 2]);

    let state = AdmState::with_christoffel(gamma, k, alpha, beta, 0.0, bad_christoffel);

    let result = state.momentum_constraint(None);
    assert!(
        result.is_err(),
        "Wrong Christoffel shape should return error"
    );
}

#[test]
fn test_adm_inverse_spatial_metric_wrong_size() {
    // Create a state with wrong-sized spatial metric (not 9 elements)
    // This requires us to directly manipulate the internal state
    // Since we cannot easily test the private method, we test via hamiltonian_constraint

    // A 4x4 spatial metric (16 elements, not 9)
    let gamma = CausalTensor::from_vec(vec![1.0; 16], &[4, 4]);
    let k = CausalTensor::zeros(&[3, 3]);
    let alpha = CausalTensor::ones(&[1]);
    let beta = CausalTensor::zeros(&[3]);

    let state = AdmState::new(gamma, k, alpha, beta, 0.0);

    let result = state.hamiltonian_constraint(None);
    assert!(
        result.is_err(),
        "Wrong spatial metric size should return error"
    );
}
