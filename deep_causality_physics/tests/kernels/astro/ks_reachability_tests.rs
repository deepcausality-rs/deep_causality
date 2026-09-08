/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The latency measurement behind `unified-math-next` task 7.5.
//!
//! `KsPropagator::solve_fictitious_time` inverts `t(s) = dt` by Newton iteration under a 100-step
//! cap. That cap now returns `PhysicsError::NotConverged` instead of the last iterate, and the
//! stage had to say whether the silence it replaced was ever reachable.
//!
//! It is not, within the bound this test fixes: 108 combinations spanning eccentricity `0` to
//! `0.9999`, three orbit sizes from low-Earth to near-lunar, and elapsed times from a billionth of
//! a period to a thousand periods. Every one converges.
//!
//! That is a **bound, not a proof** — the same standard the Meek stage used when it searched for an
//! R4 firing and recorded the search rather than concluding from it. The error path is kept because
//! the function is public and its callers are not fixed. Keeping the measurement executable is what
//! stops it decaying into a claim: if a future change makes the cap reachable, this test says so.
//!
//! # Why this asserts an invariant and not just `Ok`
//!
//! It used to assert only that every case returned `Ok`, and that proved too weak twice over. It
//! says nothing about whether the returned state is *right*, and — because the solver's acceptance
//! then depended on where an oscillating iteration happened to stop — it was not even stable across
//! platforms: this sweep passed on macOS/aarch64 and reported `NotConverged` for
//! `e = 0.9999, a = 7e6` on the Linux/x86_64 CI, the two libms differing only in the last bits of
//! `sin` and `cos`. The solver's convergence test was mis-scaled; see `solve_fictitious_time`.
//!
//! Every case now also has to conserve the specific orbital energy `v²/2 − μ/r`, which is constant
//! along a Kepler orbit and is the property a caller actually depends on. A solver that returned
//! `Ok` with the wrong root would satisfy the old assertion and fail this one.

use deep_causality_physics::KsPropagator;

const EARTH_GM: f64 = 3.986_004_418e14;

#[test]
fn test_the_fictitious_time_inversion_converges_across_its_input_space() {
    let mut converged = 0;
    let mut failures = Vec::new();

    for e in [0.0f64, 0.1, 0.5, 0.9, 0.99, 0.9999] {
        for a in [7.0e6f64, 1.0e7, 1.0e9] {
            let rp = a * (1.0 - e);
            if rp <= 0.0 {
                continue;
            }
            let vp = (EARTH_GM * (1.0 + e) / rp).sqrt();
            let propagator = KsPropagator::from_state([rp, 0.0, 0.0], [0.0, vp, 0.0], EARTH_GM)
                .expect("a bound orbit is well formed");
            let period = propagator.period().expect("a bound orbit has a period");

            // The specific orbital energy of the initial state, from vis-viva at periapsis.
            let energy0 = vp * vp / 2.0 - EARTH_GM / rp;

            for fraction in [1e-9f64, 1e-3, 0.5, 1.0, 10.0, 1000.0] {
                match propagator.propagate(period * fraction) {
                    Ok((r, v)) => {
                        converged += 1;
                        let radius = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
                        let speed_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
                        let energy = speed_sq / 2.0 - EARTH_GM / radius;
                        // Relative to the energy scale, `μ / 2a`, not to `energy0` — the two differ
                        // only by sign for a bound orbit, but the scale is what a tolerance means.
                        let scale = EARTH_GM / (2.0 * a);
                        let drift = (energy - energy0).abs() / scale;
                        // `is_nan` explicitly rather than a negated `<`: a NaN energy is a
                        // failure and must not slip through a comparison that is false either way.
                        if drift.is_nan() || drift >= 1e-9 {
                            failures.push(format!(
                                "e={e}, a={a:e}, {fraction} periods: energy drifted by {drift:e} \
                                 of μ/2a (returned Ok, so the root is wrong rather than refused)"
                            ));
                        }
                    }
                    Err(err) => failures.push(format!("e={e}, a={a:e}, {fraction} periods: {err}")),
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "the sweep no longer converges to an energy-conserving state, so 7.5's latency \
         finding no longer holds:\n{}",
        failures.join("\n")
    );
    assert_eq!(
        converged, 108,
        "the sweep must cover the whole grid; a skipped case would weaken the bound"
    );
}
