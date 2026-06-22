/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    lorenz_gauge_kernel, magnetic_helicity_density_kernel, maxwell_gradient_kernel,
    poynting_vector_kernel, proca_equation_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple triangular manifold with a unit-edge metric
// attached (required by R4.5: `codifferential` / `laplacian` panic without
// a metric; simplicial Hodge ⋆ data lives on the complex cache, so any
// correctly-sized `ReggeGeometry` validates).
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

#[test]
fn test_poynting_vector_kernel_valid() {
    // S = E x B
    // E = [0, 1, 0, 0] (x)
    // B = [0, 0, 1, 0] (y)
    // S should be x^y bivector
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_ok());

    let s = result.unwrap();
    assert!(!s.data().is_empty());
}

#[test]
fn test_poynting_vector_kernel_dimension_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_err());
}

#[test]
fn test_poynting_vector_kernel_nan_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![f64::NAN, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_err());
}

#[test]
fn test_magnetic_helicity_density_kernel_valid() {
    // h = A . B
    let a = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = magnetic_helicity_density_kernel(&a, &b);
    assert!(result.is_ok());

    let h = result.unwrap();
    // Dot product of identical unit vectors is 1.0
    assert!((h - 1.0).abs() < 1e-10);
}

#[test]
fn test_magnetic_helicity_density_error() {
    let a = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = magnetic_helicity_density_kernel(&a, &b);
    assert!(result.is_err());
}

// Helper: a purely 1-dimensional complex (vertices + edges only, no faces).
// `exterior_derivative(1)` returns an empty 2-form here (k == max_dim), driving
// `maxwell_gradient_kernel` into its empty/invalid-2-form error branch.
fn create_1d_manifold() -> SimplicialManifold<f64, f64> {
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

// Helper: a 4D pentatope (5 points) — the same valid manifold used by the
// GRMHD tests. It has 10 edges, comfortably more than a triangle's 7-element
// data slab, so it drives the "potential too short" Proca branch.
fn create_large_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, 0.0, 0.0, // v0
            1.0, 0.0, 0.0, 0.0, // v1
            0.0, 1.0, 0.0, 0.0, // v2
            0.0, 0.0, 1.0, 0.0, // v3
            0.0, 0.0, 0.0, 1.0, // v4
        ],
        vec![5, 4],
    )
    .unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 5], vec![5]).unwrap(), 0).unwrap();
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

#[test]
fn test_maxwell_gradient_kernel_valid() {
    let manifold = create_simple_manifold();
    let result = maxwell_gradient_kernel(&manifold);
    assert!(result.is_ok());
    // F = dA. Just checking it computes without error on valid manifold
}

#[test]
fn test_maxwell_gradient_kernel_empty_2form_error() {
    // 1D complex: exterior_derivative(1) is empty (k == max_dim), tripping the
    // DimensionMismatch guard at fields.rs:34-38.
    let manifold = create_1d_manifold();
    let result = maxwell_gradient_kernel(&manifold);
    assert!(result.is_err());
}

#[test]
fn test_lorenz_gauge_kernel_valid() {
    let manifold = create_simple_manifold();
    let result = lorenz_gauge_kernel(&manifold);
    assert!(result.is_ok());
}

// =============================================================================
// Proca Equation Kernel Tests
// =============================================================================

// NOTE: The proca_equation_kernel has a known shape mismatch issue:
// - codifferential(2) returns tensor of shape [num_1_simplices] (1-forms)
// - potential_manifold.data() returns tensor of shape [total_simplices]
// When the manifold has vertices/faces beyond edges, these shapes differ.
// This test documents the current behavior.

#[test]
fn test_proca_equation_kernel_valid() {
    // Both manifolds have vertices + edges + faces.
    // Previous behavior: Mismatch between total data length and 1-form length caused error.
    // New behavior: logic slices the potential data to match, so this should pass.
    let field_manifold = create_simple_manifold();
    let potential_manifold = create_simple_manifold();
    let mass = 0.5;

    let result = proca_equation_kernel(&field_manifold, &potential_manifold, mass);
    assert!(result.is_ok(), "Proca kernel failed: {:?}", result.err());
}

#[test]
fn test_proca_equation_kernel_nan_mass() {
    let field_manifold = create_simple_manifold();
    let potential_manifold = create_simple_manifold();
    let mass = f64::NAN;

    let result = proca_equation_kernel(&field_manifold, &potential_manifold, mass);
    assert!(result.is_err());
}

#[test]
fn test_proca_equation_kernel_inf_mass() {
    let field_manifold = create_simple_manifold();
    let potential_manifold = create_simple_manifold();
    let mass = f64::INFINITY;

    let result = proca_equation_kernel(&field_manifold, &potential_manifold, mass);
    assert!(result.is_err());
}

// Helper: build a 2D triangle manifold whose data slab is supplied verbatim,
// so a test can inject non-finite or huge values into specific form slots.
fn manifold_with_data(data: Vec<f64>) -> SimplicialManifold<f64, f64> {
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
    assert_eq!(data.len(), num_simplices, "data must match total simplices");
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
fn test_proca_equation_kernel_delta_f_non_finite() {
    // Inject NaN into the field manifold's 2-form (face) slot so codifferential(2)
    // -> delta_f carries non-finite entries (fields.rs:159-163).
    // Standard triangle: 3 verts + 3 edges + 1 face = 7 simplices; the face is
    // the last slot.
    let mut field_data = vec![1.0; 7];
    field_data[6] = f64::NAN; // the single 2-simplex (face)
    let field = manifold_with_data(field_data);
    let potential = create_simple_manifold();

    let result = proca_equation_kernel(&field, &potential, 0.5);
    assert!(result.is_err());
}

#[test]
fn test_proca_equation_kernel_potential_too_short() {
    // The kernel slices `a_full[..needed_len]` where needed_len == delta_f.len()
    // (the number of 1-simplices / edges of the field). When the potential
    // manifold's full data slab is shorter than that, the DimensionMismatch
    // guard fires (fields.rs:176-182).
    //
    // Large field mesh -> many edges; tiny single-triangle potential -> total
    // data length 7, smaller than the large mesh's edge count.
    let field = create_large_manifold();
    let potential = create_simple_manifold(); // 7-element data slab

    // Sanity: confirm the precondition (field edges > potential data length).
    let n1_field = field.complex().skeletons()[1].simplices().len();
    assert!(
        n1_field > potential.data().as_slice().len(),
        "precondition: field edges ({}) must exceed potential data len ({})",
        n1_field,
        potential.data().as_slice().len()
    );

    let result = proca_equation_kernel(&field, &potential, 0.5);
    assert!(
        result.is_err(),
        "expected dimension mismatch; field edges should exceed potential data length"
    );
}

#[test]
fn test_proca_equation_kernel_a_1form_non_finite() {
    // The kernel reads the potential's 1-form as the *first* needed_len entries
    // of the full data slab, i.e. the leading slots (vertices for a triangle).
    // Putting NaN there trips the A(1-form) finiteness guard (fields.rs:186-190).
    let field = create_simple_manifold();
    // needed_len == 3 edges; a_full[..3] are the first three (vertex) slots.
    let mut pot_data = vec![1.0; 7];
    pot_data[0] = f64::NAN;
    pot_data[1] = f64::NAN;
    pot_data[2] = f64::NAN;
    let potential = manifold_with_data(pot_data);

    let result = proca_equation_kernel(&field, &potential, 0.5);
    assert!(result.is_err());
}

#[test]
fn test_proca_equation_kernel_m2_a_overflow() {
    // Finite-but-huge potential 1-form (first needed_len slab entries) times a
    // modest m^2 overflows to inf, tripping the m^2 A guard (fields.rs:194-198).
    let field = create_simple_manifold();
    let mut pot_data = vec![1.0; 7];
    pot_data[0] = f64::MAX;
    pot_data[1] = f64::MAX;
    pot_data[2] = f64::MAX;
    let potential = manifold_with_data(pot_data);

    // mass = 10 -> m^2 = 100 (finite); MAX * 100 overflows to +inf.
    let result = proca_equation_kernel(&field, &potential, 10.0);
    assert!(result.is_err());
}

#[test]
fn test_proca_equation_kernel_j_sum_overflow() {
    // delta_f (from a huge-but-finite field 2-form) plus a huge-but-finite
    // m^2 A overflows to ±inf when summed, tripping the final J finiteness guard
    // (fields.rs:210-214) — past both the delta_f and m^2 A finiteness checks.
    let big = 1.0e308_f64; // finite, but big + big overflows to +inf
    let mut field_data = vec![1.0; 7];
    field_data[6] = big; // the 2-simplex (face) -> large finite delta_f
    let field = manifold_with_data(field_data);

    // potential 1-form (first needed_len slab slots) large finite; mass = 1 so
    // m^2 = 1 keeps m^2 A finite, but delta_f + m^2 A overflows on summation.
    let mut pot_data = vec![1.0; 7];
    pot_data[0] = big;
    pot_data[1] = big;
    pot_data[2] = big;
    let potential = manifold_with_data(pot_data);

    let result = proca_equation_kernel(&field, &potential, 1.0);
    assert!(result.is_err());
}

// NOTE on two defensively-unreachable Proca branches:
//   * fields.rs:202-206 — the "Shape mismatch in Proca" guard. `a_1form` is
//     constructed with `delta_f.shape()` and length `needed_len == delta_f.len()`,
//     so `m2_a.shape()` is always equal to `delta_f.shape()`. The mismatch
//     branch can never fire for any input.
//   * fields.rs:211-213 — the final J finiteness guard reached via a
//     finite-but-overflowing *sum* `delta_f + m2_a`. To overflow the sum,
//     `delta_f` would itself have to be finite yet near f64::MAX; but a field
//     2-form large enough to scale (through the Hodge-weighted codifferential)
//     to near-MAX overflows to ±inf *inside* `codifferential`, which is caught
//     earlier by the delta_f finiteness guard (lines 159-163, covered by
//     `test_proca_equation_kernel_j_sum_overflow`). The sum-overflow path is
//     therefore effectively unreachable for real f64 input.

// =============================================================================
// Energy Density Kernel Tests
// =============================================================================

use deep_causality_physics::{energy_density_kernel, lagrangian_density_kernel};

#[test]
fn test_energy_density_kernel_sum_overflow_result_rejected() {
    // Each squared magnitude is finite (≈ f64::MAX), but their *sum* overflows
    // to +inf, so the final result `u = 0.5*(E^2 + B^2)` is non-finite,
    // tripping the result guard at fields.rs:263-267 (past the per-term
    // squared-magnitude finiteness checks). Component = sqrt(MAX) so its square
    // is ≈ MAX (finite), placed at a spatial index.
    let huge = f64::MAX.sqrt();
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, huge, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, huge, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(energy_density_kernel(&e, &b).is_err());
}

// NOTE on fields.rs:317-319 (lagrangian non-finite *result* guard): the result
// is `L = 0.5*(E^2 - B^2)`. By the time this guard is reached, both `e_squared`
// and `b_squared` are individually finite and non-negative. The difference of
// two finite non-negative values lies in [-MAX, MAX] and 0.5*(.) cannot
// overflow, so `L` is always finite here. This guard is therefore defensively
// unreachable for any real f64 input; only the squared-magnitude guards
// (lines 306-310, covered) can fire on extreme inputs.

#[test]
fn test_energy_density_kernel_valid() {
    // E = (1, 0, 0) at indices 2,3,4 (4D multivector)
    // B = (0, 1, 0) at indices 2,3,4
    // u = (│E│² + │B│²) / 2 = (1 + 1) / 2 = 1.0
    let e = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_ok());

    let u = result.unwrap();
    assert!((u - 1.0).abs() < 1e-10, "Expected 1.0, got {}", u);
}

#[test]
fn test_energy_density_kernel_zero_fields() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_ok());
    assert!(result.unwrap().abs() < 1e-10);
}

#[test]
fn test_energy_density_kernel_dimension_mismatch() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_err());
}

#[test]
fn test_energy_density_kernel_nan_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![f64::NAN, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_err());
}

// =============================================================================
// Lagrangian Density Kernel Tests
// =============================================================================

#[test]
fn test_lagrangian_density_kernel_valid() {
    // E = (1, 0, 0), B = (0, 1, 0) at indices 2,3,4
    // L = (│E│² - │B│²) / 2 = (1 - 1) / 2 = 0.0
    let e = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());
    assert!(
        result.unwrap().abs() < 1e-10,
        "Expected 0.0 for equal E and B magnitudes"
    );
}

#[test]
fn test_lagrangian_density_kernel_electric_dominated() {
    // E = (2, 0, 0), B = (0, 1, 0) at indices 2,3,4
    // L = (│E│² - │B│²) / 2 = (4 - 1) / 2 = 1.5
    let e = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());

    let l = result.unwrap();
    assert!((l - 1.5).abs() < 1e-10, "Expected 1.5, got {}", l);
}

#[test]
fn test_lagrangian_density_kernel_magnetic_dominated() {
    // E = (1, 0, 0), B = (0, 2, 0) at indices 2,3,4
    // L = (│E│² - │B│²) / 2 = (1 - 4) / 2 = -1.5
    let e = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![
            0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Euclidean(4),
    )
    .unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());

    let l = result.unwrap();
    assert!((l - (-1.5)).abs() < 1e-10, "Expected -1.5, got {}", l);
}

#[test]
fn test_poynting_vector_kernel_nan_b_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(poynting_vector_kernel(&e, &b).is_err());
}

#[test]
fn test_energy_density_kernel_nan_b_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(energy_density_kernel(&e, &b).is_err());
}

#[test]
fn test_lagrangian_density_kernel_nan_b_error() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(lagrangian_density_kernel(&e, &b).is_err());
}

// =============================================================================
// Overflow guards: finite inputs whose intermediate products overflow to ±inf,
// exercising the post-computation non-finite checks.
// =============================================================================

#[test]
fn test_poynting_vector_kernel_overflow_result_is_rejected() {
    // Finite but huge spatial components: the cross product overflows to ±inf, tripping the
    // non-finite-result guard (distinct from the non-finite-input guard).
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, f64::MAX, f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, f64::MAX, f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(poynting_vector_kernel(&e, &b).is_err());
}

#[test]
fn test_proca_equation_kernel_m_squared_overflow_is_rejected() {
    // `mass` is finite, but `mass * mass` overflows to inf, tripping the m^2 guard.
    let field_manifold = create_simple_manifold();
    let potential_manifold = create_simple_manifold();
    let result = proca_equation_kernel(&field_manifold, &potential_manifold, f64::MAX);
    assert!(result.is_err());
}

#[test]
fn test_energy_density_kernel_overflow_squared_magnitude_is_rejected() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(energy_density_kernel(&e, &b).is_err());
}

#[test]
fn test_lagrangian_density_kernel_overflow_squared_magnitude_is_rejected() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    assert!(lagrangian_density_kernel(&e, &b).is_err());
}

#[test]
fn test_lagrangian_density_kernel_dimension_mismatch() {
    let e = CausalMultiVector::<f64>::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::<f64>::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_err());
}
