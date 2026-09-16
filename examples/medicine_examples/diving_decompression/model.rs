/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the SCUBA decompression planner: the Bühlmann ZH-L16C constants, the state
//! types, and the physiology.
//!
//! Sixteen tissue compartments absorb and release nitrogen at sixteen different rates, so every
//! compartment computation pairs a tension with the constant that belongs to it. That pairing is
//! what `ZipTensorWitness::zip_with` does: it walks two tensors slot by slot and combines each
//! pair. The loading law is therefore written once, and the witness carries it across all sixteen.
//!
//! # Where the constants live
//!
//! Every table below is declared **at the working type**, through `const_scalar_from_int!` for a
//! whole number and `const_scalar_from_float!` for a decimal. The compiler resolves them against
//! the alias in `main`, so switching that alias re-declares every one of them and no lift runs at
//! any call site.
//!
//! Every constant in this file is declared at the working type, and no `lift` runs anywhere in it.
//! The one scalar-generic body, [`SchreinerCurve::run`], reaches for nothing outside its arguments:
//! it names the Schreiner equation and builds `ln 2` from `S::one()`, so it holds no literal to
//! declare in the first place.

use crate::{FloatType, ascend, descend, hold_bottom, surface};
use deep_causality_algebra::Real;
use deep_causality_calculus::{DifferentiableField, DifferentiateFieldExt, Scalar};
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int};
use deep_causality_tensor::{CausalTensor, CausalTensorError};

// =============================================================================
// The small whole numbers the physiology is written with
// =============================================================================

pub const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
pub const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
pub const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
pub const TEN: FloatType = const_scalar_from_int!(FloatType, 10);
const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);

// =============================================================================
// Bühlmann ZH-L16C parameters
// =============================================================================

/// The number of tissue compartments the algorithm tracks.
pub const COMPARTMENTS: usize = 16;

/// Compartment half-times for nitrogen, in minutes. The fast compartments drive a short dive; the
/// slow ones control a long one.
pub const HALF_TIMES: [FloatType; COMPARTMENTS] = [
    const_scalar_from_int!(FloatType, 5),
    const_scalar_from_int!(FloatType, 8),
    const_scalar_from_float!(FloatType, 12.5),
    const_scalar_from_float!(FloatType, 18.5),
    const_scalar_from_int!(FloatType, 27),
    const_scalar_from_float!(FloatType, 38.3),
    const_scalar_from_float!(FloatType, 54.3),
    const_scalar_from_int!(FloatType, 77),
    const_scalar_from_int!(FloatType, 109),
    const_scalar_from_int!(FloatType, 146),
    const_scalar_from_int!(FloatType, 187),
    const_scalar_from_int!(FloatType, 239),
    const_scalar_from_int!(FloatType, 305),
    const_scalar_from_int!(FloatType, 390),
    const_scalar_from_int!(FloatType, 498),
    const_scalar_from_int!(FloatType, 635),
];

/// M-value `a` coefficients, in bar.
const A_COEFFICIENTS: [FloatType; COMPARTMENTS] = [
    const_scalar_from_float!(FloatType, 1.1696),
    const_scalar_from_int!(FloatType, 1),
    const_scalar_from_float!(FloatType, 0.8618),
    const_scalar_from_float!(FloatType, 0.7562),
    const_scalar_from_float!(FloatType, 0.6200),
    const_scalar_from_float!(FloatType, 0.5043),
    const_scalar_from_float!(FloatType, 0.4410),
    const_scalar_from_float!(FloatType, 0.4000),
    const_scalar_from_float!(FloatType, 0.3750),
    const_scalar_from_float!(FloatType, 0.3500),
    const_scalar_from_float!(FloatType, 0.3295),
    const_scalar_from_float!(FloatType, 0.3065),
    const_scalar_from_float!(FloatType, 0.2835),
    const_scalar_from_float!(FloatType, 0.2610),
    const_scalar_from_float!(FloatType, 0.2480),
    const_scalar_from_float!(FloatType, 0.2327),
];

/// M-value `b` coefficients, dimensionless.
const B_COEFFICIENTS: [FloatType; COMPARTMENTS] = [
    const_scalar_from_float!(FloatType, 0.5578),
    const_scalar_from_float!(FloatType, 0.6514),
    const_scalar_from_float!(FloatType, 0.7222),
    const_scalar_from_float!(FloatType, 0.7825),
    const_scalar_from_float!(FloatType, 0.8126),
    const_scalar_from_float!(FloatType, 0.8434),
    const_scalar_from_float!(FloatType, 0.8693),
    const_scalar_from_float!(FloatType, 0.8910),
    const_scalar_from_float!(FloatType, 0.9092),
    const_scalar_from_float!(FloatType, 0.9222),
    const_scalar_from_float!(FloatType, 0.9319),
    const_scalar_from_float!(FloatType, 0.9403),
    const_scalar_from_float!(FloatType, 0.9477),
    const_scalar_from_float!(FloatType, 0.9544),
    const_scalar_from_float!(FloatType, 0.9602),
    const_scalar_from_float!(FloatType, 0.9653),
];

/// Nitrogen partial pressure at the surface, in bar.
const SURFACE_N2_PP: FloatType = const_scalar_from_float!(FloatType, 0.79);

/// Nitrogen and oxygen fractions in air, and the lung's water-vapour pressure in bar at 37 °C.
const F_N2: FloatType = const_scalar_from_float!(FloatType, 0.79);
const F_O2: FloatType = const_scalar_from_float!(FloatType, 0.21);
const P_WATER_VAPOUR: FloatType = const_scalar_from_float!(FloatType, 0.0627);

/// The gradient factor the ceiling is computed against, for conservative recreational diving.
///
/// A full planner interpolates from a lower factor at depth to this one at the surface. This one
/// holds a single factor for the whole ascent, which is the simplification the README records.
pub const GF_HIGH: FloatType = const_scalar_from_float!(FloatType, 0.85);

/// Descent and ascent rates, in metres per minute. The ascent rate is the PADI standard.
pub const DESCENT_RATE: FloatType = const_scalar_from_int!(FloatType, 18);
pub const ASCENT_RATE: FloatType = const_scalar_from_int!(FloatType, 9);

/// NOAA CNS oxygen-toxicity limits as `(ppO2 in bar, maximum exposure in minutes)`.
const CNS_LIMITS: [(FloatType, FloatType); 7] = [
    (
        const_scalar_from_float!(FloatType, 1.60),
        const_scalar_from_int!(FloatType, 45),
    ),
    (
        const_scalar_from_float!(FloatType, 1.50),
        const_scalar_from_int!(FloatType, 120),
    ),
    (
        const_scalar_from_float!(FloatType, 1.40),
        const_scalar_from_int!(FloatType, 150),
    ),
    (
        const_scalar_from_float!(FloatType, 1.30),
        const_scalar_from_int!(FloatType, 180),
    ),
    (
        const_scalar_from_float!(FloatType, 1.20),
        const_scalar_from_int!(FloatType, 210),
    ),
    (
        const_scalar_from_float!(FloatType, 1.10),
        const_scalar_from_int!(FloatType, 240),
    ),
    (
        const_scalar_from_int!(FloatType, 1),
        const_scalar_from_int!(FloatType, 300),
    ),
];

/// A conservative no-decompression limit per depth band, as `(depth in metres, limit in minutes)`.
const NDL_TABLE: [(FloatType, FloatType); 7] = [
    (
        const_scalar_from_int!(FloatType, 12),
        const_scalar_from_int!(FloatType, 200),
    ),
    (
        const_scalar_from_int!(FloatType, 18),
        const_scalar_from_int!(FloatType, 80),
    ),
    (
        const_scalar_from_int!(FloatType, 24),
        const_scalar_from_int!(FloatType, 45),
    ),
    (
        const_scalar_from_int!(FloatType, 30),
        const_scalar_from_int!(FloatType, 25),
    ),
    (
        const_scalar_from_int!(FloatType, 36),
        const_scalar_from_int!(FloatType, 15),
    ),
    (
        const_scalar_from_int!(FloatType, 42),
        const_scalar_from_int!(FloatType, 10),
    ),
    (
        const_scalar_from_int!(FloatType, 48),
        const_scalar_from_int!(FloatType, 8),
    ),
];

/// The limit past the deepest band in [`NDL_TABLE`], in minutes.
const NDL_BEYOND_TABLE: FloatType = const_scalar_from_int!(FloatType, 6);

/// Ascent proceeds in steps of this many metres.
pub const ASCENT_STEP_M: FloatType = const_scalar_from_int!(FloatType, 3);
/// Above this depth the ascent runs to the surface, and the safety stop covers the last stretch.
pub const DECO_CLEARANCE_M: FloatType = const_scalar_from_int!(FloatType, 6);
/// A decompression stop lasts at least this long, in minutes.
pub const MIN_STOP_MINUTES: FloatType = const_scalar_from_int!(FloatType, 2);
/// Dives to at least this depth carry a safety stop.
pub const SAFETY_STOP_DEPTH_THRESHOLD_M: FloatType = const_scalar_from_int!(FloatType, 15);
/// The safety stop itself: depth in metres, duration in minutes.
pub const SAFETY_STOP_M: FloatType = const_scalar_from_int!(FloatType, 5);
pub const SAFETY_STOP_MINUTES: FloatType = const_scalar_from_int!(FloatType, 3);

/// The depths the printed table covers, in metres.
pub const TABLE_DEPTHS_M: [FloatType; 9] = [
    const_scalar_from_int!(FloatType, 10),
    const_scalar_from_int!(FloatType, 15),
    const_scalar_from_int!(FloatType, 20),
    const_scalar_from_int!(FloatType, 25),
    const_scalar_from_int!(FloatType, 30),
    const_scalar_from_int!(FloatType, 35),
    const_scalar_from_int!(FloatType, 40),
    const_scalar_from_int!(FloatType, 45),
    const_scalar_from_int!(FloatType, 50),
];

/// The bottom time the table uses for each depth: the no-decompression limit, held to this cap so
/// every row runs in the same handful of milliseconds.
pub const TABLE_BOTTOM_CAP_MINUTES: FloatType = const_scalar_from_int!(FloatType, 20);

// =============================================================================
// Types
// =============================================================================

/// A decompression stop: hold at `depth_m` for `minutes`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecoStop {
    pub depth_m: FloatType,
    pub minutes: FloatType,
}

/// The diver's physiological state as the dive proceeds.
///
/// The decompression obligation is a typed field, so a phase that discovers a stop records it
/// directly and the caller reads it back as data.
#[derive(Debug, Clone)]
pub struct DiverState {
    pub depth_m: FloatType,
    pub elapsed_minutes: FloatType,
    pub tissue_tensions: CausalTensor<FloatType>,
    pub cns_percent: FloatType,
    pub deco_stops: Vec<DecoStop>,
    /// The compartment holding the highest ceiling once the bottom phase ends, and that ceiling
    /// in metres. The bottom phase is where tissue loading peaks, which is the reading a dive
    /// plan quotes.
    pub controlling_at_bottom: usize,
    pub ceiling_at_bottom_m: FloatType,
}

impl DiverState {
    /// A diver at the surface, fully off-gassed and equilibrated with air.
    pub fn at_surface() -> Result<Self, CausalTensorError> {
        Ok(Self {
            depth_m: ZERO,
            elapsed_minutes: ZERO,
            tissue_tensions: CausalTensor::new(
                vec![SURFACE_N2_PP; COMPARTMENTS],
                vec![COMPARTMENTS],
            )?,
            cns_percent: ZERO,
            deco_stops: Vec::new(),
            controlling_at_bottom: 0,
            ceiling_at_bottom_m: ZERO,
        })
    }
}

/// Everything the run computed, which is everything the printer renders. The dive is simulated
/// once and this carries the result.
#[derive(Debug, Clone)]
pub struct DiveProfile {
    pub max_depth_m: FloatType,
    pub bottom_minutes: FloatType,
    pub total_minutes: FloatType,
    pub cns_percent: FloatType,
    pub final_tensions: CausalTensor<FloatType>,
    /// The compartment holding the highest ceiling at the end of the bottom phase, and that
    /// ceiling in metres.
    pub controlling: usize,
    pub ceiling_m: FloatType,
    pub deco_stops: Vec<DecoStop>,
    pub safety_stop: Option<DecoStop>,
}

/// One row of the printed dive table: a depth, the no-decompression limit there, and the profile
/// the planner produced for it.
#[derive(Debug, Clone)]
pub struct DiveTableRow {
    pub depth_m: FloatType,
    pub ndl_minutes: FloatType,
    pub profile: DiveProfile,
}

// =============================================================================
// Gas physics
// =============================================================================

/// Ambient pressure at depth, in bar. Ten metres of seawater is one bar.
pub fn ambient_pressure(depth_m: FloatType) -> FloatType {
    ONE + depth_m / TEN
}

/// Inspired nitrogen partial pressure at depth, in bar, breathing air.
pub fn inspired_n2_pp(depth_m: FloatType) -> FloatType {
    (ambient_pressure(depth_m) - P_WATER_VAPOUR) * F_N2
}

/// Oxygen partial pressure at depth, in bar, breathing air.
pub fn oxygen_pp(depth_m: FloatType) -> FloatType {
    ambient_pressure(depth_m) * F_O2
}

/// The Schreiner equation: a compartment's tension after `minutes` at a given inspired pressure.
///
/// `p(t) = p_inspired + (p_initial − p_inspired)·e^{−kt}` with `k = ln2 / half_time`.
pub fn tissue_loading(
    p_initial: FloatType,
    p_inspired: FloatType,
    minutes: FloatType,
    half_time: FloatType,
) -> FloatType {
    let k = Real::ln(TWO) / half_time;
    p_inspired + (p_initial - p_inspired) * Real::exp(-(k * minutes))
}

/// The ascent ceiling a single compartment imposes, in metres: the shallowest depth it tolerates.
///
/// Bühlmann's M-value line gives the tension a compartment tolerates at ambient pressure `P` as
/// `P/b + a`. A gradient factor admits only `gf` of the gap between the ambient pressure and that
/// line, so the tolerated tension at `P` is
///
/// ```text
/// tension = P + gf·(P/b + a − P)
/// ```
///
/// Solving for `P` gives the shallowest ambient pressure this compartment allows, and ten metres
/// of seawater is one bar. A compartment already below its line yields a ceiling at the surface,
/// which the clamp reports as zero.
pub fn tissue_ceiling(tension: FloatType, a: FloatType, b: FloatType, gf: FloatType) -> FloatType {
    let slope = ONE - gf + gf / b;
    let ceiling_pressure = (tension - gf * a) / slope;
    let metres = (ceiling_pressure - ONE) * TEN;
    if metres > ZERO { metres } else { ZERO }
}

/// The maximum exposure at a given oxygen partial pressure, in minutes. The NOAA table starts at
/// 1.0 bar; `None` reports a pressure under that floor, where exposure runs unlimited.
pub fn max_cns_minutes(pp_o2: FloatType) -> Option<FloatType> {
    if pp_o2 < ONE {
        return None;
    }

    for window in CNS_LIMITS.windows(2) {
        let (pp_high, time_high) = window[0];
        let (pp_low, time_low) = window[1];
        if pp_o2 >= pp_low && pp_o2 <= pp_high {
            let ratio = (pp_o2 - pp_low) / (pp_high - pp_low);
            return Some(time_low + ratio * (time_high - time_low));
        }
    }

    // Above the top of the table the tolerated time keeps shrinking in inverse proportion.
    let (top_pp, top_time) = CNS_LIMITS[0];
    if pp_o2 > top_pp {
        return Some(top_time * top_pp / pp_o2);
    }
    Some(CNS_LIMITS[CNS_LIMITS.len() - 1].1)
}

/// The CNS oxygen clock accrued by spending `minutes` at `depth_m`, in percent.
pub fn cns_accumulation(depth_m: FloatType, minutes: FloatType) -> FloatType {
    match max_cns_minutes(oxygen_pp(depth_m)) {
        Some(limit) => minutes / limit * HUNDRED,
        None => ZERO,
    }
}

/// A conservative no-decompression limit for a depth, in minutes.
pub fn estimate_ndl(depth_m: FloatType) -> FloatType {
    for (band_depth, ndl) in NDL_TABLE {
        if depth_m <= band_depth {
            return ndl;
        }
    }
    NDL_BEYOND_TABLE
}

// =============================================================================
// The differentiable gas-loading curve
// =============================================================================

/// The Schreiner gas-loading curve as a scalar-generic field of the four numbers it names.
///
/// # The type holds nothing, and the body names no primitive
///
/// `run` is universally quantified in its scalar: the caller picks `S`, and the tangent functor
/// picks `Dual<S>` behind the caller's back. Anything the body reaches for outside its arguments
/// would have to be materialised at an `S` the type cannot name, which is what drags a stored
/// parameter down to a primitive.
///
/// Taking every quantity as an argument removes the problem entirely. The body below is the
/// Schreiner equation and `ln 2`, which it builds from `S::one()`. No literal, no lift, no
/// primitive. `main` supplies [`FloatType`] values and the tangent functor supplies
/// `Dual<FloatType>` values, from one definition.
pub struct SchreinerCurve;

/// The curve's four inputs, by position.
pub const T_MINUTES: usize = 0;
pub const P_INITIAL: usize = 1;
pub const P_INSPIRED: usize = 2;
pub const HALF_TIME: usize = 3;

impl DifferentiableField<4> for SchreinerCurve {
    fn run<S: Scalar>(&self, at: &[S; 4]) -> S {
        let minutes = at[T_MINUTES];
        let p_initial = at[P_INITIAL];
        let p_inspired = at[P_INSPIRED];
        let half_time = at[HALF_TIME];

        let two = S::one() + S::one();
        let k = Real::ln(two) / half_time;

        p_inspired + (p_initial - p_inspired) * Real::exp(-(k * minutes))
    }
}

impl SchreinerCurve {
    /// The four inputs for one compartment breathing air at `depth_m`, at the working type.
    pub fn inputs_at(depth_m: FloatType, compartment: usize) -> [FloatType; 4] {
        [
            ZERO,
            inspired_n2_pp(ZERO),
            inspired_n2_pp(depth_m),
            half_time_of(compartment),
        ]
    }

    /// The tension and its loading rate `dp/dt`, from one pass over `Dual`.
    ///
    /// The direction seeds time alone, so what comes back is the partial derivative in time with
    /// the two pressures and the half-time held fixed. That is the reading a dive computer shows.
    pub fn value_and_rate(&self, at: &[FloatType; 4]) -> (FloatType, FloatType) {
        let along_time = [ONE, ZERO, ZERO, ZERO];
        (self.run(at), self.directional_derivative(at, &along_time))
    }

    /// The rate constant `k = ln2 / half_time` at the working type, for the analytic check.
    pub fn rate_constant(&self, half_time: FloatType) -> FloatType {
        Real::ln(TWO) / half_time
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// The sixteen half-times as a rank-1 tensor: the right-hand side of the loading zip.
pub fn half_time_tensor() -> Result<CausalTensor<FloatType>, CausalTensorError> {
    CausalTensor::new(HALF_TIMES.to_vec(), vec![COMPARTMENTS])
}

/// The sixteen `(compartment index, a, b)` triples: the right-hand side of the ceiling zip.
///
/// The index rides along in the payload because the reduction that follows the zip has to name
/// the compartment it picked, and a fold sees values rather than positions. The coefficients are
/// already at the working type, so nothing is converted here.
pub fn ceiling_coefficients()
-> Result<CausalTensor<(usize, FloatType, FloatType)>, CausalTensorError> {
    let data: Vec<(usize, FloatType, FloatType)> = (0..COMPARTMENTS)
        .map(|i| (i, A_COEFFICIENTS[i], B_COEFFICIENTS[i]))
        .collect();
    CausalTensor::new(data, vec![COMPARTMENTS])
}

/// A compartment's half-time, in minutes.
pub fn half_time_of(compartment: usize) -> FloatType {
    HALF_TIMES[compartment]
}

/// The saturation of a compartment against the inspired pressure at depth, in percent.
pub fn saturation_percent(tension: FloatType, depth_m: FloatType) -> FloatType {
    tension / inspired_n2_pp(depth_m) * HUNDRED
}

/// One planned dive per depth in the table, each through the same chain of phases.
///
/// Bottom time is the no-decompression limit for that depth, held to a cap so every row runs in
/// the same handful of milliseconds.
pub fn dive_table_rows() -> Result<Vec<DiveTableRow>, CausalityError> {
    TABLE_DEPTHS_M
        .iter()
        .map(|&depth_m| {
            let ndl_minutes = estimate_ndl(depth_m);
            let bottom = if ndl_minutes < TABLE_BOTTOM_CAP_MINUTES {
                ndl_minutes
            } else {
                TABLE_BOTTOM_CAP_MINUTES
            };

            let surface_state =
                DiverState::at_surface().map_err(|e| failed("surface state", &e))?;

            CausalFlow::value(surface_state)
                .try_step(move |diver| descend(diver, depth_m))
                .try_step(move |diver| hold_bottom(diver, depth_m, bottom))
                .try_step(ascend)
                .try_step(move |diver| surface(diver, depth_m, bottom))
                .finish()
                .map(|profile| DiveTableRow {
                    depth_m,
                    ndl_minutes,
                    profile,
                })
        })
        .collect()
}

/// Names the step that failed and carries the underlying reason into the error channel.
pub fn failed(step: &str, cause: &dyn core::fmt::Debug) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(format!("{step}: {cause:?}")))
}
