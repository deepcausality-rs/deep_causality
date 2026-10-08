/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration constants for the gravimeter-systematics example.
//!
//! The instrument is the cold-atom gravimeter of Louchet-Chauvet et al., *The influence of
//! transverse motion within an atomic gravimeter*, New J. Phys. 13, 065025 (2011). Each candidate's
//! response cites the paper it comes from. Values marked **placeholder** have no published source
//! for this instrument; a lab's own measurement replaces them.

use crate::FloatType;
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};

// =============================================================================
// The instrument (Louchet-Chauvet et al. 2011)
// =============================================================================

/// The rubidium D2 wavelength, in m. The two-photon wave number is `4π/λ`.
pub const WAVELENGTH: FloatType = const_scalar_from_float!(FloatType, 780.241e-9);

/// The time between pulses, `T`, in s: "free evolution times of T = 70 ms" (§2).
pub const INTERROGATION_TIME: FloatType = const_scalar_from_float!(FloatType, 0.070);

/// The repetition rate, in Hz: "a repetition rate of about 3 Hz" (§2).
pub const REPETITION_RATE: FloatType = const_scalar_from_int!(FloatType, 3);

/// The four-configuration protocol's sensitivity at 1 s, in µGal (§3.3).
pub const PROTOCOL_SENSITIVITY_UGAL: FloatType = const_scalar_from_int!(FloatType, 70);

/// The variance factor by which the four-configuration protocol degrades the sensitivity of one
/// configuration: "5/2 × 4 = 10" (§3.2).
pub const PROTOCOL_DEGRADATION: FloatType = const_scalar_from_int!(FloatType, 10);

/// The longest averaging over which the noise stays white, in s: the Allan deviation "averages
/// down as white noise" and stays below 1 µGal after 5000 s before it flickers (§3.3).
pub const WHITE_NOISE_RANGE: FloatType = const_scalar_from_int!(FloatType, 5000);

/// The fringe contrast. The papers on this instrument do not state it; this is the 40 % of the
/// transportable rubidium gravimeter of Ménoret et al., Sci. Rep. 8, 12300 (2018),
/// arXiv:1809.04908 ("T = 60 ms, C = 40%"). Sizing does not depend on it: a lower contrast flattens
/// the fringe and lengthens each effective draw alike.
pub const CONTRAST: FloatType = const_scalar_from_float!(FloatType, 0.40);

/// The atom temperature, in K: "the atoms fall at a temperature of 2 µK" (§2).
pub const ATOM_TEMPERATURE: FloatType = const_scalar_from_float!(FloatType, 2.0e-6);

/// The bias field, in T: "a 10 mG bias field" on the same instrument (Farah et al.,
/// arXiv:1406.5998, §II). Every response reads the field as a ratio to it.
pub const BIAS_FIELD: FloatType = const_scalar_from_float!(FloatType, 1.0e-6);

/// The effective Rabi frequency, in rad/s: a π/2 pulse "of order of 10 µs" (Farah et al., §II)
/// gives `π/(2 · 10 µs)`. Every response reads it as a ratio.
pub const RABI_FREQUENCY: FloatType = const_scalar_from_float!(FloatType, 1.5707963267948966e5);

/// Standard gravity, in m/s², which a tilt projects.
pub const STANDARD_GRAVITY: FloatType = const_scalar_from_float!(FloatType, 9.80665);

/// One µGal, in m/s².
pub const MICRO_GAL: FloatType = const_scalar_from_float!(FloatType, 1.0e-8);

// =============================================================================
// The offset to explain
// =============================================================================

/// The offset the gravimeter reads against a trusted reference after the tide correction, in µGal.
pub const OFFSET_UGAL: FloatType = const_scalar_from_int!(FloatType, -5);

/// The offset of the run that shows the baseline refusing tilt, in µGal.
pub const POSITIVE_OFFSET_UGAL: FloatType = const_scalar_from_int!(FloatType, 5);

// =============================================================================
// The candidates' responses
// =============================================================================

/// The quadratic Zeeman bias at a coil current `s` times nominal is `a s² + b s + c` (Hu et al.,
/// arXiv:1805.05159, eq. 9, with the field split as `s B_sn + B_bg` by eqs. 7 and 8): `a` from
/// the coil's own inhomogeneity, `b` from the coil field across the background's variation, `c`
/// from the background alone. These are the shares of the bias at nominal current in GAIN, from
/// its field maps at 13 mA and 6.5 mA (Fig. 6(a) and (b)) digitised and integrated along the
/// fountain trajectory through the printed pulse heights, by
/// `deep_causality_quantum/papers/digitised/digitise_hu2017_fig6.py`. The nominal map gives
/// 2.01 µGal against the printed 2.04 µGal.
pub const ZEEMAN_COIL_SHARE: FloatType = const_scalar_from_float!(FloatType, -0.038);
/// See [`ZEEMAN_COIL_SHARE`].
pub const ZEEMAN_CROSS_SHARE: FloatType = const_scalar_from_float!(FloatType, 1.005);
/// See [`ZEEMAN_COIL_SHARE`].
pub const ZEEMAN_BACKGROUND_SHARE: FloatType = const_scalar_from_float!(FloatType, 0.033);

/// The two-photon light shift's counter-propagating part and its co-propagating part, in mrad:
/// "a TPLS of about 22 mrad for counter-propagating transition ... we find a 11 mrad for
/// co-propagating ones" (Gauguet et al., §III). The co-propagating part comes from the
/// magnetically sensitive transitions (eq. 5), so it is the share a field step moves.
pub const LIGHT_SHIFT_COUNTER_MRAD: FloatType = const_scalar_from_int!(FloatType, 22);
/// See [`LIGHT_SHIFT_COUNTER_MRAD`].
pub const LIGHT_SHIFT_CO_MRAD: FloatType = const_scalar_from_int!(FloatType, 11);

/// The clipping shift per mm of initial cloud displacement along east-west, in µGal/mm:
/// "14.2(1.1) µGal/mm along EW" (Farah et al., arXiv:1406.5998, §III).
pub const CLIPPING_SLOPE_UGAL_PER_MM: FloatType = const_scalar_from_float!(FloatType, 14.2);

// =============================================================================
// The experiments
// =============================================================================

/// The deliberate tilt the tilt experiment applies, in rad: the end of the tilt scan Ménoret et al.
/// apply to calibrate verticality, "Applied tilts typically range from 0 to 1.5 mrad" (Methods).
pub const TILT_STEP: FloatType = const_scalar_from_float!(FloatType, 1.5e-3);

/// The field step: half the nominal coil current, as Hu et al. mapped the field at 6.5 mA against
/// 13 mA.
pub const FIELD_STEP: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The Rabi step: half the Rabi frequency at constant pulse area, as Louchet-Chauvet et al.'s
/// protocol uses (§3.2).
pub const RABI_STEP: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The east-west displacement of the initial cloud, in m: inside the ±1.6 mm over which Farah et al.
/// displaced the cloud and measured the clipping shift (Fig. 4).
pub const CLOUD_DISPLACEMENT: FloatType = const_scalar_from_float!(FloatType, 1.0e-3);

/// **Placeholder.** Each experiment's setup time, in s: the dead time before its draws start. No
/// paper read for this example reports the dead time of a configuration change.
pub const SETUP_K_REVERSAL: FloatType = const_scalar_from_int!(FloatType, 30);
/// See [`SETUP_K_REVERSAL`]: rotating the sensor head and re-levelling it.
pub const SETUP_TURN: FloatType = const_scalar_from_int!(FloatType, 600);
/// See [`SETUP_K_REVERSAL`]: the accelerometer correction is applied to the shots as taken.
pub const SETUP_ACCELEROMETER: FloatType = const_scalar_from_int!(FloatType, 0);
/// See [`SETUP_K_REVERSAL`].
pub const SETUP_TILT: FloatType = const_scalar_from_int!(FloatType, 900);
/// See [`SETUP_K_REVERSAL`].
pub const SETUP_FIELD: FloatType = const_scalar_from_int!(FloatType, 60);
/// The temperature scan's setup, in s, derived from Karcher et al. (arXiv:1804.04909): ultracold
/// operation runs at 1,200 to 3,000 nm/s² at 1 s with a 4.49 s cycle, which puts each temperature
/// point at 10⁴ s or more, and a scan takes at least two.
pub const SETUP_TEMPERATURE: FloatType = const_scalar_from_int!(FloatType, 20000);
/// See [`SETUP_K_REVERSAL`].
pub const SETUP_RABI: FloatType = const_scalar_from_int!(FloatType, 60);
/// See [`SETUP_K_REVERSAL`].
pub const SETUP_CLOUD: FloatType = const_scalar_from_int!(FloatType, 300);

// =============================================================================
// The tide
// =============================================================================

/// **Placeholder.** The site's tide as the principal lunar semidiurnal term alone: its amplitude,
/// in µGal, and its phase at the start of the run, in rad. The tide is the site's, so a site's tide
/// model replaces both; Louchet-Chauvet et al. report only its mean correction over twelve days,
/// −18.8 µGal (Table 1).
pub const TIDE_AMPLITUDE_UGAL: FloatType = const_scalar_from_int!(FloatType, 80);
/// See [`TIDE_AMPLITUDE_UGAL`].
pub const TIDE_PHASE: FloatType = const_scalar_from_float!(FloatType, 0.3);

/// The period of the principal lunar semidiurnal tide, M2, in s: 12.4206 h.
pub const TIDE_PERIOD: FloatType = const_scalar_from_float!(FloatType, 44714.16);

/// The ambient temperature the environment readings record, in K.
pub const AMBIENT_TEMPERATURE: FloatType = const_scalar_from_float!(FloatType, 296.15);

// =============================================================================
// The run
// =============================================================================

/// The separation, in bits, at which an experiment resolves a pair of causes.
pub const FLOOR_BITS: FloatType = const_scalar_from_int!(FloatType, 5);

/// How many standard errors a prediction may sit from the observation and still hold.
pub const AGREEMENT_SIGMAS: FloatType = const_scalar_from_int!(FloatType, 3);

/// How far, in read-out probability, the tide may move the planned experiment's predictions
/// before the campaign re-plans. Zero: every move re-plans, so each experiment's draws are sized
/// at the predictions it is judged against. A tolerance keeps draws sized at an earlier tide, and
/// a pair they were sized to separate at exactly the floor can fall just short of it.
pub const DRIFT: FloatType = const_scalar_from_int!(FloatType, 0);

/// The seed every simulated observation is drawn from.
pub const SEED: u64 = 20261008;

/// The shots an experiment is declared with; planning in time replaces them with the fewest that
/// reach the floor.
pub const DECLARED_SHOTS: u64 = 1;
