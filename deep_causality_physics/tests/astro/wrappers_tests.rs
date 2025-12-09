/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Length, Mass, escape_velocity, orbital_velocity, schwarzschild_radius,
};

// =============================================================================
// orbital_velocity Wrapper Tests
// =============================================================================

#[test]
fn test_orbital_velocity_wrapper_success() {
    let mass = Mass::new(5.972e24).unwrap();
    let radius = Length::new(6.371e6).unwrap();

    let effect = orbital_velocity(&mass, &radius);
    assert!(effect.is_ok(), "Expected successful PropagatingEffect");

    let speed = effect.value().clone().into_value().unwrap();
    assert!(speed.value() > 0.0);
}

#[test]
fn test_orbital_velocity_wrapper_error() {
    let mass = Mass::new(1e24).unwrap();
    let radius = Length::new(0.0).unwrap();

    let effect = orbital_velocity(&mass, &radius);
    assert!(
        effect.is_err(),
        "Expected error PropagatingEffect for zero radius"
    );
}

// =============================================================================
// escape_velocity Wrapper Tests
// =============================================================================

#[test]
fn test_escape_velocity_wrapper_success() {
    let mass = Mass::new(5.972e24).unwrap();
    let radius = Length::new(6.371e6).unwrap();

    let effect = escape_velocity(&mass, &radius);
    assert!(effect.is_ok());

    let speed = effect.value().clone().into_value().unwrap();
    assert!(speed.value() > 0.0);
}

#[test]
fn test_escape_velocity_wrapper_error() {
    let mass = Mass::new(1e24).unwrap();
    let radius = Length::new(0.0).unwrap();

    let effect = escape_velocity(&mass, &radius);
    assert!(effect.is_err());
}

// =============================================================================
// schwarzschild_radius Wrapper Tests
// =============================================================================

#[test]
fn test_schwarzschild_radius_wrapper_success() {
    let mass = Mass::new(1.989e30).unwrap();

    let effect = schwarzschild_radius(&mass);
    assert!(effect.is_ok());

    let r_s = effect.value().clone().into_value().unwrap();
    assert!(r_s.value() > 0.0);
}

#[test]
fn test_schwarzschild_radius_wrapper_zero_mass() {
    let mass = Mass::new(0.0).unwrap();

    let effect = schwarzschild_radius(&mass);
    assert!(effect.is_ok());

    let r_s = effect.value().clone().into_value().unwrap();
    assert!((r_s.value() - 0.0).abs() < 1e-10);
}
