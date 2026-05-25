/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    CauchyStress, Pressure, SpecificEnthalpy, Temperature, Velocity3, VelocityGradient,
    entropy_production_rate_kernel, specific_enthalpy_kernel, speed_of_sound_ideal_gas_kernel,
    total_enthalpy_kernel, total_pressure_isentropic_kernel, total_temperature_isentropic_kernel,
};

const TOL: f64 = 1e-6;

// =============================================================================
// speed_of_sound_ideal_gas_kernel
// =============================================================================

#[test]
fn test_speed_of_sound_air_at_room_temperature() {
    // Air at 20°C: γ ≈ 1.4, R_s ≈ 287.05 J/(kg·K), T = 293.15 K.
    // a = sqrt(1.4 · 287.05 · 293.15) ≈ 343.21 m/s.
    let t = Temperature::<f64>::new(293.15).unwrap();
    let a = speed_of_sound_ideal_gas_kernel(1.4, 287.05, &t).unwrap();
    let expected = (1.4 * 287.05 * 293.15_f64).sqrt();
    assert!((a.value() - expected).abs() < TOL);
    // sanity: should be around 343 m/s
    assert!((a.value() - 343.21).abs() < 0.1);
}

#[test]
fn test_speed_of_sound_errors_on_zero_temperature() {
    let t = Temperature::<f64>::new(0.0).unwrap();
    assert!(speed_of_sound_ideal_gas_kernel(1.4, 287.0, &t).is_err());
}

#[test]
fn test_speed_of_sound_errors_on_negative_gamma() {
    let t = Temperature::<f64>::new(293.15).unwrap();
    assert!(speed_of_sound_ideal_gas_kernel(-1.0, 287.0, &t).is_err());
}

// =============================================================================
// specific_enthalpy_kernel
// =============================================================================

#[test]
fn test_specific_enthalpy_known_value() {
    let t = Temperature::<f64>::new(300.0).unwrap();
    let h = specific_enthalpy_kernel(1005.0_f64, &t);
    // h = 1005 · 300 = 301_500 J/kg
    assert!((h.value() - 301_500.0).abs() < TOL);
}

#[test]
fn test_specific_enthalpy_zero_at_zero_kelvin() {
    let t = Temperature::<f64>::new(0.0).unwrap();
    let h = specific_enthalpy_kernel(1005.0_f64, &t);
    assert_eq!(h.value(), 0.0);
}

// =============================================================================
// total_enthalpy_kernel
// =============================================================================

#[test]
fn test_total_enthalpy_at_rest_equals_static() {
    let h = SpecificEnthalpy::<f64>::new(3.0e5).unwrap();
    let u = Velocity3::<f64>::default();
    let h0 = total_enthalpy_kernel(&h, &u).unwrap();
    assert_eq!(h0.value(), h.value());
}

#[test]
fn test_total_enthalpy_kinetic_contribution() {
    let h = SpecificEnthalpy::<f64>::new(3.0e5).unwrap();
    let u = Velocity3::<f64>::new([100.0, 0.0, 0.0]).unwrap();
    // h_0 = 3e5 + 0.5 · 100² = 3e5 + 5000
    let h0 = total_enthalpy_kernel(&h, &u).unwrap();
    assert!((h0.value() - (3.0e5 + 5_000.0)).abs() < TOL);
}

// =============================================================================
// total_temperature_isentropic_kernel  (spec scenario)
// =============================================================================

#[test]
fn test_total_temperature_at_zero_mach_equals_static() {
    // Spec scenario: T_0 = T at M = 0.
    let t = Temperature::<f64>::new(293.15).unwrap();
    let t0 = total_temperature_isentropic_kernel(&t, 0.0, 1.4).unwrap();
    assert!((t0.value() - t.value()).abs() < TOL);
}

#[test]
fn test_total_temperature_at_mach_one_for_air() {
    // For air (γ = 1.4) at M = 1: T_0/T = 1 + 0.4/2 · 1 = 1.2.
    let t = Temperature::<f64>::new(300.0).unwrap();
    let t0 = total_temperature_isentropic_kernel(&t, 1.0, 1.4).unwrap();
    assert!((t0.value() - 360.0).abs() < TOL);
}

#[test]
fn test_total_temperature_errors_on_gamma_le_1() {
    let t = Temperature::<f64>::new(300.0).unwrap();
    assert!(total_temperature_isentropic_kernel(&t, 1.0, 1.0_f64).is_err());
    assert!(total_temperature_isentropic_kernel(&t, 1.0, 0.5_f64).is_err());
}

// =============================================================================
// total_pressure_isentropic_kernel
// =============================================================================

#[test]
fn test_total_pressure_at_zero_mach_equals_static() {
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    let p0 = total_pressure_isentropic_kernel(&p, 0.0, 1.4).unwrap();
    assert!((p0.value() - p.value()).abs() < 1e-6);
}

#[test]
fn test_total_pressure_at_mach_one_for_air() {
    // For air (γ = 1.4) at M = 1: p_0/p = 1.2^(1.4/0.4) = 1.2^3.5 ≈ 1.8929
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    let p0 = total_pressure_isentropic_kernel(&p, 1.0, 1.4).unwrap();
    let ratio = p0.value() / p.value();
    let expected = 1.2_f64.powf(3.5);
    assert!((ratio - expected).abs() < TOL);
}

#[test]
fn test_total_pressure_errors_on_gamma_le_1() {
    let p = Pressure::<f64>::new(101_325.0).unwrap();
    assert!(total_pressure_isentropic_kernel(&p, 1.0, 1.0_f64).is_err());
}

// =============================================================================
// entropy_production_rate_kernel  (spec scenario)
// =============================================================================

#[test]
fn test_entropy_production_nonneg_for_newtonian_inputs() {
    // For a Newtonian fluid, both Φ/T and κ‖∇T‖²/T² are non-negative.
    // Take τ = 2μS for a simple shear flow.
    let mu = 0.01;
    let gamma_dot = 2.0;
    // S = [[0, γ̇/2, 0], [γ̇/2, 0, 0], [0, 0, 0]]
    // τ = 2μS = [[0, μγ̇, 0], [μγ̇, 0, 0], [0, 0, 0]]
    let tau = CauchyStress::<f64>::new([
        [0.0, mu * gamma_dot, 0.0],
        [mu * gamma_dot, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ])
    .unwrap();
    let grad_u =
        VelocityGradient::<f64>::new([[0.0, gamma_dot, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]])
            .unwrap();
    let t = Temperature::<f64>::new(300.0).unwrap();
    let sigma = entropy_production_rate_kernel(&t, &tau, &grad_u, 0.0, &[0.0; 3]).unwrap();
    assert!(sigma >= 0.0);
    // Φ = τ:∇u = μγ̇·γ̇ = μ·γ̇² = 0.01·4 = 0.04. σ = 0.04/300 ≈ 1.333e-4
    assert!((sigma - 0.04 / 300.0).abs() < TOL);
}

#[test]
fn test_entropy_production_includes_heat_conduction_term() {
    // Quiescent fluid (no viscous dissipation), but with a temperature gradient
    // ⇒ entropy production via heat conduction only.
    let tau = CauchyStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(300.0).unwrap();
    let kappa = 0.0257_f64; // air thermal conductivity
    let grad_t = [10.0_f64, 0.0, 0.0]; // K/m
    let sigma = entropy_production_rate_kernel(&t, &tau, &grad_u, kappa, &grad_t).unwrap();
    // σ = κ·|∇T|²/T² = 0.0257·100/90000
    let expected = 0.0257 * 100.0 / (300.0 * 300.0);
    assert!((sigma - expected).abs() < TOL);
    assert!(sigma > 0.0);
}

#[test]
fn test_entropy_production_errors_on_zero_temperature() {
    let tau = CauchyStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(0.0).unwrap();
    let r = entropy_production_rate_kernel(&t, &tau, &grad_u, 1.0_f64, &[0.0; 3]);
    assert!(r.is_err());
}

// =============================================================================
// f32 precision sweep
// =============================================================================

#[test]
fn test_speed_of_sound_f32() {
    let t = Temperature::<f32>::new(293.15).unwrap();
    let a = speed_of_sound_ideal_gas_kernel(1.4_f32, 287.05, &t).unwrap();
    // expected ~343.21 m/s
    assert!((a.value() - 343.21_f32).abs() < 0.5);
}

#[test]
fn test_total_temperature_f32() {
    let t = Temperature::<f32>::new(300.0).unwrap();
    let t0 = total_temperature_isentropic_kernel(&t, 1.0_f32, 1.4).unwrap();
    assert!((t0.value() - 360.0_f32).abs() < 1e-3);
}
