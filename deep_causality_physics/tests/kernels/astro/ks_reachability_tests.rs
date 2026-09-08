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

            for fraction in [1e-9f64, 1e-3, 0.5, 1.0, 10.0, 1000.0] {
                match propagator.propagate(period * fraction) {
                    Ok(_) => converged += 1,
                    Err(err) => failures.push(format!("e={e}, a={a:e}, {fraction} periods: {err}")),
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "the cap became reachable, so 7.5's latency finding no longer holds:\n{}",
        failures.join("\n")
    );
    assert_eq!(
        converged, 108,
        "the sweep must cover the whole grid; a skipped case would weaken the bound"
    );
}
