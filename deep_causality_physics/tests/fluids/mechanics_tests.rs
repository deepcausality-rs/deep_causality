/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, G, Length, Pressure, Speed, bernoulli_pressure_kernel, hydrostatic_pressure_kernel,
};

// =============================================================================
// hydrostatic_pressure_kernel Tests
// =============================================================================

#[test]
fn test_hydrostatic_pressure_kernel_valid() {
    // P = P0 + ρgh
    let p0 = Pressure::new(101325.0).unwrap(); // 1 atm
    let density = Density::new(1000.0).unwrap(); // water
    let depth = Length::new(10.0).unwrap(); // 10 meters

    let result = hydrostatic_pressure_kernel(&p0, &density, &depth);
    assert!(result.is_ok());

    let p_total = result.unwrap();
    // Expected: 101325 + 1000 * 9.80665 * 10 ≈ 199391.5
    let expected = p0.value() + density.value() * G * depth.value();
    assert!(
        (p_total.value() - expected).abs() < 1.0,
        "Expected {}, got {}",
        expected,
        p_total.value()
    );
}

#[test]
fn test_hydrostatic_pressure_kernel_zero_depth() {
    let p0 = Pressure::new(101325.0).unwrap();
    let density = Density::new(1000.0).unwrap();
    let depth = Length::new(0.0).unwrap();

    let result = hydrostatic_pressure_kernel(&p0, &density, &depth);
    assert!(result.is_ok());

    let p_total = result.unwrap();
    assert!(
        (p_total.value() - 101325.0).abs() < 1e-10,
        "Zero depth should give P = P0"
    );
}

// =============================================================================
// bernoulli_pressure_kernel Tests
// =============================================================================

#[test]
fn test_bernoulli_pressure_kernel_valid() {
    // P2 = P1 + 0.5*ρ*(v1² - v2²) + ρ*g*(h1 - h2)
    let p1 = Pressure::new(100000.0).unwrap();
    let v1 = Speed::new(5.0).unwrap();
    let h1 = Length::new(10.0).unwrap();
    let v2 = Speed::new(10.0).unwrap();
    let h2 = Length::new(5.0).unwrap();
    let density = Density::new(1000.0).unwrap();

    let result = bernoulli_pressure_kernel(&p1, &v1, &h1, &v2, &h2, &density);
    assert!(result.is_ok());

    let p2 = result.unwrap();
    // Verify using formula
    let rho = density.value();
    let kinetic_term = 0.5 * rho * (v1.value().powi(2) - v2.value().powi(2));
    let potential_term = rho * G * (h1.value() - h2.value());
    let expected = p1.value() + kinetic_term + potential_term;

    assert!(
        (p2.value() - expected).abs() < 1.0,
        "Expected {}, got {}",
        expected,
        p2.value()
    );
}

/// Bernoulli at same height and velocity should give same pressure
#[test]
fn test_bernoulli_pressure_kernel_same_height_velocity() {
    let p1 = Pressure::new(50000.0).unwrap();
    let v = Speed::new(10.0).unwrap();
    let h = Length::new(5.0).unwrap();
    let density = Density::new(1000.0).unwrap();

    let result = bernoulli_pressure_kernel(&p1, &v, &h, &v, &h, &density);
    assert!(result.is_ok());

    let p2 = result.unwrap();
    assert!(
        (p2.value() - p1.value()).abs() < 1e-10,
        "Same conditions should give P2 = P1"
    );
}

/// Physics invariant: Energy conservation - faster flow = lower pressure (Venturi effect)
#[test]
fn test_bernoulli_venturi_effect() {
    let p1 = Pressure::new(100000.0).unwrap();
    let v1 = Speed::new(2.0).unwrap();
    let v2 = Speed::new(10.0).unwrap(); // Faster flow
    let h = Length::new(0.0).unwrap(); // Same height
    let density = Density::new(1000.0).unwrap();

    let result = bernoulli_pressure_kernel(&p1, &v1, &h, &v2, &h, &density);
    assert!(result.is_ok());

    let p2 = result.unwrap();
    assert!(
        p2.value() < p1.value(),
        "Faster flow should result in lower pressure (Venturi effect)"
    );
}
