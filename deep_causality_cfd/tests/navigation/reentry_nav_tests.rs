/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cohesive `ReentryNavEngine` (B1 KS nominal + B2 error-state filter + B3 carried clock): coasting
//! follows the exact Kepler orbit and stays on the KS constraint manifold; the carried relativistic clock
//! accumulates a `τ`-offset distinct from coordinate time (the two-clock separation); and a returning
//! position fix reacquires while the trajectory stays a valid bound orbit.

use deep_causality_cfd::{InsErrorState, NavFilter, Quaternion, ReentryNavEngine};
use deep_causality_physics::{EARTH_GM, KsPropagator};

fn quat_norm(q: Quaternion<f64>) -> f64 {
    (q.w * q.w + q.x * q.x + q.y * q.y + q.z * q.z).sqrt()
}

fn vec_norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn state_3d() -> ([f64; 3], [f64; 3]) {
    ([7.0e6, 1.0e6, 2.0e6], [-1.0e3, 6.5e3, 3.0e3])
}

fn engine() -> ReentryNavEngine<f64> {
    let (r0, v0) = state_3d();
    let filter = NavFilter::new(InsErrorState::<f64>::zero(), [1.0; 17]).unwrap();
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
        eng.predict(dt, [0.0; 3], [0.0; 3], q).unwrap();
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
        eng.predict(dt, [0.0; 3], [0.0; 3], [0.0; 17]).unwrap();
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
fn velocity_and_filter_accessors_track_the_engine_state() {
    // `velocity()` returns the nominal velocity (changed by the KS drift), `filter()` exposes the
    // error-state filter whose position variance matches the engine's witness.
    let (_r0, v0) = state_3d();
    let mut eng = engine();
    assert_eq!(eng.velocity(), v0, "initial velocity mirrors the seed");
    // The filter accessor is consistent with the engine's variance witness.
    assert!(
        (eng.filter().position_variance() - eng.position_variance()).abs() < 1e-12,
        "filter() variance matches the engine witness"
    );

    let dt = 2.0;
    for _ in 0..10 {
        eng.predict(dt, [0.0; 3], [0.0; 3], [0.0; 17]).unwrap();
    }
    // Coasting on the KS orbit changes the velocity away from the seed.
    assert!(
        n3(eng.velocity(), v0) > 0.0,
        "the KS drift advanced the nominal velocity"
    );
    // After coasting, the two variance readings still agree (same underlying filter).
    assert!(
        (eng.filter().position_variance() - eng.position_variance()).abs() < 1e-12,
        "filter() stays consistent after predict"
    );
}

#[test]
fn position_fix_reacquires_and_stays_on_orbit() {
    let mut eng = engine();
    let dt = 1.0;
    let q = [1.0e-2; 17]; // inflate uncertainty during the coast
    for _ in 0..120 {
        eng.predict(dt, [0.0; 3], [0.0; 3], q).unwrap();
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
    eng.correct_position(measured, 0.01).unwrap();

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

// ── The attitude-error lifecycle (design D4, option a): estimate → inject → reset ─────────────────

#[test]
fn a_fix_injects_the_attitude_error_into_the_nominal_then_resets_it() {
    // Seed a gyro bias so the coast grows a real attitude error while the nominal (sensed rate ω̂ = 0)
    // stays at identity. A fix must inject that error into the nominal — the nominal rotates away from
    // identity — and only then zero the attitude block. This is the invariant the spec states: the
    // reset is legitimate because the estimate was transferred, not discarded.
    let (r0, v0) = state_3d();
    let seeded = InsErrorState::<f64>::from_biases([0.0; 3], [2.0e-3, -1.0e-3, 5.0e-4]);
    let filter = NavFilter::new(seeded, [1.0; 17]).unwrap();
    let mut eng = ReentryNavEngine::new(r0, v0, EARTH_GM, filter);
    let dt = 1.0;
    for _ in 0..50 {
        eng.predict(dt, [0.0; 3], [0.0; 3], [1.0e-6; 17]).unwrap();
    }
    let dpsi = eng.filter().state().attitude_error();
    assert!(
        vec_norm(dpsi) > 1.0e-3,
        "the coast grew a real attitude error: |δψ| = {}",
        vec_norm(dpsi)
    );
    assert_eq!(
        eng.attitude(),
        Quaternion::<f64>::identity(),
        "the nominal stayed at identity through the coast (ω̂ = 0)"
    );

    let p = eng.position();
    eng.correct_position([p[0] + 5.0, p[1], p[2]], 1.0).unwrap();

    assert_ne!(
        eng.attitude(),
        Quaternion::<f64>::identity(),
        "the fix injected the attitude error into the nominal"
    );
    assert_eq!(
        eng.filter().state().attitude_error(),
        [0.0; 3],
        "the attitude error was reset only after it was injected"
    );
}

#[test]
fn repeated_fixes_keep_correcting_the_nominal_rather_than_claiming_free_confidence() {
    // With a standing gyro bias, every predict grows an attitude error and every fix injects it into the
    // nominal before resetting. Over many cycles the nominal keeps being corrected (it rotates steadily
    // away from identity), so the covariance reductions the fixes take are matched by applied
    // corrections — not claimed for free, and the error never accumulates unbounded.
    let (r0, v0) = state_3d();
    let seeded = InsErrorState::<f64>::from_biases([0.0; 3], [1.0e-3, 0.0, 0.0]);
    let filter = NavFilter::new(seeded, [1.0; 17]).unwrap();
    let mut eng = ReentryNavEngine::new(r0, v0, EARTH_GM, filter);
    let dt = 1.0;
    let mut max_reset_error = 0.0f64;
    for _ in 0..100 {
        eng.predict(dt, [0.0; 3], [0.0; 3], [1.0e-6; 17]).unwrap();
        let p = eng.position();
        eng.correct_position([p[0], p[1], p[2]], 1.0).unwrap();
        // The attitude error is reset (bounded, not accumulating) after each injected fix.
        max_reset_error = max_reset_error.max(vec_norm(eng.filter().state().attitude_error()));
    }
    assert!(
        max_reset_error < 1.0e-9,
        "the attitude error is reset after each fix, never accumulating: max {max_reset_error}"
    );
    // The nominal has been steadily corrected — the injected corrections are real, not free confidence.
    assert!(
        quat_norm(eng.attitude()) > 0.0 && eng.attitude() != Quaternion::<f64>::identity(),
        "the nominal attitude was corrected by the injected fixes"
    );
    assert!(
        (quat_norm(eng.attitude()) - 1.0).abs() < 1.0e-12,
        "the corrected nominal stays a unit quaternion"
    );
}

#[test]
fn a_zero_rate_leaves_the_nominal_exactly_at_identity() {
    // No sensed rotation ⇒ the nominal attitude is exactly identity across the whole march, so C(q) = I
    // and the numbers reduce to the Tier-A model (this is why the point-mass examples are unchanged).
    let mut eng = engine();
    let dt = 0.5;
    for _ in 0..1000 {
        eng.predict(dt, [1.0, -2.0, 0.5], [0.0; 3], [1.0e-6; 17])
            .unwrap();
    }
    assert_eq!(
        eng.attitude(),
        Quaternion::<f64>::identity(),
        "a zero gyro input leaves the nominal exactly unchanged"
    );
}

#[test]
fn a_nonzero_rate_integrates_and_stays_normalised_across_a_long_march() {
    // A steady body rate rotates the nominal; over a long march the quaternion must stay unit — the one
    // integration-drift hazard option (a) introduces, guarded by the per-step normalise.
    let mut eng = engine();
    let dt = 0.01;
    let omega = [0.3, -0.1, 0.2];
    for _ in 0..5000 {
        eng.predict(dt, [0.0; 3], omega, [1.0e-9; 17]).unwrap();
    }
    assert_ne!(
        eng.attitude(),
        Quaternion::<f64>::identity(),
        "a nonzero sensed rate rotated the nominal"
    );
    assert!(
        (quat_norm(eng.attitude()) - 1.0).abs() < 1.0e-9,
        "the quaternion stays normalised across a long rotating march: {}",
        quat_norm(eng.attitude())
    );
}
