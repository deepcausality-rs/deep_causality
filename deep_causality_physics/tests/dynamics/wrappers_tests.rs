/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    angular_momentum, kinetic_energy, rotational_kinetic_energy, torque,
    Frequency, Mass, MomentOfInertia,
};

// =============================================================================
// kinetic_energy Wrapper Tests
// =============================================================================

#[test]
fn test_kinetic_energy_wrapper_success() {
    let mass = Mass::new(2.0).unwrap();
    let velocity = CausalMultiVector::new(vec![0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = kinetic_energy(&mass, &velocity);
    assert!(effect.is_ok());

    let energy = effect.value().clone().into_value().unwrap();
    assert!(energy.value() > 0.0);
}

// =============================================================================
// rotational_kinetic_energy Wrapper Tests
// =============================================================================

#[test]
fn test_rotational_kinetic_energy_wrapper_success() {
    let inertia = MomentOfInertia::new(4.0).unwrap();
    let omega = Frequency::new(3.0).unwrap();

    let effect = rotational_kinetic_energy(&inertia, &omega);
    assert!(effect.is_ok());

    let energy = effect.value().clone().into_value().unwrap();
    assert!(energy.value() > 0.0);
}

// =============================================================================
// torque Wrapper Tests
// =============================================================================

#[test]
fn test_torque_wrapper_success() {
    let radius = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();
    let force = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = torque(&radius, &force);
    assert!(effect.is_ok());
}

// =============================================================================
// angular_momentum Wrapper Tests
// =============================================================================

#[test]
fn test_angular_momentum_wrapper_success() {
    let radius = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();
    let momentum = CausalMultiVector::new(vec![0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0], Metric::Euclidean(3)).unwrap();

    let effect = angular_momentum(&radius, &momentum);
    assert!(effect.is_ok());
}
