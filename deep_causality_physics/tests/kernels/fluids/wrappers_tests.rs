/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    CauchyStress, Density, KinematicViscosity, Length, Pressure, Speed, Velocity3,
    VelocityGradient, VorticityVector, bernoulli_pressure, continuity_rhs, convective_acceleration,
    enstrophy_density, helicity_density, hydrostatic_pressure, kinetic_energy_density,
    pressure_gradient_force, pressure_work, rotation_rate_tensor, scalar_advection_diffusion,
    strain_rate_tensor, velocity_gradient_invariants, viscous_diffusion, viscous_dissipation_rate,
    vorticity_from_gradient, vorticity_transport,
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

// =============================================================================
// Governing kernel wrapper tests
// =============================================================================

#[test]
fn test_convective_acceleration_wrapper_success() {
    let u = Velocity3::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let effect = convective_acceleration(&u, &g);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_viscous_diffusion_wrapper_success() {
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let effect = viscous_diffusion(&nu, &[2.0, 0.0, 0.0]);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 1.0).abs() < 1e-12);
}

#[test]
fn test_pressure_gradient_force_wrapper_success() {
    let rho = Density::<f64>::new(1000.0).unwrap();
    let effect = pressure_gradient_force(&rho, &[10.0, 0.0, 0.0]);
    assert!(effect.is_ok());
}

#[test]
fn test_pressure_gradient_force_wrapper_error_path() {
    let rho = Density::<f64>::new(0.0).unwrap();
    let effect = pressure_gradient_force(&rho, &[1.0, 0.0, 0.0]);
    assert!(!effect.is_ok());
}

#[test]
fn test_continuity_rhs_wrapper_success() {
    let rho = Density::<f64>::new(1.0).unwrap();
    let u = Velocity3::<f64>::new([0.0; 3]).unwrap();
    let effect = continuity_rhs(&rho, &u, &[0.0; 3], 0.0);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 0.0);
}

#[test]
fn test_vorticity_transport_wrapper_success() {
    let omega = VorticityVector::<f64>::new([1.0, 0.0, 0.0]).unwrap();
    let u = Velocity3::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let grad_omega = [[0.0; 3]; 3];
    let lap_omega = [4.0, 0.0, 0.0];
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let effect = vorticity_transport(&omega, &u, &grad_u, &grad_omega, &lap_omega, &nu);
    assert!(effect.is_ok());
    let a = effect.value().clone().into_value().unwrap();
    assert!((a.value()[0] - 2.0).abs() < 1e-12);
}

#[test]
fn test_scalar_advection_diffusion_wrapper_success() {
    let u = Velocity3::<f64>::default();
    let effect = scalar_advection_diffusion(&u, &[0.0; 3], 0.0, 0.0, 7.5);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 7.5);
}

#[test]
fn test_kinetic_energy_density_wrapper_success() {
    let rho = Density::<f64>::new(2.0).unwrap();
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    let effect = kinetic_energy_density(&rho, &u);
    assert!(effect.is_ok());
    let e = effect.value().clone().into_value().unwrap();
    assert!((e - 25.0).abs() < 1e-12);
}

#[test]
fn test_viscous_dissipation_rate_wrapper_success() {
    let tau =
        CauchyStress::<f64>::new([[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]]).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let effect = viscous_dissipation_rate(&tau, &g);
    assert!(effect.is_ok());
    let phi = effect.value().clone().into_value().unwrap();
    assert!((phi - 10.0).abs() < 1e-12);
}

#[test]
fn test_pressure_work_wrapper_success() {
    let p = Pressure::<f64>::new(2.0).unwrap();
    let effect = pressure_work(&p, 3.0);
    assert!(effect.is_ok());
    assert_eq!(effect.value().clone().into_value().unwrap(), 6.0);
}
