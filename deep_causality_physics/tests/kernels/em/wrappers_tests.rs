/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    lorentz_force, lorentz_force_kernel, lorenz_gauge, lorenz_gauge_kernel,
    magnetic_helicity_density, magnetic_helicity_density_kernel, maxwell_gradient,
    maxwell_gradient_kernel, poynting_vector, poynting_vector_kernel, proca_equation,
    proca_equation_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple triangular manifold with a unit-edge metric
// attached. See fields_tests.rs for the rationale (R4.5 widening).
fn create_simple_manifold() -> SimplicialManifold<f64, f64> {
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

// Helper: a purely 1-dimensional complex (vertices + edges only, no faces).
// `exterior_derivative(1)` returns an empty 2-form here because k == max_dim,
// which drives `maxwell_gradient_kernel` into its empty/invalid-2-form error.
fn create_1d_manifold() -> SimplicialManifold<f64, f64> {
    // Three collinear points -> triangulation yields a 1D complex (a path),
    // i.e. 0- and 1-skeletons only, max_dim == 1.
    let points = CausalTensor::new(vec![0.0, 1.0, 2.0], vec![3, 1]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.5).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(vec![1.0; num_simplices], vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

// =============================================================================
// lorentz_force Wrapper Tests
// =============================================================================

#[test]
fn test_lorentz_force_wrapper_success() {
    let j = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = lorentz_force(&j, &b);
    let carried = effect.value_cloned().unwrap();
    let direct = lorentz_force_kernel(&j, &b).unwrap();
    assert_eq!(
        carried.inner().data(),
        direct.data(),
        "lorentz_force must carry the value its kernel produced"
    );
}

#[test]
fn test_lorentz_force_wrapper_error() {
    let j = CausalMultiVector::new(vec![1.0; 8], Metric::Euclidean(3)).unwrap();
    let b = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = lorentz_force(&j, &b);
    assert!(effect.is_err());
}

// =============================================================================
// poynting_vector Wrapper Tests
// =============================================================================

#[test]
fn test_poynting_vector_wrapper_success() {
    // S = E × B
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    // Delegation, not merely success.
    let effect = poynting_vector(&e, &b);
    let carried = effect.value_cloned().unwrap();
    let direct = poynting_vector_kernel(&e, &b).unwrap();
    assert_eq!(
        carried.inner().data(),
        direct.data(),
        "poynting_vector must carry the value its kernel produced"
    );
}

#[test]
fn test_poynting_vector_wrapper_error() {
    let e = CausalMultiVector::new(vec![1.0; 8], Metric::Euclidean(3)).unwrap();
    let b = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = poynting_vector(&e, &b);
    assert!(effect.is_err());
}

// =============================================================================
// magnetic_helicity_density Wrapper Tests
// =============================================================================

// NOTE on two defensively-unreachable wrapper branches:
//   * wrappers.rs:52 — the Err arm of `lorenz_gauge`. `lorenz_gauge_kernel`
//     only computes `codifferential(1)` and always returns `Ok`; it has no
//     error path, so this arm can never run.
//   * wrappers.rs:81 — the inner `Err(e)` arm for `MagneticFlux::<R>::new(val)`
//     inside `magnetic_helicity_density`. `MagneticFlux::new` is infallible
//     (it unconditionally returns `Ok`), so this arm is unreachable.
// Both are left uncovered by design; no input can drive them.

#[test]
fn test_magnetic_helicity_density_wrapper_success() {
    // h = A · B
    let potential = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let field = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    // Delegation, not merely success.
    let effect = magnetic_helicity_density(&potential, &field);
    assert_eq!(
        effect.value_cloned().unwrap().value(),
        magnetic_helicity_density_kernel(&potential, &field).unwrap(),
        "magnetic_helicity_density must carry the value its kernel produced"
    );
}

#[test]
fn test_magnetic_helicity_density_wrapper_error() {
    let a = CausalMultiVector::new(vec![1.0; 8], Metric::Euclidean(3)).unwrap();
    let b = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = magnetic_helicity_density(&a, &b);
    assert!(effect.is_err());
}

// =============================================================================
// Maxwell / Lorenz / Proca Wrappers
// =============================================================================

#[test]
fn test_maxwell_gradient_wrapper_success() {
    // Delegation, not merely success.
    let manifold = create_simple_manifold();
    let effect = maxwell_gradient(&manifold);
    let carried = effect.value_cloned().unwrap();
    let direct = maxwell_gradient_kernel(&manifold).unwrap();
    assert_eq!(
        carried.as_slice(),
        direct.as_slice(),
        "maxwell_gradient must carry the value its kernel produced"
    );
}

#[test]
fn test_lorenz_gauge_wrapper_success() {
    // Delegation, not merely success.
    let manifold = create_simple_manifold();
    let effect = lorenz_gauge(&manifold);
    let carried = effect.value_cloned().unwrap();
    let direct = lorenz_gauge_kernel(&manifold).unwrap();
    assert_eq!(
        carried.as_slice(),
        direct.as_slice(),
        "lorenz_gauge must carry the value its kernel produced"
    );
}

#[test]
fn test_proca_equation_wrapper_success() {
    // With the kernel refactor to slice potential data, this should now succeed
    // even with mismatched underlying tensor sizes, as long as the 1-form parts align.
    let field = create_simple_manifold();
    let potential = create_simple_manifold(); // full simplex data

    // Delegation, not merely success. `proca_equation_kernel` is where cargo mutants left four
    // survivors, including the sign of the current, because no success-path test looked at the
    // value it produced.
    let effect = proca_equation(&field, &potential, 0.5);
    let carried = effect.value_cloned().unwrap();
    let direct = proca_equation_kernel(&field, &potential, 0.5).unwrap();
    assert_eq!(
        carried.as_slice(),
        direct.as_slice(),
        "proca_equation must carry the value its kernel produced"
    );
    assert!(
        carried.as_slice().iter().any(|v: &f64| v.abs() > 0.0),
        "the Proca current must not be identically zero for this fixture"
    );
}

#[test]
fn test_maxwell_gradient_wrapper_error() {
    // A 1D complex makes the kernel produce an empty 2-form, so the wrapper
    // must take the Err arm (wrappers.rs:39) and yield an error effect.
    let manifold = create_1d_manifold();
    let effect = maxwell_gradient(&manifold);
    assert!(effect.is_err());
}

#[test]
fn test_proca_equation_wrapper_error() {
    // NaN mass forces the kernel into its NumericalInstability branch, so the
    // wrapper takes the Err arm (wrappers.rs:98).
    let field = create_simple_manifold();
    let potential = create_simple_manifold();
    let effect = proca_equation(&field, &potential, f64::NAN);
    assert!(effect.is_err());
}

#[test]
fn test_proca_equation_wrapper_error_propagation() {
    // A NaN mass drives `proca_equation_kernel` into its finiteness guard, and the wrapper must
    // forward that refusal rather than absorb it. The body used to assert success under an
    // error-propagation name, because the fixture it was written for had stopped failing.
    let field = create_simple_manifold();
    let potential = create_simple_manifold();

    let effect = proca_equation(&field, &potential, f64::NAN);
    assert!(
        effect.is_err(),
        "a non-finite mass must propagate out of the wrapper as an error"
    );
    assert!(
        effect.value().is_none(),
        "an errored effect carries no value"
    );
}
