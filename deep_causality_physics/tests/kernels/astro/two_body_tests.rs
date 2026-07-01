/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact two-body propagator (`TwoBodyPropagator`) — the constant-generator matrix-exponential Kepler core
//! validated in the Gap-3 FS-1 study. Tests cover exactness (round-off), conservation, and the rejections.

use deep_causality_physics::{EARTH_GM, TwoBodyPropagator};

fn leo() -> TwoBodyPropagator<f64> {
    // A bound, eccentric LEO-ish orbit (the FS-1 reference state).
    TwoBodyPropagator::from_state([7.0e6, 0.0], [1.0e3, 7.5e3], EARTH_GM).unwrap()
}

fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

#[test]
fn elements_are_physical() {
    let orbit = leo();
    assert!(orbit.eccentricity() > 0.0 && orbit.eccentricity() < 1.0);
    assert!(orbit.semi_major_axis() > 7.0e6);
    assert!(orbit.mean_motion() > 0.0);
    assert!((orbit.gravitational_parameter() - EARTH_GM).abs() < 1.0);
    assert!(orbit.period().unwrap() > 0.0);
}

#[test]
fn epoch_state_is_recovered_at_dt_zero() {
    let orbit = leo();
    let (pos, vel) = orbit.propagate(0.0).unwrap();
    assert!(
        dist(pos, [7.0e6, 0.0]) < 1e-6,
        "position at dt=0 must be the epoch state"
    );
    assert!(
        dist(vel, [1.0e3, 7.5e3]) < 1e-9,
        "velocity at dt=0 must be the epoch state"
    );
}

#[test]
fn one_period_returns_to_start_to_round_off() {
    // The matrix-exponential core is exactly periodic: after one period the state returns to the epoch.
    let orbit = leo();
    let t = orbit.period().unwrap();
    let (pos, vel) = orbit.propagate(t).unwrap();
    let a = orbit.semi_major_axis();
    assert!(
        dist(pos, [7.0e6, 0.0]) / a < 1e-12,
        "one-period closure (position)"
    );
    assert!(
        dist(vel, [1.0e3, 7.5e3]) < 1e-6,
        "one-period closure (velocity)"
    );
}

#[test]
fn forward_then_backward_is_identity() {
    let orbit = leo();
    let dt = 1234.5;
    let (p1, _) = orbit.propagate(dt).unwrap();
    // Build a fresh orbit from the propagated state and step back; must recover the epoch.
    let (pf, vf) = orbit.propagate(dt).unwrap();
    let back = TwoBodyPropagator::from_state(pf, vf, EARTH_GM).unwrap();
    let (p0, _) = back.propagate(-dt).unwrap();
    assert!(dist(p1, pf) < 1e-9);
    assert!(
        dist(p0, [7.0e6, 0.0]) / orbit.semi_major_axis() < 1e-10,
        "round-trip identity"
    );
}

#[test]
fn energy_and_angular_momentum_conserved() {
    let orbit = leo();
    let e0 = {
        let r = 7.0e6;
        let v2 = 1.0e3 * 1.0e3 + 7.5e3 * 7.5e3;
        0.5 * v2 - EARTH_GM / r
    };
    let h0 = 7.0e6 * 7.5e3 - 0.0 * 1.0e3;
    for &dt in &[100.0, 1500.0, 3000.0, 5000.0] {
        let (p, v) = orbit.propagate(dt).unwrap();
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        let v2 = v[0] * v[0] + v[1] * v[1];
        let e = 0.5 * v2 - EARTH_GM / r;
        let h = p[0] * v[1] - p[1] * v[0];
        assert!(
            (e - e0).abs() / e0.abs() < 1e-12,
            "energy conserved at dt={dt}"
        );
        assert!(
            (h - h0).abs() / h0.abs() < 1e-12,
            "angular momentum conserved at dt={dt}"
        );
    }
}

#[test]
fn period_matches_keplers_third_law() {
    let orbit = leo();
    let a = orbit.semi_major_axis();
    let t_expected = std::f64::consts::TAU * (a * a * a / EARTH_GM).sqrt();
    assert!((orbit.period().unwrap() - t_expected).abs() / t_expected < 1e-14);
}

#[test]
fn rejects_non_positive_gm() {
    assert!(TwoBodyPropagator::from_state([7.0e6, 0.0], [0.0, 7.5e3], 0.0).is_err());
    assert!(TwoBodyPropagator::from_state([7.0e6, 0.0], [0.0, 7.5e3], -1.0).is_err());
}

#[test]
fn rejects_unbound_orbit() {
    // Escape-speed-plus state: energy ≥ 0 ⇒ not an ellipse.
    let v_esc = (2.0 * EARTH_GM / 7.0e6).sqrt();
    assert!(TwoBodyPropagator::from_state([7.0e6, 0.0], [0.0, v_esc * 1.2], EARTH_GM).is_err());
}

#[test]
fn rejects_zero_radius() {
    assert!(TwoBodyPropagator::from_state([0.0, 0.0], [0.0, 7.5e3], EARTH_GM).is_err());
}
