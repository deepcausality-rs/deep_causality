/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    PARK_NO_IONIZATION_ACTIVATION_TEMP, PARK_NO_IONIZATION_EXPONENT, PARK_NO_IONIZATION_PREFACTOR,
    THETA_VIB_N2, Temperature, VibrationalTemperature, arrhenius_rate_kernel,
    vibrational_relaxation_kernel,
};

// ── Arrhenius rate kernel ────────────────────────────────────────────────

#[test]
fn test_arrhenius_matches_closed_form() {
    // k(T) = Cf · T^η · exp(−θd / T) with the Gupta RP-1232 N+O→NO⁺+e⁻ coeffs.
    let t = Temperature::<f64>::new(7000.0).unwrap();
    let k = arrhenius_rate_kernel(
        t,
        PARK_NO_IONIZATION_PREFACTOR,
        PARK_NO_IONIZATION_EXPONENT,
        PARK_NO_IONIZATION_ACTIVATION_TEMP,
    )
    .unwrap();
    let expected = PARK_NO_IONIZATION_PREFACTOR
        * 7000.0_f64.powf(PARK_NO_IONIZATION_EXPONENT)
        * (-PARK_NO_IONIZATION_ACTIVATION_TEMP / 7000.0).exp();
    assert!((k.value() - expected).abs() / expected < 1e-12);
    assert!(k.value() > 0.0);
}

#[test]
fn test_arrhenius_monotonic_in_temperature() {
    // For θd > 0 the forward rate increases with temperature.
    let lo = arrhenius_rate_kernel(
        Temperature::<f64>::new(5000.0).unwrap(),
        PARK_NO_IONIZATION_PREFACTOR,
        PARK_NO_IONIZATION_EXPONENT,
        PARK_NO_IONIZATION_ACTIVATION_TEMP,
    )
    .unwrap();
    let hi = arrhenius_rate_kernel(
        Temperature::<f64>::new(9000.0).unwrap(),
        PARK_NO_IONIZATION_PREFACTOR,
        PARK_NO_IONIZATION_EXPONENT,
        PARK_NO_IONIZATION_ACTIVATION_TEMP,
    )
    .unwrap();
    assert!(hi.value() > lo.value());
}

#[test]
fn test_arrhenius_rejects_nonpositive_temperature() {
    // Temperature::new already rejects negatives; zero is rejected by the kernel.
    let zero = Temperature::<f64>::new(0.0).unwrap();
    assert!(arrhenius_rate_kernel(zero, 1.0, 0.0, 100.0).is_err());
}

#[test]
fn test_arrhenius_rejects_negative_prefactor() {
    let t = Temperature::<f64>::new(7000.0).unwrap();
    assert!(arrhenius_rate_kernel(t, -1.0, 0.0, 100.0).is_err());
}

// ── Vibrational relaxation (LER) kernel ──────────────────────────────────

#[test]
fn test_vibrational_relaxation_zero_dt_is_identity() {
    let t_ve = VibrationalTemperature::<f64>::new(300.0).unwrap();
    let t_tr = Temperature::<f64>::new(7000.0).unwrap();
    let out = vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 14.0, THETA_VIB_N2, 0.0).unwrap();
    assert!((out.value() - 300.0).abs() < 1e-9);
}

#[test]
fn test_vibrational_relaxation_large_dt_reaches_target() {
    // dt ≫ τ ⇒ T_ve → T_tr (the stiff / equilibrium limit), bounded, no overshoot.
    let t_ve = VibrationalTemperature::<f64>::new(300.0).unwrap();
    let t_tr = Temperature::<f64>::new(7000.0).unwrap();
    let out = vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 14.0, THETA_VIB_N2, 1.0).unwrap();
    assert!((out.value() - 7000.0).abs() < 1.0);
    assert!(out.value() <= 7000.0); // never overshoots the target
}

#[test]
fn test_vibrational_relaxation_is_bounded_and_monotone() {
    // For any dt the result stays within [T_ve, T_tr] — the LER stability property.
    let t_ve = VibrationalTemperature::<f64>::new(300.0).unwrap();
    let t_tr = Temperature::<f64>::new(7000.0).unwrap();
    for &dt in &[1e-9, 1e-7, 1e-6, 1e-5] {
        let out = vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 14.0, THETA_VIB_N2, dt).unwrap();
        assert!(out.value() >= 300.0 - 1e-9 && out.value() <= 7000.0 + 1e-9);
    }
}

#[test]
fn test_vibrational_relaxation_rejects_bad_inputs() {
    let t_ve = VibrationalTemperature::<f64>::new(300.0).unwrap();
    let t_tr = Temperature::<f64>::new(7000.0).unwrap();
    assert!(vibrational_relaxation_kernel(t_ve, t_tr, 0.0, 14.0, THETA_VIB_N2, 1e-6).is_err());
    assert!(vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 0.0, THETA_VIB_N2, 1e-6).is_err());
    assert!(vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 14.0, 0.0, 1e-6).is_err());
    assert!(vibrational_relaxation_kernel(t_ve, t_tr, 1.0, 14.0, THETA_VIB_N2, -1.0).is_err());
}
