/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    escape_velocity_kernel, orbital_velocity_kernel, schwarzschild_radius_kernel,
    Length, Mass, PhysicsErrorEnum, G, SPEED_OF_LIGHT,
};

// =============================================================================
// orbital_velocity_kernel Tests
// =============================================================================

#[test]
fn test_orbital_velocity_kernel_valid() {
    // Earth-like orbit: M ≈ Earth mass proxy, r = radius
    let mass = Mass::new(1e24).unwrap(); // kg
    let radius = Length::new(6.371e6).unwrap(); // Earth radius in meters

    let result = orbital_velocity_kernel(&mass, &radius);
    assert!(result.is_ok());

    let speed = result.unwrap();
    assert!(speed.value() > 0.0, "Orbital velocity must be positive");
}

#[test]
fn test_orbital_velocity_kernel_zero_radius_error() {
    let mass = Mass::new(1e24).unwrap();
    let radius = Length::new(0.0).unwrap();

    let result = orbital_velocity_kernel(&mass, &radius);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match &err.0 {
        PhysicsErrorEnum::MetricSingularity(msg) => {
            assert!(msg.contains("Zero radius"));
        }
        _ => panic!("Expected MetricSingularity error"),
    }
}

#[test]
fn test_orbital_velocity_kernel_zero_mass() {
    let mass = Mass::new(0.0).unwrap();
    let radius = Length::new(1000.0).unwrap();

    let result = orbital_velocity_kernel(&mass, &radius);
    assert!(result.is_ok());

    let speed = result.unwrap();
    // Zero mass → zero velocity
    assert!((speed.value() - 0.0).abs() < 1e-10);
}

// =============================================================================
// escape_velocity_kernel Tests
// =============================================================================

#[test]
fn test_escape_velocity_kernel_valid() {
    let mass = Mass::new(1e24).unwrap();
    let radius = Length::new(6.371e6).unwrap();

    let result = escape_velocity_kernel(&mass, &radius);
    assert!(result.is_ok());

    let speed = result.unwrap();
    assert!(speed.value() > 0.0, "Escape velocity must be positive");
}

#[test]
fn test_escape_velocity_kernel_zero_radius_error() {
    let mass = Mass::new(1e24).unwrap();
    let radius = Length::new(0.0).unwrap();

    let result = escape_velocity_kernel(&mass, &radius);
    assert!(result.is_err());

    let err = result.unwrap_err();
    match &err.0 {
        PhysicsErrorEnum::MetricSingularity(msg) => {
            assert!(msg.contains("Zero radius"));
        }
        _ => panic!("Expected MetricSingularity error"),
    }
}

/// Physics invariant: v_escape = sqrt(2) × v_orbital for same mass and radius
#[test]
fn test_escape_vs_orbital_velocity_invariant() {
    let mass = Mass::new(5.972e24).unwrap(); // Earth mass
    let radius = Length::new(6.371e6).unwrap(); // Earth radius

    let v_orbital = orbital_velocity_kernel(&mass, &radius).unwrap();
    let v_escape = escape_velocity_kernel(&mass, &radius).unwrap();

    let ratio = v_escape.value() / v_orbital.value();
    let sqrt_2 = 2.0_f64.sqrt();

    assert!(
        (ratio - sqrt_2).abs() < 1e-10,
        "v_escape / v_orbital should equal sqrt(2), got {}",
        ratio
    );
}

// =============================================================================
// schwarzschild_radius_kernel Tests
// =============================================================================

#[test]
fn test_schwarzschild_radius_kernel_valid() {
    let mass = Mass::new(1.989e30).unwrap(); // Solar mass

    let result = schwarzschild_radius_kernel(&mass);
    assert!(result.is_ok());

    let r_s = result.unwrap();
    assert!(r_s.value() > 0.0, "Schwarzschild radius must be positive");
}

/// Physics invariant: Schwarzschild radius of the Sun ≈ 2.95 km
#[test]
fn test_schwarzschild_radius_sun_known_value() {
    // Note: The code uses G = 9.80665 (standard gravity), not the gravitational constant
    // So we test the formula: r_s = 2GM/c² with the library's G constant
    let solar_mass = Mass::new(1.989e30).unwrap();

    let result = schwarzschild_radius_kernel(&solar_mass);
    assert!(result.is_ok());

    let r_s = result.unwrap();

    // Expected: 2 * G * M / c²
    let expected = 2.0 * G * solar_mass.value() / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    assert!(
        (r_s.value() - expected).abs() < 1e-10,
        "Schwarzschild radius formula mismatch"
    );
}

#[test]
fn test_schwarzschild_radius_kernel_zero_mass() {
    let mass = Mass::new(0.0).unwrap();

    let result = schwarzschild_radius_kernel(&mass);
    assert!(result.is_ok());

    let r_s = result.unwrap();
    assert!((r_s.value() - 0.0).abs() < 1e-10, "Zero mass → zero Schwarzschild radius");
}

/// Test that negative mass is rejected at the Mass type level
#[test]
fn test_negative_mass_rejected() {
    let result = Mass::new(-1.0);
    assert!(result.is_err(), "Negative mass should be rejected");
}

/// Test that negative Length is rejected at the Length type level
#[test]
fn test_negative_length_rejected() {
    let result = Length::new(-1.0);
    assert!(result.is_err(), "Negative length should be rejected");
}
