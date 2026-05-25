/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, KinematicViscosity, Length, Speed, Viscosity, WallShearStress,
    friction_velocity_kernel, log_law_velocity_kernel, skin_friction_coefficient_kernel,
    viscous_length_scale_kernel, viscous_sublayer_velocity_kernel,
    wall_shear_stress_newtonian_kernel, y_plus_kernel,
};

const TOL: f64 = 1e-10;

// =============================================================================
// wall_shear_stress_newtonian_kernel
// =============================================================================

#[test]
fn test_wall_shear_stress_known_value() {
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    let tau_w = wall_shear_stress_newtonian_kernel(&mu, 100.0_f64).unwrap();
    // τ_w = 1e-3 · 100 = 0.1
    assert!((tau_w.value() - 0.1).abs() < TOL);
}

#[test]
fn test_wall_shear_stress_uses_magnitude() {
    let mu = Viscosity::<f64>::new(0.5).unwrap();
    let tau_positive = wall_shear_stress_newtonian_kernel(&mu, 2.0).unwrap();
    let tau_negative = wall_shear_stress_newtonian_kernel(&mu, -2.0).unwrap();
    assert_eq!(tau_positive.value(), tau_negative.value());
}

#[test]
fn test_wall_shear_stress_zero_for_zero_gradient() {
    let mu = Viscosity::<f64>::new(1.0).unwrap();
    let tau_w = wall_shear_stress_newtonian_kernel(&mu, 0.0_f64).unwrap();
    assert_eq!(tau_w.value(), 0.0);
}

#[test]
fn test_wall_shear_stress_errors_on_non_finite_gradient() {
    let mu = Viscosity::<f64>::new(1.0e-3).unwrap();
    assert!(wall_shear_stress_newtonian_kernel(&mu, f64::NAN).is_err());
    assert!(wall_shear_stress_newtonian_kernel(&mu, f64::INFINITY).is_err());
    assert!(wall_shear_stress_newtonian_kernel(&mu, f64::NEG_INFINITY).is_err());
}

// =============================================================================
// friction_velocity_kernel
// =============================================================================

#[test]
fn test_friction_velocity_known_value() {
    let tau = WallShearStress::<f64>::new(0.1).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    // u_τ = sqrt(0.1/1) = sqrt(0.1) ≈ 0.3162
    let u_tau = friction_velocity_kernel(&tau, &rho).unwrap();
    assert!((u_tau.value() - 0.1_f64.sqrt()).abs() < TOL);
}

#[test]
fn test_friction_velocity_errors_on_zero_density() {
    let tau = WallShearStress::<f64>::new(1.0).unwrap();
    let rho = Density::<f64>::new(0.0).unwrap();
    assert!(friction_velocity_kernel(&tau, &rho).is_err());
}

#[test]
fn test_friction_velocity_zero_for_zero_stress() {
    let tau = WallShearStress::<f64>::new(0.0).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_tau = friction_velocity_kernel(&tau, &rho).unwrap();
    assert_eq!(u_tau.value(), 0.0);
}

// =============================================================================
// viscous_length_scale_kernel
// =============================================================================

#[test]
fn test_viscous_length_scale_known_value() {
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    // δ_ν = 1.5e-5 / 0.5 = 3e-5
    let delta_nu = viscous_length_scale_kernel(&nu, &u_tau).unwrap();
    assert!((delta_nu.value() - 3.0e-5).abs() < TOL);
}

#[test]
fn test_viscous_length_scale_errors_on_zero_u_tau() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    let u_tau = Speed::<f64>::new(0.0).unwrap();
    assert!(viscous_length_scale_kernel(&nu, &u_tau).is_err());
}

// =============================================================================
// y_plus_kernel
// =============================================================================

#[test]
fn test_y_plus_known_value() {
    let y = Length::<f64>::new(1.0e-4).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    // y⁺ = 1e-4 · 0.5 / 1.5e-5 = 5e-5 / 1.5e-5 ≈ 3.333
    let yp = y_plus_kernel(&y, &u_tau, &nu).unwrap();
    let expected = 1.0e-4 * 0.5 / 1.5e-5;
    assert!((yp - expected).abs() < TOL);
}

#[test]
fn test_y_plus_scales_linearly_with_y() {
    // Spec scenario: y⁺ scales linearly with wall distance.
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let y_ref = Length::<f64>::new(1.0e-4).unwrap();
    let yp_ref = y_plus_kernel(&y_ref, &u_tau, &nu).unwrap();
    for k in [0.5_f64, 1.0, 2.0, 10.0] {
        let y_k = Length::<f64>::new(1.0e-4 * k).unwrap();
        let yp_k = y_plus_kernel(&y_k, &u_tau, &nu).unwrap();
        assert!((yp_k - k * yp_ref).abs() < TOL * yp_ref.abs());
    }
}

#[test]
fn test_y_plus_errors_on_zero_viscosity() {
    let y = Length::<f64>::new(1.0).unwrap();
    let u_tau = Speed::<f64>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(y_plus_kernel(&y, &u_tau, &nu).is_err());
}

// =============================================================================
// viscous_sublayer_velocity_kernel + log_law_velocity_kernel
// =============================================================================

#[test]
fn test_viscous_sublayer_law_is_identity() {
    for yp in [0.1_f64, 1.0, 5.0] {
        assert_eq!(viscous_sublayer_velocity_kernel(yp), yp);
    }
}

#[test]
fn test_log_law_known_value_at_y_plus_100() {
    // κ = 0.41, B = 5.0, y⁺ = 100: u⁺ = (1/0.41)·ln(100) + 5
    //   = ln(100)/0.41 + 5 ≈ 11.231 + 5 = 16.231
    let u_plus = log_law_velocity_kernel(100.0_f64, 0.41, 5.0).unwrap();
    let expected = 100.0_f64.ln() / 0.41 + 5.0;
    assert!((u_plus - expected).abs() < TOL);
    assert!((u_plus - 16.231).abs() < 0.01);
}

#[test]
fn test_log_law_errors_on_nonpositive_y_plus() {
    assert!(log_law_velocity_kernel(0.0_f64, 0.41, 5.0).is_err());
    assert!(log_law_velocity_kernel(-1.0_f64, 0.41, 5.0).is_err());
}

#[test]
fn test_log_law_errors_on_zero_kappa() {
    assert!(log_law_velocity_kernel(100.0_f64, 0.0, 5.0).is_err());
}

#[test]
fn test_sublayer_and_log_law_differ_in_buffer_region() {
    // Spec scenario: at y⁺ = 11.5 (buffer layer) the two laws give
    // different velocities and the caller chooses which to apply.
    let yp = 11.5_f64;
    let u_sublayer = viscous_sublayer_velocity_kernel(yp);
    let u_log = log_law_velocity_kernel(yp, 0.41, 5.0).unwrap();
    assert!((u_sublayer - u_log).abs() > 0.1);
}

// =============================================================================
// skin_friction_coefficient_kernel
// =============================================================================

#[test]
fn test_skin_friction_coefficient_known_value() {
    let tau = WallShearStress::<f64>::new(0.5).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_inf = Speed::<f64>::new(10.0).unwrap();
    // C_f = 0.5 / (0.5 · 1 · 100) = 0.5 / 50 = 0.01
    let cf = skin_friction_coefficient_kernel(&tau, &rho, &u_inf).unwrap();
    assert!((cf - 0.01).abs() < TOL);
}

#[test]
fn test_skin_friction_errors_on_zero_density() {
    let tau = WallShearStress::<f64>::new(1.0).unwrap();
    let rho = Density::<f64>::new(0.0).unwrap();
    let u_inf = Speed::<f64>::new(1.0).unwrap();
    assert!(skin_friction_coefficient_kernel(&tau, &rho, &u_inf).is_err());
}

#[test]
fn test_skin_friction_errors_on_zero_u_inf() {
    let tau = WallShearStress::<f64>::new(1.0).unwrap();
    let rho = Density::<f64>::new(1.0).unwrap();
    let u_inf = Speed::<f64>::new(0.0).unwrap();
    assert!(skin_friction_coefficient_kernel(&tau, &rho, &u_inf).is_err());
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_y_plus_linear_scaling_f32() {
    let u_tau = Speed::<f32>::new(0.5).unwrap();
    let nu = KinematicViscosity::<f32>::new(1.5e-5).unwrap();
    let y_ref = Length::<f32>::new(1.0e-4).unwrap();
    let yp_ref = y_plus_kernel(&y_ref, &u_tau, &nu).unwrap();
    let y2 = Length::<f32>::new(2.0e-4).unwrap();
    let yp2 = y_plus_kernel(&y2, &u_tau, &nu).unwrap();
    assert!((yp2 - 2.0 * yp_ref).abs() < 1.0e-3 * yp_ref.abs());
}

#[test]
fn test_log_law_f32() {
    let u_plus = log_law_velocity_kernel(100.0_f32, 0.41, 5.0).unwrap();
    let expected = 100.0_f32.ln() / 0.41 + 5.0;
    assert!((u_plus - expected).abs() < 1.0e-5);
}
