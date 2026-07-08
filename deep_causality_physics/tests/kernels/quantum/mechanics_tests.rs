/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    apply_gate_kernel, born_probability_kernel, commutator_kernel, expectation_value_kernel,
    fidelity_kernel, haruna_cz_gate_kernel, haruna_hadamard_gate_kernel, haruna_s_gate_kernel,
    haruna_t_gate_kernel, haruna_x_gate_kernel, haruna_z_gate_kernel, klein_gordon_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple manifold for Klein-Gordon test with a unit-edge
// metric attached (required by R4.5: `klein_gordon_kernel` calls
// `manifold.laplacian()` which now requires a metric).
fn create_simple_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let initial_data = vec![1.0; num_simplices];
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

// Helper to create a normalized quantum state
fn create_test_state() -> HilbertState<f64> {
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
    HilbertState::<f64>::from_multivector(mv)
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
fn test_klein_gordon_kernel_valid() {
    // This manifold has vertices + edges + faces.
    // Previously caused shape mismatch, now should work by slicing.
    let manifold = create_simple_manifold();
    let mass = 1.0;

    let result = klein_gordon_kernel(&manifold, mass);
    assert!(
        result.is_ok(),
        "Klein-Gordon kernel failed: {:?}",
        result.err()
    );
}

#[test]
fn test_klein_gordon_kernel_nan_mass() {
    let manifold = create_simple_manifold();
    let mass = f64::NAN;
    let result = klein_gordon_kernel(&manifold, mass);
    assert!(result.is_err());
}

#[test]
fn test_klein_gordon_kernel_inf_mass() {
    let manifold = create_simple_manifold();
    let mass = f64::INFINITY;
    let result = klein_gordon_kernel(&manifold, mass);
    assert!(result.is_err());
}

// Builds a triangular manifold whose stored field data is supplied by the
// caller, so non-finite or oversized vertex values can be injected to
// exercise the Klein-Gordon finiteness guards.
fn create_manifold_with_data(data: Vec<f64>) -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    assert_eq!(data.len(), num_simplices);
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_klein_gordon_kernel_nonfinite_laplacian() {
    // NaN vertex data propagates through the exterior derivative into the
    // Hodge-Laplacian, tripping the "Laplacian contains non-finite entries"
    // guard (mechanics.rs:37-41).
    let manifold = create_manifold_with_data(vec![f64::NAN; 7]);
    let result = klein_gordon_kernel(&manifold, 1.0);
    assert!(result.is_err());
}

#[test]
fn test_klein_gordon_kernel_m2_psi_overflow() {
    // Finite-but-huge uniform vertex data combined with a huge (finite) mass
    // makes m^2 * psi overflow to +inf, tripping the "m^2 * psi produced
    // non-finite entries" guard (mechanics.rs:66-70). A uniform field yields a
    // finite (~0) laplacian, so the earlier laplacian guard does not fire, and
    // m^2 = (1e60)^2 = 1e120 is finite, so the m^2 guard does not fire either.
    // 1e120 * 1e200 = 1e320 = +inf (> f64::MAX ~ 1.8e308) trips line 66-70.
    let manifold = create_manifold_with_data(vec![1e200; 7]);
    let result = klein_gordon_kernel(&manifold, 1e60);
    assert!(result.is_err());
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
fn test_born_probability_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let basis_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = born_probability_kernel(&state, &basis_wrong);
    assert!(result.is_err());
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
    let state2 = HilbertState::<f64>::from_multivector(mv2);

    let result = born_probability_kernel(&state1, &state2);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        p < 0.01,
        "Orthogonal states should have ~0 overlap, got {}",
        p
    );
}

#[test]
fn test_born_probability_kernel_nonfinite() {
    // A state with enormous amplitudes makes |<basis|state>|^2 overflow to
    // +inf, tripping the "Born probability is not finite" guard
    // (mechanics.rs:101-105).
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let basis = HilbertState::<f64>::from_multivector(mv);

    let result = born_probability_kernel(&state, &basis);
    assert!(result.is_err());
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

#[test]
fn test_expectation_value_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let operator_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = expectation_value_kernel(&state, &operator_wrong);
    assert!(result.is_err());
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

#[test]
fn test_apply_gate_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let gate_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = apply_gate_kernel(&state, &gate_wrong);
    assert!(result.is_err());
}

#[test]
fn test_apply_gate_kernel_nonfinite() {
    // A gate and state with enormous amplitudes make the geometric product
    // overflow to non-finite components, tripping the "Non-finite component in
    // state after gate application" guard (mechanics.rs:158-166).
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let gate = HilbertState::<f64>::from_multivector(mv);

    let result = apply_gate_kernel(&state, &gate);
    assert!(result.is_err());
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
fn test_commutator_kernel_dimension_error() {
    let op_a = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let op_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = commutator_kernel(&op_a, &op_wrong);
    assert!(result.is_err());
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
fn test_haruna_hadamard_gate_kernel_dimension_error() {
    let field_a = create_real_field();
    let field_wrong = CausalMultiVector::new(vec![1.0, 0.0], Metric::Euclidean(1)).unwrap();
    let result = haruna_hadamard_gate_kernel(&field_a, &field_wrong);
    assert!(result.is_err());
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

// NOTE on three defensively-unreachable Klein-Gordon guards
// (`klein_gordon_kernel`):
//   * mechanics.rs:53-55 — "psi_data is smaller than laplacian data".
//     `vertex_count == laplacian.len()` is the number of 0-simplices (vertices),
//     and `Manifold` stores one datum per simplex, so `psi_data.len() ==
//     total_simplices >= vertex_count` always. The guard never fires.
//   * mechanics.rs:59-61 — "psi data contains non-finite entries". To reach
//     this, the *laplacian* (line 37 guard) must first be finite. But the
//     Hodge-Laplacian of a 0-form is built from the vertex values; a non-finite
//     vertex value propagates into the laplacian and is caught by the earlier
//     line-37 guard, so a finite laplacian implies finite vertex psi. The two
//     conditions are mutually exclusive.
//   * mechanics.rs:74-76 — "Klein-Gordon result contains non-finite entries"
//     (`laplacian + m2_psi`). Both summands are already guaranteed finite (lines
//     37 and 66). The sum overflows to ±inf only if both summands are ~1e308;
//     but a uniform field (which keeps m2_psi large) yields laplacian ≈ 0, and a
//     non-uniform field large enough to push the laplacian near 1e308 overflows
//     the laplacian first (caught at line 37). No data configuration makes both
//     summands simultaneously near MAX, so the result-overflow guard is
//     unreachable. An offline scan over magnitudes 1e150..1e308 confirmed only
//     the laplacian and m2_psi guards ever fire.
