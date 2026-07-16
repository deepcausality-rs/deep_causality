/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The corridor's own knobs: the descent horizon, the counterfactual bank sweep, and the gate
//! thresholds. Everything the corridor shares with the weather-dispersion example (the carrier,
//! the anchors, the atmosphere, the flight physics, the finite-rate ionization network, the
//! navigation budget, the envelope) lives in `avionics_examples::shared::constants`, with
//! its model labels and the precision notes.

/// Horizon (coupled steps) of any single leg; predicates pause earlier.
pub const STEPS: usize = 700;

// ── Branch study

/// Steps each counterfactual branch continues past the shared blackout onset.
pub const BRANCH_STEPS: usize = 100;
/// Candidate commanded bank angles (degrees). Zero is the ballistic reference; the fine sweep
/// brackets the reachable optimum (the miss landscape bottoms out near 15 deg for the
/// configured aim); 40 deg exceeds the envelope cap, flies clamped, and overshoots, showing
/// that commanding more bank than the certified envelope allows buys a worse trajectory. The
/// scoped fan-out flies all six concurrently, so the sweep costs one branch of wall-clock.
pub const BANK_ANGLES_DEG: [f64; 6] = [0.0, 5.0, 10.0, 15.0, 20.0, 40.0];
/// Fine-sweep half-width around the coarse winner, in fine steps. Five steps up and five down
/// at [`FINE_STEP_DEG`] brackets the coarse winner by half a coarse interval on each side, so
/// together the two rounds resolve the miss landscape at 0.5 deg for the cost of one extra
/// branch fan-out from the same paused onset state.
pub const FINE_SPAN_STEPS: usize = 5;
/// Fine-sweep resolution, degrees.
pub const FINE_STEP_DEG: f64 = 0.5;
/// Cross-range offset of the aim point from the ballistic terminal state, m (in the lift-plane
/// side direction a positive bank pushes toward). Sized so the optimum bank sits *inside* the
/// envelope cap and between sweep candidates: the sweep has to find it, and a finer sweep finds
/// it better. Re-pinned for the finite-rate network's higher flow-resolved onset (74.7 km):
/// the branches fork in thinner air, so the reachable cross-range over the branch dwell is
/// smaller (measured: 33.5 m at the clamped 40 deg, ~21 m at 15 deg).
pub const AIM_CROSS_RANGE_M: f64 = 20.0;
/// The value-of-counterfactuals gate: the committed branch's trajectory-derived miss must beat
/// the ballistic branch's by at least this factor.
pub const MISS_IMPROVEMENT_FACTOR: f64 = 3.0;
/// Steps flown after the flow-resolved exit so the reacquired fixes collapse the drift.
pub const REACQ_STEPS: usize = 30;

// ── Gate thresholds

/// The RAM-C II blackout-exit flight window, km: the flight signal recovery fell at 25 to 30 km
/// on descent. Reported for comparison; the corridor's own gate band is
/// [`EXIT_ALTITUDE_BAND_KM`], because the two vehicles differ by design (see there).
pub const RAMC_EXIT_WINDOW_KM: (f64, f64) = (25.0, 30.0);
/// Pinned acceptance band for the corridor's flow-resolved blackout-exit altitude, km.
/// Measured 47.0 km with the uncalibrated finite-rate network (see output.txt). The corridor's probe flies a
/// deliberately light ballistic bundle (`CDA_OVER_M`, β ≈ 170 kg/m²) so the compressed descent
/// decelerates below the ionization threshold before the atmosphere-table floor; it therefore
/// exits well above the RAM-C II window, and the offset is ballistics, not chemistry. The band
/// catches regressions in either.
pub const EXIT_ALTITUDE_BAND_KM: (f64, f64) = (40.0, 50.0);
/// Minimum separation between the committed steered terminal state and the zero-bank terminal
/// state, m (the steering-is-real gate).
pub const DIVERGENCE_MIN_M: f64 = 1.0;
/// Maximum solver rebuilds while following the schedule.
pub const MAX_REBUILDS: usize = 3;
/// Wall-clock budget for the whole example, s (the minutes-not-hours pin).
pub const WALL_CLOCK_BUDGET_S: f64 = 600.0;
