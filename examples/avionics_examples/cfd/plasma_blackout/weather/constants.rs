/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The weather-dispersion knobs. Everything the study shares with the corridor (the carrier,
//! the anchors, the baseline atmosphere, the flight physics, the finite-rate ionization
//! network, the navigation budget, the envelope) lives in
//! `avionics_examples::shared::constants`, with its model labels; this file adds only what
//! the dispersion table itself varies and gates.
//!
//! **Model labels:**
//!
//! * Each weather condition is the baseline atmosphere with a temperature offset and a density
//!   scale applied per row (sound speed rescaled by the square root of the temperature ratio).
//!   Day-to-day dispersion shapes, not reanalysis profiles.
//! * The INS thermal model: the accelerometer bias departs from its calibration point by a
//!   tactical-grade temperature coefficient, `bias * (1 + K_T * |dT|)`. The magnitude grows with
//!   the *departure* from standard conditions in either direction, because what degrades the
//!   navigation is the distance from the calibration point, not cold per se. The filter's priors
//!   stay standard-day in every world; that mismatch is the phenomenon under study.
//! * The dominant weather-to-navigation path needs no IMU model at all: the atmosphere sets the
//!   ionization, the ionization sets the blackout window, and the window sets how long the
//!   dead-reckoning drift integrates. The table separates the two effects by printing both the
//!   window and the drift.

/// Horizon (coupled steps) of each world's descent. Longer than the corridor's: the baseline
/// exit sits near 70 s from entry, and every weather variant needs room to exit *and* fold
/// reacquisition fixes before the horizon.
pub const STEPS: usize = 850;

/// The six weather conditions: `(name, dT K, density scale)`. The baseline plus five
/// dispersions. The polar-winter world compounds the two INS mechanisms (densest window and
/// largest thermal departure); the thin day pits them against each other (less ionization but
/// less deceleration).
pub const WEATHER: [(&str, f64, f64); 6] = [
    ("standard_day", 0.0, 1.0),
    ("hot_day", 20.0, 0.90),
    ("cold_day", -25.0, 1.10),
    ("polar_winter", -40.0, 1.20),
    ("thin_day", -5.0, 0.75),
    ("dense_day", 5.0, 1.30),
];

/// Accelerometer-bias thermal departure per Kelvin away from the calibration point (relative).
/// Tactical-grade class: a 40 K departure grows the uncompensated bias by 40 percent.
pub const IMU_THERMAL_COEFF_PER_K: f64 = 0.01;

// ── Monte Carlo

/// Receiver-noise realizations flown per weather condition. Each draw is a deterministic
/// phase-shifted low-discrepancy sequence (no RNG dependency; draw 0 is the corridor's
/// sequence), so the whole campaign remains bit-reproducible while the drift cells gain error
/// bars. Six conditions times eight draws is 48 full descents; the scoped fan-out packs them
/// onto the available cores.
pub const MC_DRAWS: usize = 8;

/// The significance requirement on the headline effect: the polar-winter mean drift must exceed
/// the standard-day mean by at least this many combined standard deviations, so the cold effect
/// is resolved above the receiver-noise scatter rather than within it.
pub const DRIFT_SIGNIFICANCE_SIGMA: f64 = 2.0;

// ── Gate thresholds (pinned from the measured dispersion; honest bands, not tuned fits)

/// Weather must move the blackout window: max onset minus min onset across the table, s. The
/// onset carries the weather signal (denser air ionizes earlier); the dwell is more robust
/// (measured spread 3.1 s vs the onset's 4.2 s under the finite-rate network), so the window
/// gate pins the onset.
pub const MIN_ONSET_SPREAD_S: f64 = 2.0;
/// The polar-winter blackout drift must exceed the standard day's by at least this factor
/// (the INS-does-not-behave-as-assumed gate).
pub const COLD_DRIFT_FACTOR_MIN: f64 = 1.2;
/// Every world must reacquire: terminal navigation error ceiling, m.
pub const REACQ_ERR_MAX_M: f64 = 1.0;
/// Wall-clock budget for the whole table, s.
pub const WALL_CLOCK_BUDGET_S: f64 = 600.0;
