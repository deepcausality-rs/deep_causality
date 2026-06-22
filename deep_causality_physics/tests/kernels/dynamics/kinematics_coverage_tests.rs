/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for error branches in `kernels::dynamics::kinematics`.

use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_physics::{Mass, PhysicsErrorEnum, kinetic_energy_kernel};

// =============================================================================
// kinetic_energy_kernel error branches (kinematics.rs:55-57, 62-64)
// =============================================================================

#[test]
fn test_kinetic_energy_kernel_non_finite_velocity() {
    // A velocity component of +∞ makes the squared magnitude non-finite,
    // hitting the `!v_sq.is_finite()` branch (kinematics.rs:55-57).
    let mass = Mass::<f64>::new(2.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::NumericalInstability(_) => {}
        e => panic!("Expected NumericalInstability, got {e:?}"),
    }
}

#[test]
fn test_kinetic_energy_kernel_negative_squared_speed() {
    // Under a Minkowski (+ - - -) metric a purely spacelike vector has a
    // strictly negative squared magnitude, hitting the negative-squared-speed
    // branch (kinematics.rs:62-64).
    let mass = Mass::<f64>::new(2.0).unwrap();
    // Cl(1,3): 16 basis blades; place a unit value on a spacelike grade-1 axis.
    let mut data = vec![0.0_f64; 16];
    data[2] = 1.0; // spacelike basis vector e1
    let velocity = CausalMultiVector::new(data, Metric::Minkowski(4)).unwrap();

    // Confirm this metric yields a negative squared magnitude before asserting
    // the kernel rejects it.
    let v_sq = velocity.squared_magnitude();
    assert!(
        v_sq < 0.0,
        "expected negative squared magnitude, got {v_sq}"
    );

    let result = kinetic_energy_kernel(mass, &velocity);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        e => panic!("Expected PhysicalInvariantBroken, got {e:?}"),
    }
}
