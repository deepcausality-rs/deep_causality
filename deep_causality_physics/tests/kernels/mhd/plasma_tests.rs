/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    ElectronDensity, Mass, PhysicalField, Speed, Temperature, debye_length_kernel,
    larmor_radius_kernel, plasma_frequency_kernel,
};

#[test]
fn test_plasma_frequency_matches_reference() {
    // ω_p = √(n_e e²/(ε₀ m_e)). For n_e = 1e18 m⁻³, ω_p ≈ 5.64e10 rad/s
    // (f_p ≈ 8.98·√n_e ≈ 8.98e9 Hz).
    let n_e = ElectronDensity::<f64>::new(1.0e18).unwrap();
    let w = plasma_frequency_kernel(n_e).unwrap();
    assert!((w.value() - 5.64e10).abs() / 5.64e10 < 0.01);
}

#[test]
fn test_plasma_frequency_zero_density_errors() {
    // PlasmaFrequency requires a strictly positive value, so n_e = 0 (no plasma)
    // surfaces as an error; the blackout trigger treats that as link-available.
    let n_e = ElectronDensity::<f64>::new(0.0).unwrap();
    assert!(plasma_frequency_kernel(n_e).is_err());
}

#[test]
fn test_plasma_frequency_monotonic() {
    let lo = plasma_frequency_kernel(ElectronDensity::<f64>::new(1.0e17).unwrap()).unwrap();
    let hi = plasma_frequency_kernel(ElectronDensity::<f64>::new(1.0e19).unwrap()).unwrap();
    assert!(hi.value() > lo.value());
}

#[test]
fn test_debye_length() {
    let t = Temperature::new(100.0).unwrap();
    let n = 1e18;
    let eps0 = 8.854e-12;
    let e = 1.602e-19;

    let res = debye_length_kernel(t, n, eps0, e);
    assert!(res.is_ok());
    assert!(res.unwrap().value() > 0.0);
}

#[test]
fn test_debye_length_zero_density_error() {
    // density_n <= 0 -> Singularity (plasma.rs:31-33).
    let t = Temperature::new(100.0).unwrap();
    assert!(debye_length_kernel(t, 0.0, 8.854e-12, 1.602e-19).is_err());
    let t2 = Temperature::new(100.0).unwrap();
    assert!(debye_length_kernel(t2, -1.0, 8.854e-12, 1.602e-19).is_err());
}

#[test]
fn test_debye_length_non_positive_permittivity_error() {
    // epsilon_0 <= 0 -> PhysicalInvariantBroken (plasma.rs:34-38).
    let t = Temperature::new(100.0).unwrap();
    assert!(debye_length_kernel(t, 1e18, 0.0, 1.602e-19).is_err());
    let t2 = Temperature::new(100.0).unwrap();
    assert!(debye_length_kernel(t2, 1e18, -1.0, 1.602e-19).is_err());
}

// NOTE on plasma.rs:41-42 — the `ok_or_else` closure body for
// `R::from_f64(BOLTZMANN_CONSTANT)`. `from_f64` is infallible for every
// concrete `RealField` used by this crate (f32/f64 always return `Some`), so
// the closure can never run. It is a defensive guard with no reachable input
// and is therefore left uncovered by design.

#[test]
fn test_larmor_radius() {
    let m = Mass::new(1.0).unwrap();
    let v = Speed::new(10.0).unwrap();
    let q = 1.0;

    let b_vec = CausalMultiVector::new(vec![0.0, 5.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b = PhysicalField::<f64>::new(b_vec);

    let res = larmor_radius_kernel(m, v, q, &b);
    assert!(res.is_ok());
    // r = mv/qB = 1*10 / 1*5 = 2
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}
