/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Acceleration, Length, Speed, ignition_altitude_kernel, stopping_distance_kernel,
    suicide_burn_deceleration_kernel,
};

const G0: f64 = 9.80665;

#[test]
fn test_stopping_distance_identity() {
    // v = 100 m/s against a_net = 5 m/s^2: d = v^2/(2a) = 1000 m exactly.
    let d = stopping_distance_kernel(
        Speed::new(100.0_f64).unwrap(),
        Acceleration::new(5.0).unwrap(),
    )
    .unwrap();
    assert!((d.value() - 1000.0).abs() < 1e-12);
}

#[test]
fn test_stopping_distance_rejects_nonpositive_deceleration() {
    assert!(
        stopping_distance_kernel(Speed::new(100.0).unwrap(), Acceleration::new(0.0).unwrap())
            .is_err()
    );
    assert!(
        stopping_distance_kernel(Speed::new(100.0).unwrap(), Acceleration::new(-1.0).unwrap())
            .is_err()
    );
}

#[test]
fn test_ignition_altitude_matches_stopping_distance_plus_margin() {
    // a_T = 3g gives a_net = 2g; v = 200 m/s: d = v^2/(4g); margin 50 m on top.
    let v = 200.0;
    let h = ignition_altitude_kernel(
        Speed::new(v).unwrap(),
        Acceleration::new(3.0 * G0).unwrap(),
        Acceleration::new(G0).unwrap(),
        Length::new(50.0).unwrap(),
    )
    .unwrap();
    assert!((h.value() - (v * v / (4.0 * G0) + 50.0)).abs() < 1e-9);
}

#[test]
fn test_ignition_altitude_rejects_thrust_to_weight_below_one() {
    // a_T = g exactly: the vehicle hovers, never stops. a_T < g: falls.
    for a_t in [G0, 0.5 * G0] {
        assert!(
            ignition_altitude_kernel(
                Speed::new(100.0).unwrap(),
                Acceleration::new(a_t).unwrap(),
                Acceleration::new(G0).unwrap(),
                Length::new(0.0).unwrap(),
            )
            .is_err()
        );
    }
}

#[test]
fn test_ignition_altitude_rejects_nonpositive_gravity() {
    assert!(
        ignition_altitude_kernel(
            Speed::new(100.0).unwrap(),
            Acceleration::new(30.0).unwrap(),
            Acceleration::new(0.0).unwrap(),
            Length::new(0.0).unwrap(),
        )
        .is_err()
    );
}

#[test]
fn test_suicide_burn_command_keeps_specific_deceleration_invariant() {
    // Integrating the commanded net deceleration (a_cmd - g = v^2/2h) from
    // v0 = 100 m/s, h0 = 600 m: the closed-form solution is v = v0*sqrt(h/h0)
    // (v^2/2h stays constant), so speed and altitude null together with the
    // touchdown speed scaling as sqrt(h). Both properties are asserted.
    let g = G0;
    let (v0, h0) = (100.0_f64, 600.0_f64);
    let k0 = v0 * v0 / (2.0 * h0);
    let (mut v, mut h) = (v0, h0);
    let dt = 1e-5;
    let mut max_invariant_drift = 0.0_f64;
    while h > 0.05 && v > 0.0 {
        let a_cmd = suicide_burn_deceleration_kernel(
            Speed::new(v).unwrap(),
            Length::new(h).unwrap(),
            Acceleration::new(g).unwrap(),
        )
        .unwrap()
        .value();
        v -= (a_cmd - g) * dt;
        h -= v * dt;
        if h > 0.0 {
            max_invariant_drift = max_invariant_drift.max((v * v / (2.0 * h) - k0).abs() / k0);
        }
    }
    assert!(h <= 0.05);
    // The invariant v^2/2h holds through the whole burn (integration error only).
    assert!(
        max_invariant_drift < 1e-2,
        "invariant drifted by {max_invariant_drift}"
    );
    // Residual speed at the 5 cm floor matches v0*sqrt(h/h0) = 0.9129 m/s.
    let expected = v0 * (h / h0).sqrt();
    assert!(
        (v - expected).abs() < 0.01,
        "residual speed {v} vs closed form {expected}"
    );
}

#[test]
fn test_suicide_burn_hover_limit_at_zero_speed() {
    // v = 0: the command is exactly gravity (hover), no divide-by-zero.
    let a = suicide_burn_deceleration_kernel(
        Speed::new(0.0).unwrap(),
        Length::new(100.0).unwrap(),
        Acceleration::new(G0).unwrap(),
    )
    .unwrap();
    assert!((a.value() - G0).abs() < 1e-12);
}

#[test]
fn test_suicide_burn_rejects_ground_contact_and_bad_gravity() {
    assert!(
        suicide_burn_deceleration_kernel(
            Speed::new(10.0).unwrap(),
            Length::new(0.0).unwrap(),
            Acceleration::new(G0).unwrap(),
        )
        .is_err()
    );
    assert!(
        suicide_burn_deceleration_kernel(
            Speed::new(10.0).unwrap(),
            Length::new(100.0).unwrap(),
            Acceleration::new(0.0).unwrap(),
        )
        .is_err()
    );
}
