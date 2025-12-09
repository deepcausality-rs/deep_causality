/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    apply_gate_kernel, born_probability_kernel, commutator_kernel, expectation_value_kernel,
    fidelity_kernel, haruna_cz_gate_kernel, haruna_hadamard_gate_kernel, haruna_s_gate_kernel,
    haruna_t_gate_kernel, haruna_x_gate_kernel, haruna_z_gate_kernel, klein_gordon_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Helper to create a simple manifold for Klein-Gordon test
fn create_simple_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let initial_data = vec![1.0; num_simplices];
    Manifold::new(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        0,
    )
    .unwrap()
}

// Helper to create a normalized quantum state
fn create_test_state() -> HilbertState {
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    HilbertState::from_multivector(mv)
}

// Helper to create a real-valued multivector field
fn create_real_field() -> CausalMultiVector<f64> {
    CausalMultiVector::new(
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap()
}

// =============================================================================
// Klein-Gordon Kernel Tests
// =============================================================================

// NOTE: The klein_gordon_kernel has a known shape mismatch issue:
// - laplacian(0) returns tensor of shape [num_vertices]
// - psi_manifold.data() returns tensor of shape [total_simplices]
// When the manifold has edges/faces, these shapes differ causing panic on addition.
// This test documents the current behavior.

#[test]
#[should_panic(expected = "ShapeMismatch")]
fn test_klein_gordon_kernel_panics_on_shape_mismatch() {
    // This manifold has vertices + edges + faces, causing shape mismatch
    let manifold = create_simple_manifold();
    let mass = 1.0;
    // This will panic because laplacian(0) returns [3] vertices
    // but psi_data is [total_simplices] which is larger
    let _ = klein_gordon_kernel(&manifold, mass);
}

// =============================================================================
// Born Probability Kernel Tests
// =============================================================================

#[test]
fn test_born_probability_kernel_normalized() {
    let state = create_test_state();
    let basis = create_test_state();

    let result = born_probability_kernel(&state, &basis);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&p),
        "Probability must be in [0,1], got {}",
        p
    );
}

#[test]
fn test_born_probability_kernel_orthogonal() {
    let state1 = create_test_state(); // |0>

    // Create orthogonal state |1> (e1 component)
    let data2 = vec![
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv2 = CausalMultiVector::new(data2, Metric::Euclidean(3)).unwrap();
    let state2 = HilbertState::from_multivector(mv2);

    let result = born_probability_kernel(&state1, &state2);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        p < 0.01,
        "Orthogonal states should have ~0 overlap, got {}",
        p
    );
}

// =============================================================================
// Expectation Value Kernel Tests
// =============================================================================

#[test]
fn test_expectation_value_kernel_valid() {
    let state = create_test_state();
    let operator = create_test_state(); // Use state as simple operator

    let result = expectation_value_kernel(&state, &operator);
    assert!(result.is_ok());
}

// =============================================================================
// Apply Gate Kernel Tests
// =============================================================================

#[test]
fn test_apply_gate_kernel_identity() {
    let state = create_test_state();
    let gate = create_test_state(); // Identity-like operation

    let result = apply_gate_kernel(&state, &gate);
    assert!(result.is_ok());
}

// =============================================================================
// Commutator Kernel Tests
// =============================================================================

#[test]
fn test_commutator_kernel_valid() {
    let op_a = create_test_state();
    let op_b = create_test_state();

    let result = commutator_kernel(&op_a, &op_b);
    assert!(result.is_ok());
}

#[test]
fn test_commutator_kernel_self_is_zero() {
    let op_a = create_test_state();

    // [A, A] = 0
    let result = commutator_kernel(&op_a, &op_a).unwrap();

    // Scalar part should be close to zero
    let scalar = result.mv().data()[0];
    assert!(
        scalar.norm() < 1e-10,
        "Commutator [A,A] should be zero, got norm {}",
        scalar.norm()
    );
}

// =============================================================================
// Fidelity Kernel Tests
// =============================================================================

#[test]
fn test_fidelity_kernel_identical_states() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let result = fidelity_kernel(&ideal, &actual);
    assert!(result.is_ok());

    let f = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&f),
        "Fidelity must be in [0,1], got {}",
        f
    );
}

// =============================================================================
// Haruna Gate Kernel Tests
// =============================================================================

#[test]
fn test_haruna_s_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_s_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_z_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_z_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_x_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_x_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_kernel_valid() {
    let field_a = create_real_field();
    let field_b = create_real_field();
    let result = haruna_hadamard_gate_kernel(&field_a, &field_b);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_cz_gate_kernel_valid() {
    let field_a1 = create_real_field();
    let field_a2 = create_real_field();
    let result = haruna_cz_gate_kernel(&field_a1, &field_a2);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_t_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_t_gate_kernel(&field);
    assert!(result.is_ok());
}
