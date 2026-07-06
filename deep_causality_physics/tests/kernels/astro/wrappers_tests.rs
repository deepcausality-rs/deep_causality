/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Length, Mass, escape_velocity, orbital_velocity, schwarzschild_radius,
};

// =============================================================================
// orbital_velocity Wrapper Tests
// =============================================================================

#[test]
fn test_orbital_velocity_wrapper_success() {
    let mass = Mass::<f64>::new(5.972e24).unwrap();
    let radius = Length::<f64>::new(6.371e6).unwrap();

    let effect = orbital_velocity(&mass, &radius);
    assert!(effect.is_ok(), "Expected successful PropagatingEffect");

    let speed = effect.value_cloned().unwrap();
    assert!(speed.value() > 0.0);
}

#[test]
fn test_orbital_velocity_wrapper_error() {
    let mass = Mass::<f64>::new(1e24).unwrap();
    let radius = Length::<f64>::new(0.0).unwrap();

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
    let mass = Mass::<f64>::new(5.972e24).unwrap();
    let radius = Length::<f64>::new(6.371e6).unwrap();

    let effect = escape_velocity(&mass, &radius);
    assert!(effect.is_ok());

    let speed = effect.value_cloned().unwrap();
    assert!(speed.value() > 0.0);
}

#[test]
fn test_escape_velocity_wrapper_error() {
    let mass = Mass::<f64>::new(1e24).unwrap();
    let radius = Length::<f64>::new(0.0).unwrap();

    let effect = escape_velocity(&mass, &radius);
    assert!(effect.is_err());
}

// =============================================================================
// schwarzschild_radius Wrapper Tests
// =============================================================================

#[test]
fn test_schwarzschild_radius_wrapper_success() {
    let mass = Mass::<f64>::new(1.989e30).unwrap();

    let effect = schwarzschild_radius(&mass);
    assert!(effect.is_ok());

    let r_s = effect.value_cloned().unwrap();
    assert!(r_s.value() > 0.0);
}

#[test]
fn test_schwarzschild_radius_wrapper_zero_mass() {
    let mass = Mass::<f64>::new(0.0).unwrap();

    let effect = schwarzschild_radius(&mass);
    assert!(effect.is_ok());

    let r_s = effect.value_cloned().unwrap();
    assert!((r_s.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_schwarzschild_radius_wrapper_error_negative_mass() {
    // r_s = 2·G·m / c². A negative mass yields a negative radius, which
    // `Length::new` rejects (Length cannot be negative), so the kernel returns
    // `Err` and the wrapper forwards it via its error arm (wrappers.rs:41).
    // `Mass::new` rejects negatives, so we feed the negative mass through
    // `new_unchecked` to reach the kernel's `Length::new` failure.
    let mass = Mass::<f64>::new_unchecked(-1.989e30);

    let effect = schwarzschild_radius(&mass);
    assert!(
        effect.is_err(),
        "negative mass must produce a negative radius rejected by Length::new"
    );
}
