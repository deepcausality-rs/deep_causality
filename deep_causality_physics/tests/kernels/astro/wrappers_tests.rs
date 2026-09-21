/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Length, Mass, escape_velocity, escape_velocity_kernel, orbital_velocity,
    orbital_velocity_kernel, schwarzschild_radius, schwarzschild_radius_kernel,
};

// =============================================================================
// orbital_velocity Wrapper Tests
// =============================================================================

#[test]
fn test_orbital_velocity_wrapper_success() {
    let mass = Mass::<f64>::new(5.972e24).unwrap();
    let radius = Length::<f64>::new(6.371e6).unwrap();

    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant. `escape_velocity` in this file
    // already had this assertion; its two siblings did not.
    let effect = orbital_velocity(&mass, &radius);
    assert_eq!(
        effect.value_cloned().unwrap(),
        orbital_velocity_kernel(&mass, &radius).unwrap(),
        "orbital_velocity must carry the value its kernel produced"
    );

    // v = sqrt(GM/r) for Earth is about 7.9 km/s.
    let speed = effect.value_cloned().unwrap();
    let v: f64 = speed.value();
    assert!(
        (v - 7_909.0).abs() < 20.0,
        "low Earth orbital speed is about 7.9 km/s, got {v}"
    );
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        escape_velocity_kernel(&mass, &radius).unwrap(),
        "escape_velocity must carry the value its kernel produced"
    );

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

    // Delegation, not merely success.
    let effect = schwarzschild_radius(&mass);
    assert_eq!(
        effect.value_cloned().unwrap(),
        schwarzschild_radius_kernel(&mass).unwrap(),
        "schwarzschild_radius must carry the value its kernel produced"
    );

    // The Sun's Schwarzschild radius is about 2.95 km.
    let r_s = effect.value_cloned().unwrap();
    let radius_m: f64 = r_s.value();
    assert!(
        (radius_m - 2953.0).abs() < 5.0,
        "expected about 2953 m, got {radius_m}"
    );
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
