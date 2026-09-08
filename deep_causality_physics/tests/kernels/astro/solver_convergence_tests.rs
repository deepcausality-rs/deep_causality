/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The three iterative solvers that reached their iteration cap and returned the last iterate in
//! silence: the electroweak W-mass fixed point, Kepler's equation, and the fictitious-time
//! inversion.
//!
//! # The oracle
//!
//! Kepler's equation `M = E − e·sin E` is monotone increasing in `E` on `[0, 2π)`, so a bisection
//! finds its root without sharing anything with the Newton iteration under test — no formula, no
//! derivative, no starting point. The expected **position** is then the definition
//! `(a(cos E − e), a√(1−e²)·sin E)` applied to that root, which is a conversion rather than a
//! solve. Nothing here takes an expected value from the solver it is checking.
//!
//! # The trap this pins
//!
//! At `e = 0.9999, M = 1e-6` the Newton iteration reaches its 100-iteration cap **with the root
//! already in hand**: the step test runs out because `1 − e·cos E` is near zero and the correction
//! is dominated by rounding, while the residual is about `1e-18`. Erroring at the cap would turn a
//! correct answer into a failure on a case that works today, which is why the acceptance is on the
//! residual and the refusal is reserved for an iterate that satisfies neither test.

use deep_causality_physics::{TwoBodyPropagator, solve_w_mass};

/// The root of `E − e·sin E = M` by bisection on `[0, 2π)`.
///
/// Independent of the solver under test: no derivative, no Newton step, no shared starting point.
/// 300 halvings take the bracket well below `f64` resolution.
fn kepler_root_by_bisection(m: f64, e: f64) -> f64 {
    let f = |x: f64| x - e * x.sin() - m;
    let (mut lo, mut hi) = (0.0f64, 2.0 * std::f64::consts::PI);
    for _ in 0..300 {
        let mid = 0.5 * (lo + hi);
        if f(lo) * f(mid) <= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    0.5 * (lo + hi)
}

/// An ellipse of semi-major axis `a` and eccentricity `e` about `gm = 1`, started at periapsis on
/// the `+x` axis so the perifocal frame and the state frame coincide.
fn periapsis_orbit(a: f64, e: f64) -> TwoBodyPropagator<f64> {
    let gm = 1.0f64;
    let rp = a * (1.0 - e);
    let vp = (gm * (1.0 + e) / rp).sqrt();
    TwoBodyPropagator::from_state([rp, 0.0], [0.0, vp], gm).unwrap()
}

// =============================================================================
// Kepler: the cap is reached with a correct iterate, and that is accepted
// =============================================================================

#[test]
fn test_kepler_at_high_eccentricity_returns_the_root_rather_than_an_error() {
    // `e = 0.9999, M = 1e-6` exhausts all 100 iterations. The naive repair — error at the cap —
    // would reject this, and the answer is right.
    let a = 1.0f64;
    let e = 0.9999f64;
    let orbit = periapsis_orbit(a, e);
    assert!(
        (orbit.eccentricity() - e).abs() < 1e-12,
        "fixture eccentricity"
    );

    let m = 1e-6f64;
    let (position, _) = orbit
        .propagate(m / orbit.mean_motion())
        .expect("a correct iterate at the cap must be accepted, not refused");

    // The independent root, and the position it defines.
    let root = kepler_root_by_bisection(m, e);
    let want_x = a * (root.cos() - e);
    let want_y = a * (1.0 - e * e).sqrt() * root.sin();

    assert!(
        (position[0] - want_x).abs() < 1e-12,
        "x: got {}, bisection gives {want_x}",
        position[0]
    );
    assert!(
        (position[1] - want_y).abs() < 1e-12,
        "y: got {}, bisection gives {want_y}",
        position[1]
    );
}

#[test]
fn test_kepler_agrees_with_the_bisection_across_eccentricities() {
    // The converged paths, unchanged. Each of these meets the step test well inside the cap.
    for (a, e, m) in [
        (1.0f64, 1e-6f64, 0.7f64),
        (1.0, 0.5, 1.0),
        (2.0, 0.9, 0.25),
        (1.0, 0.99, 0.01),
    ] {
        let orbit = periapsis_orbit(a, e);
        let (position, _) = orbit.propagate(m / orbit.mean_motion()).unwrap();
        let root = kepler_root_by_bisection(m, e);
        let want_x = a * (root.cos() - e);
        let want_y = a * (1.0 - e * e).sqrt() * root.sin();
        assert!(
            (position[0] - want_x).abs() < 1e-9 && (position[1] - want_y).abs() < 1e-9,
            "e={e}, M={m}: got {position:?}, bisection gives ({want_x}, {want_y})"
        );
    }
}

#[test]
fn test_a_near_circular_orbit_returns_to_its_start_after_one_period() {
    // An invariant rather than a value: after exactly one period the state repeats. Independent of
    // both the Newton solve and the bisection.
    //
    // `e = 1e-6` rather than `0`, because a *perfectly* circular orbit has no defined periapsis and
    // `from_state` produces a `NaN` mean anomaly for it — see
    // `test_a_perfectly_circular_orbit_is_refused_rather_than_returning_nan` below.
    let orbit = periapsis_orbit(1.0, 1e-6);
    let period = orbit.period().unwrap();
    let (p, _) = orbit.propagate(period).unwrap();
    assert!((p[0] - 1.0).abs() < 1e-5 && p[1].abs() < 1e-5, "got {p:?}");
}

#[test]
fn test_a_perfectly_circular_orbit_is_refused_rather_than_returning_nan() {
    // A pre-existing defect this stage **surfaced** rather than introduced, recorded here so it is
    // not rediscovered. At `e = 0` exactly the eccentricity vector vanishes, so the periapsis
    // direction — and with it the mean anomaly — is undefined; `from_state` reports the right
    // eccentricity and mean motion but a `NaN` mean anomaly. Newton's step is then `NaN` at every
    // iteration, and the old code returned that as a position without a word.
    //
    // Signalling non-convergence converts the silent `NaN` into a typed refusal. That is strictly
    // better and is still not a fix: giving a circular orbit a reference direction is a decision
    // about convention, and belongs to whoever makes it rather than to this stage.
    let orbit = periapsis_orbit(1.0, 0.0);
    assert_eq!(orbit.eccentricity(), 0.0, "the orbit itself is well formed");
    let err = orbit
        .propagate(0.7)
        .expect_err("a NaN mean anomaly must not come back as a position");
    assert!(format!("{err}").contains("did not converge"), "got {err}");
}

// =============================================================================
// The electroweak fixed point: reachable, and now signalled
// =============================================================================

/// The shipped constants, as `deep_causality_physics`'s own consumer supplies them.
const Z_MASS: f64 = 91.1876;
const TOP_MASS: f64 = 172.52;
const ALPHA_EM_MZ: f64 = 1.0 / 127.95;
const ALPHA_EM_0: f64 = 1.0 / 137.035999;
const FERMI_CONSTANT: f64 = 1.1663787e-5;

#[test]
fn test_the_w_mass_solver_converges_at_the_shipped_constants() {
    // The latency evidence: at the constants actually supplied, the fixed point converges well
    // inside its 20-iteration cap, so the silence this stage removes was a latent defect on a
    // public function rather than a live one.
    assert!(
        solve_w_mass(Z_MASS, TOP_MASS, ALPHA_EM_MZ, ALPHA_EM_0, FERMI_CONSTANT).is_ok(),
        "the shipped constants must still converge"
    );
}

#[test]
fn test_the_w_mass_solver_reports_reaching_its_cap() {
    // Reachable, though not at the shipped constants: a physical Z mass with a 1000 GeV top slows
    // the contraction past 20 iterations. Found by sweeping 192 combinations of the five inputs,
    // of which 4 reach the cap — so the error path is not merely defensive.
    let err = solve_w_mass(Z_MASS, 1000.0, ALPHA_EM_MZ, ALPHA_EM_0, FERMI_CONSTANT)
        .expect_err("this input exhausts the 20-iteration cap");
    let message = format!("{err}");
    assert!(
        message.contains("did not converge"),
        "unexpected message: {message}"
    );
}

#[test]
fn test_non_convergence_is_distinguishable_from_the_negative_discriminant() {
    // The site's other numerical failure, which must not be conflated with the new one — the
    // reason this got its own variant rather than reusing `NumericalInstability`.
    let err = solve_w_mass(Z_MASS, TOP_MASS, 1.0 / 60.0, ALPHA_EM_0, FERMI_CONSTANT)
        .expect_err("a large alpha drives the discriminant negative");
    let message = format!("{err}");
    assert!(
        message.contains("Negative discriminant") && !message.contains("did not converge"),
        "unexpected message: {message}"
    );
}

// =============================================================================
// The mean-anomaly wrap
//
// `solve_kepler` reduces `M` into `[0, 2π)` with `m - 2π·floor(m/2π)` before iterating. Every case
// above supplies `M < 2π`, so `floor(..)` is zero and the whole expression is the identity — which
// is why mutation testing found the wrap unpinned: `-` for `+`, and `/` for `*`, both survived.
//
// Periodicity is the invariant that exercises it, and it needs no oracle: the two-body state after
// `t + T` is the state after `t`, for any `t` and any whole number of periods.
// =============================================================================

#[test]
fn test_the_state_is_periodic_across_the_mean_anomaly_wrap() {
    for (a, e) in [(1.0f64, 0.1f64), (1.0, 0.5), (2.0, 0.9), (1.0, 0.99)] {
        let orbit = periapsis_orbit(a, e);
        let period = orbit.period().unwrap();
        for frac in [0.25f64, 0.5, 0.75] {
            let (base, base_v) = orbit.propagate(frac * period).unwrap();
            // One, two and three whole periods later: the same place, at the same velocity.
            for laps in [1.0f64, 2.0, 3.0] {
                let (later, later_v) = orbit.propagate((frac + laps) * period).unwrap();
                let tol = 1e-8 * a.max(1.0);
                assert!(
                    (later[0] - base[0]).abs() < tol && (later[1] - base[1]).abs() < tol,
                    "e={e}, {frac} + {laps} periods: position {later:?} against {base:?}"
                );
                assert!(
                    (later_v[0] - base_v[0]).abs() < tol * 10.0
                        && (later_v[1] - base_v[1]).abs() < tol * 10.0,
                    "e={e}, {frac} + {laps} periods: velocity {later_v:?} against {base_v:?}"
                );
            }
        }
    }
}

// =============================================================================
// Kepler at f32: the stopping test has to be in units of the scalar's own resolution
//
// `solve_kepler` used a fixed absolute tolerance of `1e-15`. That is about `4.5` ulp at `f64` and
// eight decades below anything `f32` arithmetic can produce: with `E` and `M` both `O(1)` radians,
// the residual's floor at `f32` is a few times `1.19e-7`. So an `f32` iterate that solves the
// equation to the last bit the type holds failed the step test *and* the residual test, and came
// back as `NotConverged`.
//
// The oracle is the same bisection as above, run in `f64` — a different arithmetic as well as a
// different algorithm, so nothing about the `f32` path can flatter it.
// =============================================================================

/// The `f32` twin of [`periapsis_orbit`].
fn periapsis_orbit_f32(a: f32, e: f32) -> TwoBodyPropagator<f32> {
    let gm = 1.0f32;
    let rp = a * (1.0 - e);
    let vp = (gm * (1.0 + e) / rp).sqrt();
    TwoBodyPropagator::from_state([rp, 0.0], [0.0, vp], gm).unwrap()
}

#[test]
fn test_kepler_converges_at_f32_where_the_absolute_tolerance_refused_a_correct_iterate() {
    // `a = 2, e = 0.9, M = 0.25` is the measured case: `f64` converges on its seventh step and
    // `f32` returned `NotConverged` — not because the iterate was wrong, but because `1e-15` is
    // unreachable in `f32`. The `f64` control below is what makes that unambiguous.
    let (a, e, m) = (2.0f32, 0.9f32, 0.25f32);

    let orbit64 = periapsis_orbit(a as f64, e as f64);
    orbit64
        .propagate((m as f64) / orbit64.mean_motion())
        .expect("the f64 control converges, so the case is not genuinely hard");

    let orbit = periapsis_orbit_f32(a, e);
    let (position, _) = orbit
        .propagate(m / orbit.mean_motion())
        .expect("an f32 iterate that solves the equation to f32 resolution must be accepted");

    // The independent root, and the position it defines, both in f64.
    let root = kepler_root_by_bisection(m as f64, e as f64);
    let want_x = (a as f64) * (root.cos() - e as f64);
    let want_y = (a as f64) * (1.0 - (e as f64) * (e as f64)).sqrt() * root.sin();

    // `f32` carries about seven decimal digits; `1e-5` absolute against an orbit of size 2 is a
    // few dozen ulp, which is what a solve at this eccentricity can hold.
    assert!(
        (position[0] as f64 - want_x).abs() < 1e-5,
        "x: got {}, bisection gives {want_x}",
        position[0]
    );
    assert!(
        (position[1] as f64 - want_y).abs() < 1e-5,
        "y: got {}, bisection gives {want_y}",
        position[1]
    );
}

#[test]
fn test_kepler_agrees_with_the_bisection_at_f32_across_eccentricities() {
    // The same sweep as the `f64` case, run in the narrower scalar. Every one of these is a case
    // the fixed tolerance could only pass by luck — when the `f32` residual happened to round to
    // exactly zero — and `e = 0.9` is one where it did not.
    for (a, e, m) in [
        (1.0f32, 0.1f32, 0.7f32),
        (1.0, 0.5, 1.0),
        (2.0, 0.9, 0.25),
        (1.0, 0.3, 2.0),
        (1.0, 0.99, 0.01),
    ] {
        let orbit = periapsis_orbit_f32(a, e);
        let (position, _) = orbit
            .propagate(m / orbit.mean_motion())
            .unwrap_or_else(|err| panic!("e={e}, M={m}: {err}"));
        let root = kepler_root_by_bisection(m as f64, e as f64);
        let want_x = (a as f64) * (root.cos() - e as f64);
        let want_y = (a as f64) * (1.0 - (e as f64) * (e as f64)).sqrt() * root.sin();
        assert!(
            (position[0] as f64 - want_x).abs() < 1e-5
                && (position[1] as f64 - want_y).abs() < 1e-5,
            "e={e}, M={m}: got {position:?}, bisection gives ({want_x}, {want_y})"
        );
    }
}

#[test]
fn test_the_f32_refusal_still_fires_where_there_is_no_root_to_find() {
    // The other half of the change: widening the tolerance to the scalar's resolution must not
    // turn the refusal into a rubber stamp. A perfectly circular orbit has no defined periapsis,
    // so `from_state` produces a `NaN` mean anomaly and every Newton step is `NaN`; neither the
    // step test nor the residual test can be met at any tolerance, and the call must still fail.
    let orbit = periapsis_orbit_f32(1.0, 0.0);
    assert_eq!(orbit.eccentricity(), 0.0, "the orbit itself is well formed");
    let err = orbit
        .propagate(0.7)
        .expect_err("a NaN mean anomaly must not come back as a position");
    assert!(format!("{err}").contains("did not converge"), "got {err}");
}
