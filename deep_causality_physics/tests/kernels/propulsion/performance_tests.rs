/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Force, Mass, propellant_mass_flow_kernel, tsiolkovsky_delta_v_kernel,
};

const G0: f64 = 9.80665;

#[test]
fn test_mass_flow_definition_identity() {
    // Exact arithmetic identity: T = 1000 N, Isp = 100 s -> mdot = 1000/(100*g0).
    let mdot = propellant_mass_flow_kernel(Force::new(1000.0).unwrap(), 100.0).unwrap();
    assert!((mdot.value() - 1000.0 / (100.0 * G0)).abs() < 1e-12);
}

#[test]
fn test_mass_flow_published_engine_band() {
    // Published sea-level figures for a Merlin-1D-class engine: T = 845 kN at
    // Isp = 282 s -> mdot = 305.55 kg/s (SpaceX published thrust/Isp pair).
    let mdot = propellant_mass_flow_kernel(Force::new(845_000.0_f64).unwrap(), 282.0).unwrap();
    assert!((mdot.value() - 305.553).abs() < 0.01);
}

#[test]
fn test_mass_flow_zero_thrust_is_zero() {
    let mdot = propellant_mass_flow_kernel(Force::new(0.0_f64).unwrap(), 300.0).unwrap();
    assert_eq!(mdot.value(), 0.0);
}

#[test]
fn test_mass_flow_rejects_nonpositive_isp() {
    assert!(propellant_mass_flow_kernel(Force::new(1000.0_f64).unwrap(), 0.0).is_err());
    assert!(propellant_mass_flow_kernel(Force::new(1000.0).unwrap(), -10.0).is_err());
    assert!(propellant_mass_flow_kernel(Force::new(1000.0).unwrap(), f64::NAN).is_err());
}

#[test]
fn test_mass_flow_rejects_negative_thrust() {
    assert!(propellant_mass_flow_kernel(Force::new(-1.0_f64).unwrap(), 300.0).is_err());
}

#[test]
fn test_tsiolkovsky_unit_log_ratio() {
    // m0/m1 = e makes ln(m0/m1) = 1 exactly, so dv = Isp * g0: 300 s -> 2941.995 m/s.
    let m1 = 1000.0;
    let m0 = m1 * core::f64::consts::E;
    let dv =
        tsiolkovsky_delta_v_kernel(300.0, Mass::new(m0).unwrap(), Mass::new(m1).unwrap()).unwrap();
    assert!((dv.value() - 300.0 * G0).abs() < 1e-9);
}

#[test]
fn test_tsiolkovsky_equal_masses_zero_dv() {
    let dv = tsiolkovsky_delta_v_kernel(
        300.0_f64,
        Mass::new(500.0).unwrap(),
        Mass::new(500.0).unwrap(),
    )
    .unwrap();
    assert!(dv.value().abs() < 1e-12);
}

#[test]
fn test_tsiolkovsky_rejects_burn_ending_heavier() {
    assert!(
        tsiolkovsky_delta_v_kernel(
            300.0_f64,
            Mass::new(400.0).unwrap(),
            Mass::new(500.0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn test_tsiolkovsky_rejects_zero_final_mass() {
    assert!(
        tsiolkovsky_delta_v_kernel(
            300.0_f64,
            Mass::new(500.0).unwrap(),
            Mass::new(0.0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn test_tsiolkovsky_rejects_nonpositive_isp() {
    assert!(
        tsiolkovsky_delta_v_kernel(
            0.0_f64,
            Mass::new(500.0).unwrap(),
            Mass::new(400.0).unwrap()
        )
        .is_err()
    );
}
