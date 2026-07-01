/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Encke↔Cowell integrator regime switch (B4): `ε = a_aero/a_grav` is the g-load computed from
//! state; the Schmitt-trigger switch selects the integrator with hysteresis (no chatter near `ε_switch`);
//! and in the overlap band the perturbed-conformal (KS Strang) and direct (RK4) integrators agree.

use deep_causality_cfd::{IntegratorRegime, RegimeSwitch, aero_gravity_ratio};
use deep_causality_physics::{EARTH_GM, KsPropagator, ks_strang_step};

#[test]
fn epsilon_is_the_g_load_from_state() {
    // ε = |a_aero| / (GM/r²). At r = 7e6 m, a_grav = GM/r² ≈ 8.13 m/s²; a 0.813 m/s² drag ⇒ ε ≈ 0.1.
    let r = 7.0e6;
    let a_grav = EARTH_GM / (r * r);
    let eps = aero_gravity_ratio([a_grav * 0.1, 0.0, 0.0], r, EARTH_GM).unwrap();
    assert!(
        (eps - 0.1).abs() < 1e-12,
        "ε should be the g-load ratio: {eps}"
    );
    // Non-positive radius is rejected.
    assert!(aero_gravity_ratio([1.0, 0.0, 0.0], 0.0, EARTH_GM).is_err());
}

#[test]
fn switch_selects_conformal_below_and_direct_above() {
    let mut sw = RegimeSwitch::new(0.1, 0.5, IntegratorRegime::PerturbedConformal);
    assert_eq!(sw.select(0.05), IntegratorRegime::PerturbedConformal);
    assert_eq!(sw.select(0.4), IntegratorRegime::PerturbedConformal); // still below the enter threshold
    assert_eq!(sw.select(0.6), IntegratorRegime::Direct); // crosses enter_direct
    assert_eq!(sw.regime(), IntegratorRegime::Direct);
}

#[test]
fn hysteresis_prevents_chatter() {
    let mut sw = RegimeSwitch::new(0.1, 0.5, IntegratorRegime::PerturbedConformal);
    // Oscillate inside the dead band [0.1, 0.5], cross up, oscillate again, cross down.
    let seq = [0.2, 0.4, 0.2, 0.4, 0.6, 0.4, 0.2, 0.3, 0.05, 0.2];
    let mut prev = sw.regime();
    let mut transitions = 0;
    let mut states = Vec::new();
    for &e in &seq {
        let r = sw.select(e);
        states.push(r);
        if r != prev {
            transitions += 1;
            prev = r;
        }
    }
    // In-band oscillation before the up-cross keeps Conformal; after the up-cross keeps Direct.
    assert_eq!(states[0..4], [IntegratorRegime::PerturbedConformal; 4]);
    assert_eq!(states[5..8], [IntegratorRegime::Direct; 3]);
    assert_eq!(
        transitions, 2,
        "exactly two regime changes despite in-band oscillation (no chatter)"
    );
}

/// RK4 of the perturbed two-body EOM `ẍ = −μ x/r³ − k v` (the direct / Cowell integrator).
fn rk4_direct(r0: [f64; 3], v0: [f64; 3], k: f64, horizon: f64, n: usize) -> [f64; 3] {
    let h = horizon / n as f64;
    let deriv = |r: [f64; 3], v: [f64; 3]| {
        let rr = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
        let c = -EARTH_GM / (rr * rr * rr);
        (
            v,
            [
                c * r[0] - k * v[0],
                c * r[1] - k * v[1],
                c * r[2] - k * v[2],
            ],
        )
    };
    let add =
        |a: [f64; 3], b: [f64; 3], s: f64| [a[0] + b[0] * s, a[1] + b[1] * s, a[2] + b[2] * s];
    let (mut r, mut v) = (r0, v0);
    for _ in 0..n {
        let (k1r, k1v) = deriv(r, v);
        let (k2r, k2v) = deriv(add(r, k1r, h / 2.0), add(v, k1v, h / 2.0));
        let (k3r, k3v) = deriv(add(r, k2r, h / 2.0), add(v, k2v, h / 2.0));
        let (k4r, k4v) = deriv(add(r, k3r, h), add(v, k3v, h));
        for i in 0..3 {
            r[i] += h / 6.0 * (k1r[i] + 2.0 * k2r[i] + 2.0 * k3r[i] + k4r[i]);
            v[i] += h / 6.0 * (k1v[i] + 2.0 * k2v[i] + 2.0 * k3v[i] + k4v[i]);
        }
    }
    r
}

#[test]
fn overlap_band_conformal_and_direct_agree() {
    // A small-ε aero (drag a = −k·v): ε is well inside the perturbative band, and there the
    // perturbed-conformal (KS Strang) and the direct (RK4) integrator agree — the handover is seamless.
    let (r0, v0): ([f64; 3], [f64; 3]) = ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3]);
    let k = 1.0e-6;
    let radius = (r0[0] * r0[0] + r0[1] * r0[1] + r0[2] * r0[2]).sqrt();
    let a_aero = [-k * v0[0], -k * v0[1], -k * v0[2]];
    let eps = aero_gravity_ratio(a_aero, radius, EARTH_GM).unwrap();
    assert!(
        eps < 0.01,
        "the test aero is in the perturbative band: ε = {eps}"
    );

    let (horizon, n) = (300.0, 60usize);
    let h = horizon / n as f64;
    let accel = move |_r: [f64; 3], v: [f64; 3]| [-k * v[0], -k * v[1], -k * v[2]];
    let (mut r, mut v) = (r0, v0);
    for _ in 0..n {
        let (rn, vn) = ks_strang_step(r, v, EARTH_GM, h, accel).unwrap();
        r = rn;
        v = vn;
    }
    let r_direct = rk4_direct(r0, v0, k, horizon, n * 50);
    let diff = ((r[0] - r_direct[0]).powi(2)
        + (r[1] - r_direct[1]).powi(2)
        + (r[2] - r_direct[2]).powi(2))
    .sqrt();
    // Both integrate the same physics; the Strang split agrees with the direct solve in the band.
    assert!(
        diff < 1.0,
        "conformal and direct agree in the overlap band: {diff} m"
    );
    // (sanity: KsPropagator builds from the state — a valid bound orbit throughout.)
    assert!(KsPropagator::from_state(r, v, EARTH_GM).is_ok());
}
