/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Frequency, Mass, MomentOfInertia, PhysicalVector, angular_momentum_kernel,
    kinetic_energy_kernel, rotational_kinetic_energy_kernel, torque_kernel,
};

// =============================================================================
// kinetic_energy_kernel Tests
// =============================================================================

#[test]
fn test_kinetic_energy_kernel_valid() {
    // KE = 0.5 * m * v^2
    let mass = Mass::new(2.0).unwrap();
    // Create a 3D velocity vector [3, 4, 0] with magnitude 5
    let velocity = CausalMultiVector::new(
        vec![0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    // v^2 = 3^2 + 4^2 = 25
    // KE = 0.5 * 2 * 25 = 25
    assert!((ke - 25.0).abs() < 1e-10, "Expected KE = 25, got {}", ke);
}

#[test]
fn test_kinetic_energy_kernel_zero_velocity() {
    let mass = Mass::new(10.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!(
        (ke - 0.0).abs() < 1e-10,
        "Zero velocity should give zero KE"
    );
}

#[test]
fn test_kinetic_energy_kernel_zero_mass() {
    let mass = Mass::new(0.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!((ke - 0.0).abs() < 1e-10, "Zero mass should give zero KE");
}

// =============================================================================
// rotational_kinetic_energy_kernel Tests
// =============================================================================

#[test]
fn test_rotational_kinetic_energy_kernel_valid() {
    // KE_rot = 0.5 * I * omega^2
    let inertia = MomentOfInertia::new(4.0).unwrap();
    let omega = Frequency::new(3.0).unwrap();

    let result = rotational_kinetic_energy_kernel(inertia, omega);
    assert!(result.is_ok());

    let ke = result.unwrap();
    // KE = 0.5 * 4 * 9 = 18
    assert!((ke - 18.0).abs() < 1e-10, "Expected KE = 18, got {}", ke);
}

#[test]
fn test_rotational_kinetic_energy_kernel_zero_omega() {
    let inertia = MomentOfInertia::new(10.0).unwrap();
    let omega = Frequency::new(0.0).unwrap();

    let result = rotational_kinetic_energy_kernel(inertia, omega);
    assert!(result.is_ok());

    let ke = result.unwrap();
    assert!((ke - 0.0).abs() < 1e-10);
}

// =============================================================================
// torque_kernel Tests
// =============================================================================

#[test]
fn test_torque_kernel_valid() {
    // Torque = r × F (outer product in GA)
    // r = [0, 1, 0, 0] (x-direction)
    // F = [0, 0, 1, 0] (y-direction)
    // r ^ F should give bivector in xy-plane
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = torque_kernel(&radius, &force);
    assert!(result.is_ok());

    let torque: PhysicalVector = result.unwrap();
    // The result is a bivector (torque plane)
    assert!(!torque.inner().data().is_empty());
}

#[test]
fn test_torque_kernel_parallel_vectors() {
    // Parallel vectors have zero cross product
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = torque_kernel(&radius, &force);
    assert!(result.is_ok());
    // Result should be zero bivector
}

// =============================================================================
// angular_momentum_kernel Tests
// =============================================================================

#[test]
fn test_angular_momentum_kernel_valid() {
    // L = r × p (outer product)
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let momentum = CausalMultiVector::new(
        vec![0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = angular_momentum_kernel(&radius, &momentum);
    assert!(result.is_ok());

    let l: PhysicalVector = result.unwrap();
    assert!(!l.inner().data().is_empty());
}

// =============================================================================
// PhysicalVector Tests
// =============================================================================

#[test]
fn test_physical_vector_default() {
    let pv = PhysicalVector::default();
    // Default should be a scalar 0 multivector
    assert!((pv.inner().data()[0] - 0.0).abs() < 1e-10);
}

#[test]
fn test_physical_vector_new_and_accessors() {
    let mv = CausalMultiVector::new(
        vec![1.0, 2.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let pv = PhysicalVector::new(mv.clone());

    assert_eq!(pv.inner().data(), mv.data());

    let inner = pv.into_inner();
    assert_eq!(inner.data(), mv.data());
}
