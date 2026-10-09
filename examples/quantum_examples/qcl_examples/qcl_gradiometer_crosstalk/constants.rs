/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the gradiometer-crosstalk example.
//!
//! The instrument is the dual-cloud gradiometer of Sorrentino et al., *Sensitivity limits of a
//! Raman atom interferometer as a gravity gradiometer*, Phys. Rev. A 89, 023607 (2014),
//! arXiv:1312.3741. Values marked **placeholder** have no published source: no paper read for this
//! example measures light leaking between the two clouds' detection signals, so a lab's own
//! measurement replaces them.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

// =============================================================================
// The instrument (Sorrentino et al. 2014)
// =============================================================================

/// The rubidium D2 wavelength, in m. The two-photon wave number is `4π/λ`.
pub const WAVELENGTH: FloatType = const_scalar_from_float!(FloatType, 780.241e-9);

/// The time between pulses, `T`, in s: "T = 160 ms" (§IV C).
pub const INTERROGATION_TIME: FloatType = const_scalar_from_float!(FloatType, 0.160);

/// The fringe contrast: the fitted fringe amplitudes are `A ≃ C ≃ 0.225` (§III A), and a
/// contrast is twice an amplitude.
pub const CONTRAST: FloatType = const_scalar_from_float!(FloatType, 0.45);

/// One measurement cycle, in s: "each point is acquired in 1.9 s" (Fig. 2).
pub const CYCLE_TIME: FloatType = const_scalar_from_float!(FloatType, 1.9);

/// The short-term sensitivity to differential acceleration, in units of g per √Hz (abstract).
pub const SENSITIVITY_G: FloatType = const_scalar_from_float!(FloatType, 3.0e-9);

/// Standard gravity, in m/s², which turns the sensitivity into (m/s²)/√Hz.
pub const STANDARD_GRAVITY: FloatType = const_scalar_from_float!(FloatType, 9.80665);

/// The longest averaging the paper reports, in s: a resolution of 5 × 10⁻¹¹ g after 8000 s
/// (abstract), which sizing treats as the white-noise range.
pub const WHITE_NOISE_RANGE: FloatType = const_scalar_from_int!(FloatType, 8000);

/// The vertical separation of the two interferometers, in m: the clouds reach their apogees "at
/// about 60 cm and 90 cm above the MOT" (§II).
pub const BASELINE: FloatType = const_scalar_from_float!(FloatType, 0.30);

// =============================================================================
// The site
// =============================================================================

/// The normal free-air gravity gradient, 0.3086 mGal/m, in s⁻².
pub const GRAVITY_GRADIENT: FloatType = const_scalar_from_float!(FloatType, 3.086e-6);

/// Earth's rotation rate, in rad/s (IERS conventional value).
pub const EARTH_RATE: FloatType = const_scalar_from_float!(FloatType, 7.292115e-5);

/// The latitude of the laboratory, Sesto Fiorentino (the paper's affiliation), in degrees.
pub const LATITUDE_DEG: FloatType = const_scalar_from_float!(FloatType, 43.8);

// =============================================================================
// Placeholders: the lab's own measurements
// =============================================================================

/// **Placeholder.** The fraction of a dark cloud's detection signal that reads excited, from
/// background light alone.
pub const DARK_BACKGROUND: FloatType = const_scalar_from_float!(FloatType, 0.02);

/// **Placeholder.** The leak the leak hypotheses posit: the fraction of one cloud's fluorescence
/// that reaches the other cloud's detection signal.
pub const LEAK: FloatType = const_scalar_from_float!(FloatType, 0.06);

/// **Placeholder.** The dead time of reprogramming the pulse sequence to brighten one cloud and
/// darken the other, in s.
pub const SETUP_TIME: FloatType = const_scalar_from_int!(FloatType, 60);

/// **Placeholder.** The dead time of mounting the sensor head on a motion platform, in s.
pub const PLATFORM_SETUP_TIME: FloatType = const_scalar_from_int!(FloatType, 600);

/// The rotation rate about a horizontal axis that the rotation step applies, in rad/s: a
/// configuration choice, large enough that the step's centrifugal phase is resolved inside the
/// white-noise range.
pub const STEP_RATE: FloatType = const_scalar_from_float!(FloatType, 1.0e-3);

// =============================================================================
// The run
// =============================================================================

/// The separation, in bits, at which an experiment resolves a pair of causes.
pub const FLOOR_BITS: FloatType = const_scalar_from_int!(FloatType, 5);

/// How many standard errors a world's prediction may sit from the observation and still hold.
pub const AGREEMENT_SIGMAS: FloatType = const_scalar_from_int!(FloatType, 3);

/// How far, in read-out probability, a context change may move the planned experiment's
/// predictions before the campaign re-plans.
pub const DRIFT: FloatType = const_scalar_from_float!(FloatType, 0.01);

/// The seed every simulated observation is drawn from.
pub const SEED: u64 = 20261008;

/// The shots an experiment is declared with; planning in time replaces them with the fewest that
/// reach the floor.
pub const DECLARED_SHOTS: u64 = 1;

// =============================================================================
// The nodes, one system each
// =============================================================================

/// Cloud A's atoms, excited or not.
pub const ATOMS_A: usize = 0;
/// Cloud B's atoms.
pub const ATOMS_B: usize = 1;
/// Cloud A's detection signal, reading excited or not.
pub const SIGNAL_A: usize = 2;
/// Cloud B's detection signal.
pub const SIGNAL_B: usize = 3;
/// The phase both clouds share, from the common Raman laser and its mirror.
pub const SHARED_PHASE: usize = 4;
/// The platform's rotation rate about a horizontal axis: the operating rate, or the step's.
pub const ROTATION: usize = 5;

/// The values the shared phase takes, equally spaced over a turn. Four points average every
/// product of at most three fringe terms exactly, which covers each candidate's predictions.
pub const PHASE_POINTS: usize = 4;
