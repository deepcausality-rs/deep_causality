/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    lorenz_gauge_kernel, magnetic_helicity_density_kernel, maxwell_gradient_kernel,
    poynting_vector_kernel, proca_equation_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

// Helper to create a simple triangular manifold
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

#[test]
fn test_poynting_vector_kernel_valid() {
    // S = E x B
    // E = [0, 1, 0, 0] (x)
    // B = [0, 0, 1, 0] (y)
    // S should be x^y bivector
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

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_ok());

    let s = result.unwrap();
    assert!(!s.data().is_empty());
}

#[test]
fn test_poynting_vector_kernel_dimension_error() {
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = poynting_vector_kernel(&e, &b);
    assert!(result.is_err());
}

#[test]
fn test_poynting_vector_kernel_nan_error() {
    let e = CausalMultiVector::new(
        vec![f64::NAN, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
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
    let a = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
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
    let a = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = magnetic_helicity_density_kernel(&a, &b);
    assert!(result.is_err());
}

#[test]
fn test_maxwell_gradient_kernel_valid() {
    let manifold = create_simple_manifold();
    let result = maxwell_gradient_kernel(&manifold);
    assert!(result.is_ok());
    // F = dA. Just checking it computes without error on valid manifold
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

// =============================================================================
// Energy Density Kernel Tests
// =============================================================================

use deep_causality_physics::{energy_density_kernel, lagrangian_density_kernel};

#[test]
fn test_energy_density_kernel_valid() {
    // E = [0, 1, 0, 0] (unit vector in x)
    // B = [0, 0, 1, 0] (unit vector in y)
    // u = (E² + B²) / 2 = (1 + 1) / 2 = 1.0
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

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_ok());

    let u = result.unwrap();
    assert!((u - 1.0).abs() < 1e-10, "Expected 1.0, got {}", u);
}

#[test]
fn test_energy_density_kernel_zero_fields() {
    let e = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
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
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = energy_density_kernel(&e, &b);
    assert!(result.is_err());
}

#[test]
fn test_energy_density_kernel_nan_error() {
    let e = CausalMultiVector::new(
        vec![f64::NAN, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
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
    // E = [0, 1, 0, 0], B = [0, 0, 1, 0]
    // L = (E² - B²) / 2 = (1 - 1) / 2 = 0.0
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

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());
    assert!(
        result.unwrap().abs() < 1e-10,
        "Expected 0.0 for equal E and B magnitudes"
    );
}

#[test]
fn test_lagrangian_density_kernel_electric_dominated() {
    // E = [0, 2, 0, 0], B = [0, 0, 1, 0]
    // L = (4 - 1) / 2 = 1.5
    let e = CausalMultiVector::new(
        vec![0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());

    let l = result.unwrap();
    assert!((l - 1.5).abs() < 1e-10, "Expected 1.5, got {}", l);
}

#[test]
fn test_lagrangian_density_kernel_magnetic_dominated() {
    // E = [0, 1, 0, 0], B = [0, 0, 2, 0]
    // L = (1 - 4) / 2 = -1.5
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_ok());

    let l = result.unwrap();
    assert!((l - (-1.5)).abs() < 1e-10, "Expected -1.5, got {}", l);
}

#[test]
fn test_lagrangian_density_kernel_dimension_mismatch() {
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let result = lagrangian_density_kernel(&e, &b);
    assert!(result.is_err());
}
