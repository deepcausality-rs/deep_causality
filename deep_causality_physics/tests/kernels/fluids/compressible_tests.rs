/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Pressure, SpecificEnthalpy, Temperature, Velocity3, VelocityGradient, ViscousStress,
    area_mach_ratio_kernel, entropy_production_rate_kernel, isentropic_density_ratio_kernel,
    isentropic_pressure_ratio_kernel, isentropic_temperature_ratio_kernel,
    specific_enthalpy_kernel, speed_of_sound_ideal_gas_kernel, total_enthalpy_kernel,
    total_pressure_isentropic_kernel, total_temperature_isentropic_kernel,
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
    let tau = ViscousStress::<f64>::new([
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
    let tau = ViscousStress::<f64>::default();
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
    let tau = ViscousStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(0.0).unwrap();
    let r = entropy_production_rate_kernel(&t, &tau, &grad_u, 1.0_f64, &[0.0; 3]);
    assert!(r.is_err());
}

#[test]
fn test_entropy_production_errors_on_negative_thermal_conductivity() {
    // κ < 0 would make σ < 0 (via the +κ‖∇T‖²/T² term) and break the
    // non-negativity contract advertised in the docstring.
    let tau = ViscousStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(300.0).unwrap();
    let r = entropy_production_rate_kernel(&t, &tau, &grad_u, -1.0_f64, &[1.0, 0.0, 0.0]);
    assert!(r.is_err());
}

#[test]
fn test_entropy_production_accepts_zero_thermal_conductivity() {
    // κ = 0 is the adiabatic limit and is physically realisable.
    let tau = ViscousStress::<f64>::default();
    let grad_u = VelocityGradient::<f64>::default();
    let t = Temperature::<f64>::new(300.0).unwrap();
    let sigma =
        entropy_production_rate_kernel(&t, &tau, &grad_u, 0.0_f64, &[1.0, 0.0, 0.0]).unwrap();
    assert_eq!(sigma, 0.0);
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

// NOTE on compressible.rs:88-90 — the "base of exponent must be positive" guard
// in `total_pressure_isentropic_kernel`. By the time this guard runs the kernel
// has already required `gamma > 1` (line 79). The base is
// `1 + (gamma - 1)/2 · mach²`; with `gamma > 1` the coefficient `(gamma-1)/2`
// is positive and `mach²` is non-negative for any real `mach`, so `base >= 1`
// always. `base <= 0` is therefore unreachable for any real-valued input.

// =============================================================================
// isentropic ratio kernels (Anderson, Modern Compressible Flow, Ch. 3)
// =============================================================================

#[test]
fn test_isentropic_ratios_at_rest_are_unity() {
    assert!((isentropic_pressure_ratio_kernel(0.0_f64, 1.4).unwrap() - 1.0).abs() < TOL);
    assert!((isentropic_temperature_ratio_kernel(0.0_f64, 1.4).unwrap() - 1.0).abs() < TOL);
    assert!((isentropic_density_ratio_kernel(0.0_f64, 1.4).unwrap() - 1.0).abs() < TOL);
}

#[test]
fn test_isentropic_temperature_ratio_known_values() {
    // T0/T = 1 + 0.2·M²: M = 1 → 1.2; M = 2 → 1.8 (γ = 1.4).
    assert!((isentropic_temperature_ratio_kernel(1.0_f64, 1.4).unwrap() - 1.2).abs() < TOL);
    assert!((isentropic_temperature_ratio_kernel(2.0_f64, 1.4).unwrap() - 1.8).abs() < TOL);
}

#[test]
fn test_isentropic_pressure_ratio_known_values() {
    // p0/p = (1 + 0.2·M²)^3.5: M = 1 → 1.2^3.5 ≈ 1.892929; M = 2 → 1.8^3.5 ≈ 7.824449.
    let p_m1 = isentropic_pressure_ratio_kernel(1.0_f64, 1.4).unwrap();
    assert!((p_m1 - 1.2_f64.powf(3.5)).abs() < TOL);
    let p_m2 = isentropic_pressure_ratio_kernel(2.0_f64, 1.4).unwrap();
    assert!((p_m2 - 1.8_f64.powf(3.5)).abs() < TOL);
    assert!((p_m2 - 7.824449).abs() < 1e-5);
}

#[test]
fn test_isentropic_density_ratio_known_value() {
    // ρ0/ρ = (1 + 0.2·M²)^2.5: M = 1 → 1.2^2.5 ≈ 1.577441.
    let d = isentropic_density_ratio_kernel(1.0_f64, 1.4).unwrap();
    assert!((d - 1.2_f64.powf(2.5)).abs() < TOL);
}

#[test]
fn test_isentropic_ratio_identity_p_equals_rho_times_t() {
    // Ideal gas: p0/p = (ρ0/ρ)·(T0/T) at every Mach.
    for &m in &[0.3_f64, 0.9, 1.0, 1.7, 3.2] {
        let p = isentropic_pressure_ratio_kernel(m, 1.4_f64).unwrap();
        let d = isentropic_density_ratio_kernel(m, 1.4_f64).unwrap();
        let t = isentropic_temperature_ratio_kernel(m, 1.4_f64).unwrap();
        assert!((p - d * t).abs() < 1e-9 * p);
    }
}

#[test]
fn test_isentropic_ratio_guards() {
    // NaN and negative Mach; γ ≤ 1 and NaN γ — all rejected on every kernel.
    assert!(isentropic_pressure_ratio_kernel(f64::NAN, 1.4).is_err());
    assert!(isentropic_pressure_ratio_kernel(-0.5_f64, 1.4).is_err());
    assert!(isentropic_pressure_ratio_kernel(1.0_f64, 1.0).is_err());
    assert!(isentropic_pressure_ratio_kernel(1.0_f64, f64::NAN).is_err());
    assert!(isentropic_temperature_ratio_kernel(f64::NAN, 1.4).is_err());
    assert!(isentropic_temperature_ratio_kernel(1.0_f64, 0.9).is_err());
    assert!(isentropic_density_ratio_kernel(f64::INFINITY, 1.4).is_err());
    assert!(isentropic_density_ratio_kernel(1.0_f64, f64::INFINITY).is_err());
}

#[test]
fn test_isentropic_ratio_purity() {
    // Pure free functions: identical inputs give bit-identical outputs.
    let a = isentropic_pressure_ratio_kernel(1.37_f64, 1.4).unwrap();
    let b = isentropic_pressure_ratio_kernel(1.37_f64, 1.4).unwrap();
    assert_eq!(a.to_bits(), b.to_bits());
}

// =============================================================================
// area_mach_ratio_kernel (Anderson, Modern Compressible Flow, Ch. 5)
// =============================================================================

#[test]
fn test_area_mach_ratio_is_unity_at_throat() {
    // A/A* = 1 exactly at M = 1.
    let r = area_mach_ratio_kernel(1.0_f64, 1.4).unwrap();
    assert!((r - 1.0).abs() < TOL);
}

#[test]
fn test_area_mach_ratio_known_values() {
    // γ = 1.4 closed-form checks: M = 0.5 → 1.339844; M = 2 → 1.6875 (exact: 27/16).
    let sub = area_mach_ratio_kernel(0.5_f64, 1.4).unwrap();
    assert!((sub - 1.339_844).abs() < 1e-5);
    let sup = area_mach_ratio_kernel(2.0_f64, 1.4).unwrap();
    assert!((sup - 1.6875).abs() < 1e-9);
}

#[test]
fn test_area_mach_ratio_monotone_on_each_branch() {
    // Strictly decreasing on the subsonic branch, strictly increasing on the
    // supersonic branch, with the minimum A/A* = 1 at the throat.
    let subsonic = [0.1_f64, 0.3, 0.5, 0.7, 0.9];
    for w in subsonic.windows(2) {
        let a = area_mach_ratio_kernel(w[0], 1.4_f64).unwrap();
        let b = area_mach_ratio_kernel(w[1], 1.4_f64).unwrap();
        assert!(a > b, "subsonic branch must decrease toward the throat");
        assert!(b > 1.0);
    }
    let supersonic = [1.1_f64, 1.5, 2.0, 3.0, 4.0];
    for w in supersonic.windows(2) {
        let a = area_mach_ratio_kernel(w[0], 1.4_f64).unwrap();
        let b = area_mach_ratio_kernel(w[1], 1.4_f64).unwrap();
        assert!(
            b > a,
            "supersonic branch must increase away from the throat"
        );
        assert!(a > 1.0);
    }
}

#[test]
fn test_area_mach_ratio_guards() {
    // M = 0 diverges (division by M); NaN/negative M and γ ≤ 1 are rejected.
    assert!(area_mach_ratio_kernel(0.0_f64, 1.4).is_err());
    assert!(area_mach_ratio_kernel(-1.0_f64, 1.4).is_err());
    assert!(area_mach_ratio_kernel(f64::NAN, 1.4).is_err());
    assert!(area_mach_ratio_kernel(2.0_f64, 1.0).is_err());
    assert!(area_mach_ratio_kernel(2.0_f64, f64::NAN).is_err());
}

#[test]
fn test_area_mach_ratio_purity() {
    let a = area_mach_ratio_kernel(2.5_f64, 1.4).unwrap();
    let b = area_mach_ratio_kernel(2.5_f64, 1.4).unwrap();
    assert_eq!(a.to_bits(), b.to_bits());
}
