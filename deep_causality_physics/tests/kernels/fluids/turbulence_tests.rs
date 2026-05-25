/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    KinematicViscosity, ReynoldsStress, StrainRateTensor, Velocity3, VelocityGradient,
    dissipation_rate_kernel, eddy_viscosity_boussinesq_kernel, integral_length_scale_kernel,
    kolmogorov_length_kernel, kolmogorov_time_kernel, kolmogorov_velocity_kernel,
    reynolds_stress_kernel, taylor_microscale_kernel, turbulent_kinetic_energy_kernel,
};

const TOL: f64 = 1e-10;

// =============================================================================
// turbulent_kinetic_energy_kernel
// =============================================================================

#[test]
fn test_tke_known_value() {
    let u = Velocity3::<f64>::new([3.0, 4.0, 0.0]).unwrap();
    // k = 0.5 * (9 + 16 + 0) = 12.5
    let k = turbulent_kinetic_energy_kernel(&u).unwrap();
    assert!((k - 12.5).abs() < TOL);
}

#[test]
fn test_tke_zero_for_zero_velocity() {
    let u = Velocity3::<f64>::default();
    assert_eq!(turbulent_kinetic_energy_kernel(&u).unwrap(), 0.0);
}

#[test]
fn test_tke_nonneg() {
    for raw in [[1.0, 0.0, 0.0], [-1.0, 2.0, -3.0]] {
        let u = Velocity3::<f64>::new(raw).unwrap();
        assert!(turbulent_kinetic_energy_kernel(&u).unwrap() >= 0.0);
    }
}

// =============================================================================
// dissipation_rate_kernel
// =============================================================================

#[test]
fn test_dissipation_known_value() {
    // Diagonal velocity gradient: ∂u_x/∂x = 1, others 0.
    // S'_00 = 1, rest 0. sum(S':S') = 1.
    // ε = 2 * ν * 1 = 2ν.
    let nu = KinematicViscosity::<f64>::new(0.5).unwrap();
    let g =
        VelocityGradient::<f64>::new([[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    let eps = dissipation_rate_kernel(&nu, &g).unwrap();
    assert!((eps - 1.0).abs() < TOL);
}

#[test]
fn test_dissipation_zero_for_rigid_body_rotation() {
    // Antisymmetric ∇u => S' = 0 => ε = 0.
    let nu = KinematicViscosity::<f64>::new(1.0).unwrap();
    let g = VelocityGradient::<f64>::new([[0.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]])
        .unwrap();
    let eps = dissipation_rate_kernel(&nu, &g).unwrap();
    assert!(eps.abs() < TOL);
}

#[test]
fn test_dissipation_nonneg() {
    // Arbitrary mixed gradient.
    let nu = KinematicViscosity::<f64>::new(1.0e-3).unwrap();
    let g = VelocityGradient::<f64>::new([[0.5, 1.0, -2.0], [3.0, 0.0, 0.5], [-1.5, 4.0, 2.0]])
        .unwrap();
    let eps = dissipation_rate_kernel(&nu, &g).unwrap();
    assert!(eps >= 0.0);
}

// =============================================================================
// Kolmogorov scales + algebraic identities
// =============================================================================

#[test]
fn test_kolmogorov_identity_eta_u_eta_over_nu() {
    // η · u_η / ν == 1
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let eps = 1.0e-3_f64;
    let eta = kolmogorov_length_kernel(&nu, eps).unwrap();
    let u_eta = kolmogorov_velocity_kernel(&nu, eps).unwrap();
    let ratio = eta.value() * u_eta.value() / nu.value();
    assert!((ratio - 1.0).abs() < TOL);
}

#[test]
fn test_kolmogorov_identity_eta_over_u_eta_tau_eta() {
    // η / (u_η · τ_η) == 1
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    let eps = 2.5e-4_f64;
    let eta = kolmogorov_length_kernel(&nu, eps).unwrap();
    let u_eta = kolmogorov_velocity_kernel(&nu, eps).unwrap();
    let tau_eta = kolmogorov_time_kernel(&nu, eps).unwrap();
    let ratio = eta.value() / (u_eta.value() * tau_eta);
    assert!((ratio - 1.0).abs() < TOL);
}

#[test]
fn test_kolmogorov_length_errors_on_zero_epsilon() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(kolmogorov_length_kernel(&nu, 0.0).is_err());
    assert!(kolmogorov_length_kernel(&nu, -1.0).is_err());
}

#[test]
fn test_kolmogorov_time_errors_on_zero_epsilon() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(kolmogorov_time_kernel(&nu, 0.0).is_err());
}

#[test]
fn test_kolmogorov_velocity_errors_on_zero_epsilon() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(kolmogorov_velocity_kernel(&nu, 0.0).is_err());
}

#[test]
fn test_kolmogorov_errors_on_zero_nu() {
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    assert!(kolmogorov_length_kernel(&nu, 1.0_f64).is_err());
    assert!(kolmogorov_time_kernel(&nu, 1.0_f64).is_err());
    assert!(kolmogorov_velocity_kernel(&nu, 1.0_f64).is_err());
}

// =============================================================================
// Taylor microscale + Block-B5 algebraic identity
// =============================================================================

#[test]
fn test_taylor_identity_lambda_sq_eps_eq_15_nu_k() {
    // λ² · ε == 15 · ν · k
    let nu = KinematicViscosity::<f64>::new(1.5e-5).unwrap();
    let k = 2.0_f64;
    let eps = 1.0e-2_f64;
    let lambda = taylor_microscale_kernel(k, eps, &nu).unwrap();
    let lhs = lambda.value() * lambda.value() * eps;
    let rhs = 15.0 * nu.value() * k;
    assert!((lhs - rhs).abs() < TOL * rhs);
}

#[test]
fn test_taylor_errors_on_negative_k() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(taylor_microscale_kernel(-1.0_f64, 1.0e-2, &nu).is_err());
}

#[test]
fn test_taylor_errors_on_zero_epsilon() {
    let nu = KinematicViscosity::<f64>::new(1.0e-5).unwrap();
    assert!(taylor_microscale_kernel(1.0_f64, 0.0, &nu).is_err());
}

// =============================================================================
// Integral length scale
// =============================================================================

#[test]
fn test_integral_length_scale_known_value() {
    // L = k^(3/2) / ε with k = 4, ε = 8: L = 8 / 8 = 1.
    let l = integral_length_scale_kernel(4.0_f64, 8.0).unwrap();
    assert!((l.value() - 1.0).abs() < TOL);
}

#[test]
fn test_integral_length_scale_errors_on_zero_epsilon() {
    assert!(integral_length_scale_kernel(1.0_f64, 0.0).is_err());
}

#[test]
fn test_integral_length_scale_errors_on_negative_k() {
    assert!(integral_length_scale_kernel(-1.0_f64, 1.0).is_err());
}

// =============================================================================
// reynolds_stress_kernel
// =============================================================================

#[test]
fn test_reynolds_stress_round_trip() {
    let raw = [[1.0, 0.5, 0.2], [0.5, 2.0, -0.1], [0.2, -0.1, 0.8]];
    let r_in = StrainRateTensor::<f64>::new(raw).unwrap();
    let stress = reynolds_stress_kernel(&r_in);
    assert_eq!(stress.value(), &raw);
}

#[test]
fn test_reynolds_stress_trace_equals_2k() {
    // R_ii = ⟨u'·u'⟩ = 2k (identity by definition).
    let u = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let k = turbulent_kinetic_energy_kernel(&u).unwrap();
    let raw = [[1.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 9.0]];
    let r_in = StrainRateTensor::<f64>::new(raw).unwrap();
    let stress = reynolds_stress_kernel(&r_in);
    let trace = stress.value()[0][0] + stress.value()[1][1] + stress.value()[2][2];
    assert!((trace - 2.0 * k).abs() < TOL);
}

// =============================================================================
// eddy_viscosity_boussinesq_kernel
// =============================================================================

#[test]
fn test_eddy_viscosity_simple_shear() {
    // Canonical shear flow: u(y), so ∇u has S_xy = S_yx = 0.5·γ.
    // Reynolds stress: R_xy = -μ_t·γ (and symmetric). With k > 0 and
    // diagonal R = (2/3)k I, the deviatoric R has only off-diagonal entries.
    let gamma = 2.0;
    let k = 1.0;
    let nu_t_expected = 0.05; // pick a target ν_t
    // R_xy = -(2 ν_t) · S_xy = -(2 * 0.05) * (0.5 * 2) = -0.1
    let r_xy = -(2.0 * nu_t_expected) * (0.5 * gamma);
    let r = ReynoldsStress::<f64>::new([
        [(2.0 / 3.0) * k, r_xy, 0.0],
        [r_xy, (2.0 / 3.0) * k, 0.0],
        [0.0, 0.0, (2.0 / 3.0) * k],
    ])
    .unwrap();
    let s = StrainRateTensor::<f64>::new([
        [0.0, 0.5 * gamma, 0.0],
        [0.5 * gamma, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ])
    .unwrap();
    let nu_t = eddy_viscosity_boussinesq_kernel(&r, &s, k).unwrap();
    assert!((nu_t.value() - nu_t_expected).abs() < TOL);
}

#[test]
fn test_eddy_viscosity_errors_on_zero_strain() {
    let r = ReynoldsStress::<f64>::default();
    let s = StrainRateTensor::<f64>::default();
    assert!(eddy_viscosity_boussinesq_kernel(&r, &s, 0.5).is_err());
}

#[test]
fn test_eddy_viscosity_errors_on_negative_result() {
    // Choose R^dev:S > 0 so that -R^dev:S < 0 => ν_t < 0 => Viscosity::new rejects.
    let r =
        ReynoldsStress::<f64>::new([[1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [0.0, 0.0, 1.0]]).unwrap();
    let s =
        StrainRateTensor::<f64>::new([[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 0.0]]).unwrap();
    // R^dev = R - (2/3)·0 ·I = R. R^dev:S = 2·1·1 + 2·1·1 = 4 (off-diagonals doubled). S:S = 2.
    // ν_t = -4 / (2·2) = -1 → rejected by Viscosity::new.
    assert!(eddy_viscosity_boussinesq_kernel(&r, &s, 0.0).is_err());
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_kolmogorov_identity_f32() {
    let nu = KinematicViscosity::<f32>::new(1.5e-5).unwrap();
    let eps = 1.0e-3_f32;
    let eta = kolmogorov_length_kernel(&nu, eps).unwrap();
    let u_eta = kolmogorov_velocity_kernel(&nu, eps).unwrap();
    let ratio = eta.value() * u_eta.value() / nu.value();
    assert!((ratio - 1.0).abs() < 1.0e-4);
}

#[test]
fn test_taylor_identity_f32() {
    let nu = KinematicViscosity::<f32>::new(1.5e-5).unwrap();
    let k = 2.0_f32;
    let eps = 1.0e-2_f32;
    let lambda = taylor_microscale_kernel(k, eps, &nu).unwrap();
    let lhs = lambda.value() * lambda.value() * eps;
    let rhs = 15.0 * nu.value() * k;
    assert!((lhs - rhs).abs() < 1.0e-4 * rhs);
}
