/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cohesive `ReentryNavEngine` (B1 KS nominal + B2 error-state filter + B3 carried clock): coasting
//! follows the exact Kepler orbit and stays on the KS constraint manifold; the carried relativistic clock
//! accumulates a `τ`-offset distinct from coordinate time (the two-clock separation); and a returning
//! position fix reacquires while the trajectory stays a valid bound orbit.

use deep_causality_cfd::{InsErrorState, NavFilter, ReentryNavEngine};
use deep_causality_physics::{EARTH_GM, KsPropagator};

fn state_3d() -> ([f64; 3], [f64; 3]) {
    ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3])
}

fn engine() -> ReentryNavEngine<f64> {
    let (r0, v0) = state_3d();
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]);
    ReentryNavEngine::new(r0, v0, EARTH_GM, filter)
}

fn n3(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

#[test]
fn coast_follows_kepler_and_stays_on_manifold() {
    // Zero aero, no correction ⇒ the nominal is the exact KS drift; N steps of dt compose (semigroup)
    // to propagate(N·dt) to round-off, and the state stays a bound orbit (the B2 constraint manifold).
    let (r0, v0) = state_3d();
    let reference = KsPropagator::from_state(r0, v0, EARTH_GM).unwrap();
    let mut eng = engine();
    let dt = 5.0;
    let steps = 40usize;
    let q = [0.0; 17];
    for _ in 0..steps {
        eng.predict(dt, [0.0; 3], q).unwrap();
        assert!(
            eng.is_on_orbit_manifold(),
            "stays on the KS constraint manifold"
        );
    }
    let (rp, _vp) = reference.propagate(dt * steps as f64).unwrap();
    assert!(
        n3(eng.position(), rp) < 1e-3,
        "coast follows the exact Kepler orbit: {}",
        n3(eng.position(), rp)
    );
}

#[test]
fn carried_clock_offset_is_relativistic_and_distinct_from_coordinate_time() {
    let mut eng = engine();
    let dt = 1.0;
    for _ in 0..300 {
        eng.predict(dt, [0.0; 3], [0.0; 17]).unwrap();
    }
    let tau = eng.carried_clock_offset();
    let t = eng.elapsed_time();
    assert!((t - 300.0).abs() < 1e-9, "coordinate time accumulates: {t}");
    // The relativistic offset is non-zero but ~1e-9·t in magnitude — a different clock, not `t`.
    assert!(tau.abs() > 0.0, "a relativistic offset accumulates");
    assert!(
        tau.abs() < 1e-4 && tau.abs() < 1e-6 * t.max(1.0) * 1.0e3,
        "τ-offset is the tiny relativistic correction, not coordinate time: τ={tau}, t={t}"
    );
}

#[test]
fn position_fix_reacquires_and_stays_on_orbit() {
    let mut eng = engine();
    let dt = 1.0;
    let q = [1.0e-2; 17]; // inflate uncertainty during the coast
    for _ in 0..120 {
        eng.predict(dt, [0.0; 3], q).unwrap();
    }
    let var_before = eng.position_variance();
    assert!(
        var_before > 1.0,
        "uncertainty grew in the coast: {var_before}"
    );

    // A returning fix 10 m off the nominal x. With the large accumulated variance the gain is near 1,
    // so the nominal snaps toward the fix and the variance collapses.
    let p = eng.position();
    let measured = [p[0] + 10.0, p[1], p[2]];
    eng.correct_position(measured, 0.01);

    assert!(
        (eng.position()[0] - measured[0]).abs() < 1.0,
        "the fix reacquires the nominal: {} vs {}",
        eng.position()[0],
        measured[0]
    );
    assert!(
        eng.position_variance() < var_before * 0.5,
        "the fix collapses the uncertainty: {var_before} -> {}",
        eng.position_variance()
    );
    assert!(
        eng.is_on_orbit_manifold(),
        "the corrected state is still a valid bound orbit (B2 projection holds)"
    );
}
