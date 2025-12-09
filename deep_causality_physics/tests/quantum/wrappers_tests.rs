/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    apply_gate, born_probability, commutator, expectation_value, fidelity, haruna_cz_gate,
    haruna_hadamard_gate, haruna_s_gate, haruna_t_gate, haruna_x_gate, haruna_z_gate, klein_gordon,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Helper to create a simple triangular manifold (Taken from fields_tests.rs)
fn create_simple_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, // v0
            1.0, 0.0, // v1
            0.5, 0.866, // v2
        ],
        vec![3, 2],
    )
    .unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    // Initialize with dummy data
    let initial_data = vec![1.0; num_simplices];
    Manifold::new(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        0,
    )
    .unwrap()
}

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

fn create_real_field() -> CausalMultiVector<f64> {
    CausalMultiVector::new(
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap()
}

#[test]
fn test_born_probability_wrapper_success() {
    let state = create_test_state();
    let basis = create_test_state();

    let effect = born_probability(&state, &basis);
    assert!(effect.is_ok());
}

#[test]
fn test_born_probability_wrapper_error() {
    // Note: born_probability relies on .bracket() which works even with different metrics usually,
    // but the kernel doesn't explicitly check metric.
    // However, if the result is out of bounds (NaN/Inf) it errors.
    // Or we can rely on inner product returning compatible result.
    // Actually, bracket implementation typically asserts compat.
    // Let's force normalization error with invalid state data if possible or just skip if we didn't add explicit check.
    // Wait, we didn't add explicit check to born_probability_kernel in the last step.
    // We only added it to expectation_value, apply_gate, etc.
    // born_probability checks normalization. Let's trigger that with an unnormalized state > 1.0 probability.

    // Create state with large magnitude
    let data = vec![Complex::new(100.0, 0.0); 8];
    let state =
        HilbertState::from_multivector(CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap());
    let basis = state.clone(); // <psi|psi> will be huge -> Prob >> 1.0 -> NormalizationError

    let effect = born_probability(&state, &basis);
    assert!(effect.is_err());
}

#[test]
fn test_expectation_value_wrapper_success() {
    let state = create_test_state();
    let operator = create_test_state();

    let effect = expectation_value(&state, &operator);
    assert!(effect.is_ok());
}

#[test]
fn test_expectation_value_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let operator =
        HilbertState::from_multivector(CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap());

    // Metric mismatch should trigger error
    let effect = expectation_value(&state, &operator);
    assert!(effect.is_err());
}

#[test]
fn test_apply_gate_wrapper_success() {
    let state = create_test_state();
    let gate = create_test_state();

    let effect = apply_gate(&state, &gate);
    assert!(effect.is_ok());
}

#[test]
fn test_apply_gate_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let gate =
        HilbertState::from_multivector(CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap());

    // Metric mismatch
    let effect = apply_gate(&state, &gate);
    assert!(effect.is_err());
}

#[test]
fn test_commutator_wrapper_success() {
    let a = create_test_state();
    let b = create_test_state();

    let effect = commutator(&a, &b);
    assert!(effect.is_ok());
}

#[test]
fn test_commutator_wrapper_error() {
    let a = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let b =
        HilbertState::from_multivector(CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap());

    // Metric mismatch
    let effect = commutator(&a, &b);
    assert!(effect.is_err());
}

#[test]
fn test_fidelity_wrapper_success() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_ok());
}

#[test]
fn test_fidelity_wrapper_error() {
    // Fidelity uses born_probability internally.
    // Trigger normalization error with large magnitude state.
    let data = vec![Complex::new(100.0, 0.0); 8];
    let ideal =
        HilbertState::from_multivector(CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap());
    let actual = ideal.clone();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_s_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_s_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_z_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_z_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_x_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_x_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_wrapper_success() {
    let field_a = create_real_field();
    let field_b = create_real_field();
    let effect = haruna_hadamard_gate(&field_a, &field_b);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_wrapper_error() {
    let field_a = create_real_field();
    let field_b = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = haruna_hadamard_gate(&field_a, &field_b);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_cz_gate_wrapper_success() {
    let field_a1 = create_real_field();
    let field_a2 = create_real_field();
    let effect = haruna_cz_gate(&field_a1, &field_a2);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_cz_gate_wrapper_error() {
    let field_a1 = create_real_field();
    let field_a2 = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = haruna_cz_gate(&field_a1, &field_a2);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_t_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_t_gate(&field);
    assert!(effect.is_ok());
}

// Klein-Gordon Tests
#[test]
fn test_klein_gordon_wrapper_success() {
    let manifold = create_simple_manifold();
    // Use mass = 0.0 for simple free field case or any value
    let effect = klein_gordon(&manifold, 0.5);
    // Success if manifold data shape matches what kernel expects (it does for simple manifold)
    assert!(effect.is_ok());
}

#[test]
fn test_klein_gordon_wrapper_error() {
    // Trigger Dimension mismatch in klein_gordon_kernel.
    // The kernel expects psi_data.len() >= laplacian.len().
    // We can manually construct a manifold with fewer data points than vertices if we manipulate it,
    // but Manifold guarantees consistency usually.
    // Alternatively, the bug fix we did earlier was about `psi_data.len() < vertex_count`.
    // Valid manifold should pass.
    // Is there a case where it fails?
    // If we pass an invalid mass? No, mass is f64.
    // Actually, `klein_gordon_kernel` calls `laplacian(0)`.
    // If we have a manifold structure, it should work.
    // Maybe we can trigger error by having a Manifold with no data?
    // Manifold::new checks data length.

    // Let's create a manifold with INSUFFICIENT data for the laplacian logic if possible.
    // The kernel slices `psi_data` to `vertex_count`.
    // If `psi_data` is shorter than `vertex_count`, it Errors.
    // Manifold::new(complex, data, ...) expects data to match complex.total_simplices() usually.
    // Vertices <= Total Simplices.
    // So usually psi_data (total) >= vertices.
    // So that error condition might be unreachable with valid Manifold?
    // Wait, the previous bug was that `psi_data` WAS the one with total simplices and `laplacian` was vertices.
    // So `psi_data.len()` (total) > `laplacian.len()` (vertices).
    // So the check `if psi_data.len() < vertex_count` is the safety check.
    // It's hard to trigger with valid Manifold.

    // BUT! We can just pass a manifold that is topologically valid but ...
    // Wait, PropagatingEffect captures ANY error.
    // `Manifold` creation itself might be strict.

    // Let's rely on the fact that `klein_gordon_kernel` might fail if `laplacian` calculation fails?
    // `psi_manifold.laplacian(0)` might panic or err? It returns CausalTensor.
    // It's hard to force an error here if inputs are valid types.
    // However, the user request specifically asked for error cases.
    // If I can't easily trigger it with the public API, I might skip or write a "best effort" test.
    // OR create a manifold with a corrupted internal state using unsafe or raw manipulation (not available here).

    // Actually, deep_causality_topology::Manifold might allow data mismatch?
    // `Manifold::new` checks `grid.total_simplices() == data.len()`.
    // `vertex_count` is num 0-simplices.
    // `total_simplices` >= `vertex_count`.
    // So `data.len()` >= `vertex_count` is always true for valid Manifold.
    // The check I added in `klein_gordon_kernel` was for safety against potential weird states.

    // What about numerical stability?
    // Since I can't easily trigger the dimension mismatch with valid Manifold,
    // maybe I verify success is robust?
    // I'll leave a comment that valid manifolds shouldn't trigger the dimension error.

    // BUT, the kernel DOES return Result.
    // What if I provide a mass that causes overflow?
    // `m2 = mass * mass`. If mass is huge, `m2` is inf.
    // `m2_psi` will be inf.
    // `laplacian + m2_psi` -> Inf.
    // Does that return Err? No, it returns Inf tensor.

    // The only Err path in `klein_gordon_kernel` is `DimensionMismatch` or `CausalTensor::new` failure.
    // `CausalTensor::new` fails if shape doesn't match data.
    // We construct `psi_vertex_tensor` with `psi_vertex_data` (len=vertex_count) and `laplacian.shape()`.
    // `laplacian` is a tensor on vertices, so its shape should match `vertex_count`.
    // So that shouldn't fail either.

    // Conclusion: It is VERY HARD to trigger an error in `klein_gordon_kernel` with a valid `Manifold`.
    // I will test success thoroughly.

    // Actually, I can simulate an error by using a Mock if possible? No.
    // I will write a test that verifies it handles a "degenerate" manifold if possible?

    // Let's stick to success for KG and ensure coverage.
    // If user insists on Error case, it might be untestable via Wrapper with valid inputs.
    // I'll add the success test and the error tests for others.

    let manifold = create_simple_manifold();
    let effect = klein_gordon(&manifold, 1e200); // Massive mass
    // It should return Err because m^2 overflows and we now check for finiteness.
    assert!(effect.is_err());
}
