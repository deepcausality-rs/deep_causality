/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, Length, Pressure, Speed, Velocity3, VelocityGradient, VorticityVector,
    bernoulli_pressure, enstrophy_density, helicity_density, hydrostatic_pressure,
    rotation_rate_tensor, strain_rate_tensor, velocity_gradient_invariants,
    vorticity_from_gradient,
};

// =============================================================================
// hydrostatic_pressure Wrapper Tests
// =============================================================================

#[test]
fn test_hydrostatic_pressure_wrapper_success() {
    let p0 = Pressure::new(101325.0).unwrap();
    let density = Density::new(1000.0).unwrap();
    let depth = Length::new(10.0).unwrap();

    let effect = hydrostatic_pressure(&p0, &density, &depth);
    assert!(effect.is_ok());

    let p = effect.value().clone().into_value().unwrap();
    assert!(p.value() > p0.value());
}

// =============================================================================
// bernoulli_pressure Wrapper Tests
// =============================================================================

#[test]
fn test_bernoulli_pressure_wrapper_success() {
    let p1 = Pressure::new(100000.0).unwrap();
    let v1 = Speed::new(5.0).unwrap();
    let h1 = Length::new(10.0).unwrap();
    let v2 = Speed::new(10.0).unwrap();
    let h2 = Length::new(5.0).unwrap();
    let density = Density::new(1000.0).unwrap();

    let effect = bernoulli_pressure(&p1, &v1, &h1, &v2, &h2, &density);
    assert!(effect.is_ok());
}

// =============================================================================
// Kinematic wrapper tests
// =============================================================================

#[test]
fn test_strain_rate_tensor_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let effect = strain_rate_tensor(&g);
    assert!(effect.is_ok());
    let s = effect.value().clone().into_value().unwrap();
    let raw = s.value();
    // Symmetric by construction
    assert!((raw[0][1] - raw[1][0]).abs() < 1e-12);
}

#[test]
fn test_rotation_rate_tensor_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]).unwrap();
    let effect = rotation_rate_tensor(&g);
    assert!(effect.is_ok());
    let o = effect.value().clone().into_value().unwrap();
    // Antisymmetric: diagonal vanishes
    assert!(o.value()[0][0].abs() < 1e-12);
}

#[test]
fn test_vorticity_from_gradient_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[0.0, 0.0, 0.0], [0.0, 0.0, -0.5], [0.0, 0.5, 0.0]]).unwrap();
    let effect = vorticity_from_gradient(&g);
    assert!(effect.is_ok());
    let w = effect.value().clone().into_value().unwrap();
    assert!((w.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_velocity_gradient_invariants_wrapper_success() {
    let g =
        VelocityGradient::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let effect = velocity_gradient_invariants(&g);
    assert!(effect.is_ok());
    let (p, _q, _r) = effect.value().clone().into_value().unwrap();
    assert!((p - (-10.0)).abs() < 1e-12);
}

#[test]
fn test_helicity_density_wrapper_success() {
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let w = VorticityVector::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    let effect = helicity_density(&u, &w);
    assert!(effect.is_ok());
    let h = effect.value().clone().into_value().unwrap();
    assert!((h - 32.0).abs() < 1e-12);
}

#[test]
fn test_enstrophy_density_wrapper_success() {
    let w = VorticityVector::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = enstrophy_density(&w);
    assert!(effect.is_ok());
    let e = effect.value().clone().into_value().unwrap();
    assert!((e - 12.5).abs() < 1e-12);
}
