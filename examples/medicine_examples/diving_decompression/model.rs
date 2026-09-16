/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Model layer for the SCUBA decompression planner: the Bühlmann ZH-L16C constants, the state
//! types, and the physiology.
//!
//! Sixteen tissue compartments absorb and release nitrogen at sixteen different rates, so every
//! compartment computation pairs a tension with the constant that belongs to it. That pairing is
//! what [`ZipTensorWitness::zip_with`] does: it walks two tensors slot by slot and combines each
//! pair. The loading law is therefore written once, and the witness carries it across all sixteen.
//!
//! Every quantity is typed [`FloatType`], so switching the alias in `main` re-runs the whole model
//! at another precision.

use crate::FloatType;
use deep_causality_algebra::Real;
use deep_causality_calculus::{DifferentiableArrow, Scalar};
use deep_causality_core::{CausalFlow, CausalityError, CausalityErrorEnum};
use deep_causality_haft::{Foldable, Semigroupal};
use deep_causality_num::lift;
use deep_causality_tensor::{
    CausalTensor, CausalTensorError, CausalTensorWitness, ZipTensorWitness,
};

// =============================================================================
// Bühlmann ZH-L16C parameters
// =============================================================================

/// The number of tissue compartments the algorithm tracks.
pub const COMPARTMENTS: usize = 16;

/// Compartment half-times for nitrogen, in minutes. The fast compartments drive a short dive; the
/// slow ones control a long one.
pub const HALF_TIMES: [f64; COMPARTMENTS] = [
    5.0, 8.0, 12.5, 18.5, 27.0, 38.3, 54.3, 77.0, 109.0, 146.0, 187.0, 239.0, 305.0, 390.0, 498.0,
    635.0,
];

/// M-value `a` coefficients, in bar.
const A_COEFFICIENTS: [f64; COMPARTMENTS] = [
    1.1696, 1.0000, 0.8618, 0.7562, 0.6200, 0.5043, 0.4410, 0.4000, 0.3750, 0.3500, 0.3295, 0.3065,
    0.2835, 0.2610, 0.2480, 0.2327,
];

/// M-value `b` coefficients, dimensionless.
const B_COEFFICIENTS: [f64; COMPARTMENTS] = [
    0.5578, 0.6514, 0.7222, 0.7825, 0.8126, 0.8434, 0.8693, 0.8910, 0.9092, 0.9222, 0.9319, 0.9403,
    0.9477, 0.9544, 0.9602, 0.9653,
];

/// Nitrogen partial pressure at the surface, in bar.
const SURFACE_N2_PP: f64 = 0.79;
/// Nitrogen fraction in air.
const F_N2: f64 = 0.79;
/// Oxygen fraction in air.
const F_O2: f64 = 0.21;
/// Water-vapour pressure in the lung at 37 °C, in bar.
const P_WATER_VAPOUR: f64 = 0.0627;

/// Gradient factors for conservative recreational diving.
pub const GF_LOW: f64 = 0.30;
pub const GF_HIGH: f64 = 0.85;

/// Descent and ascent rates, in metres per minute. The ascent rate is the PADI standard.
pub const DESCENT_RATE: f64 = 18.0;
pub const ASCENT_RATE: f64 = 9.0;

/// NOAA CNS oxygen-toxicity limits as `(ppO2 in bar, maximum exposure in minutes)`.
const CNS_LIMITS: [(f64, f64); 7] = [
    (1.60, 45.0),
    (1.50, 120.0),
    (1.40, 150.0),
    (1.30, 180.0),
    (1.20, 210.0),
    (1.10, 240.0),
    (1.00, 300.0),
];

/// Ascent proceeds in steps of this many metres.
pub const ASCENT_STEP_M: f64 = 3.0;
/// Above this depth the ascent runs to the surface, and the safety stop covers the last stretch.
pub const DECO_CLEARANCE_M: f64 = 6.0;
/// A decompression stop lasts at least this long, in minutes.
pub const MIN_STOP_MINUTES: f64 = 2.0;
/// Dives to at least this depth carry a safety stop.
pub const SAFETY_STOP_DEPTH_THRESHOLD_M: f64 = 15.0;
/// The safety stop itself: depth in metres, duration in minutes.
pub const SAFETY_STOP_M: f64 = 5.0;
pub const SAFETY_STOP_MINUTES: f64 = 3.0;

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
        let zero = lift::<FloatType>(0.0);
        Ok(Self {
            depth_m: zero,
            elapsed_minutes: zero,
            tissue_tensions: CausalTensor::new(
                vec![lift::<FloatType>(SURFACE_N2_PP); COMPARTMENTS],
                vec![COMPARTMENTS],
            )?,
            cns_percent: zero,
            deco_stops: Vec::new(),
            controlling_at_bottom: 0,
            ceiling_at_bottom_m: zero,
        })
    }

    /// Spends `minutes` at `depth_m`: loads the sixteen compartments, advances the CNS oxygen
    /// clock and the elapsed time, and leaves the diver at that depth.
    ///
    /// Every phase of the dive is this one operation applied at a different depth for a different
    /// duration, so the three phases below read as the schedule they describe.
    pub fn advance(
        &self,
        depth_m: FloatType,
        minutes: FloatType,
    ) -> Result<Self, CausalTensorError> {
        Ok(Self {
            depth_m,
            elapsed_minutes: self.elapsed_minutes + minutes,
            tissue_tensions: update_tissues(&self.tissue_tensions, depth_m, minutes)?,
            cns_percent: self.cns_percent + cns_accumulation(depth_m, minutes),
            deco_stops: self.deco_stops.clone(),
            controlling_at_bottom: self.controlling_at_bottom,
            ceiling_at_bottom_m: self.ceiling_at_bottom_m,
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

/// The depths the printed table covers, in metres.
pub const TABLE_DEPTHS_M: [f64; 9] = [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0];

/// The bottom time the table uses for each depth: the no-decompression limit, held to this cap so
/// every row runs in the same handful of milliseconds.
pub const TABLE_BOTTOM_CAP_MINUTES: f64 = 20.0;

// =============================================================================
// Gas physics
// =============================================================================

/// Ambient pressure at depth, in bar.
///
/// The three gas laws here are written over the `Scalar` bound, because two callers need them at
/// two different scalars: the simulation evaluates them at [`FloatType`], and the differentiable
/// loading curve evaluates them at `f64` to fix its configuration. One definition serves both.
pub fn ambient_pressure<S: Scalar>(depth_m: S) -> S {
    lift::<S>(1.0) + depth_m / lift::<S>(10.0)
}

/// Inspired nitrogen partial pressure at depth, in bar.
pub fn inspired_n2_pp<S: Scalar>(depth_m: S) -> S {
    (ambient_pressure(depth_m) - lift::<S>(P_WATER_VAPOUR)) * lift::<S>(F_N2)
}

/// Oxygen partial pressure at depth, in bar.
pub fn oxygen_pp<S: Scalar>(depth_m: S) -> S {
    ambient_pressure(depth_m) * lift::<S>(F_O2)
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
    let k = Real::ln(lift::<FloatType>(2.0)) / half_time;
    p_inspired + (p_initial - p_inspired) * Real::exp(-(k * minutes))
}

/// The ascent ceiling a single compartment imposes, in metres: the shallowest depth it tolerates.
pub fn tissue_ceiling(tension: FloatType, a: FloatType, b: FloatType, gf: FloatType) -> FloatType {
    let m_value = tension / b + a;
    let allowed_gradient = gf * (m_value - tension / b);
    let ceiling_pressure = tension - allowed_gradient;
    let metres = (ceiling_pressure - lift::<FloatType>(1.0)) * lift::<FloatType>(10.0);
    let zero = lift::<FloatType>(0.0);
    if metres > zero { metres } else { zero }
}

/// Loads all sixteen compartments for `minutes` spent at `depth_m`.
///
/// `zip_with` pairs each tension with its own half-time and applies the loading law to the pair,
/// which is the whole of the update.
pub fn update_tissues(
    tensions: &CausalTensor<FloatType>,
    depth_m: FloatType,
    minutes: FloatType,
) -> Result<CausalTensor<FloatType>, CausalTensorError> {
    let p_inspired = inspired_n2_pp(depth_m);
    let half_times = constant_tensor(&HALF_TIMES)?;

    Ok(ZipTensorWitness::zip_with(
        tensions.clone(),
        half_times,
        |p_initial, half_time| tissue_loading(p_initial, p_inspired, minutes, half_time),
    ))
}

/// The controlling compartment and the ceiling it imposes, in metres.
///
/// Two categorical steps carry this. `zip_with` turns a tension and its M-value coefficients into
/// that compartment's ceiling, and `fold` reduces the sixteen ceilings to the highest one. The
/// index rides along in the payload so the reduction can name the compartment it picked.
pub fn find_ceiling(
    tensions: &CausalTensor<FloatType>,
    gf: FloatType,
) -> Result<(usize, FloatType), CausalTensorError> {
    let coefficients = coefficient_tensor()?;

    let ceilings = ZipTensorWitness::zip_with(tensions.clone(), coefficients, |tension, ab| {
        let (index, a, b) = ab;
        (index, tissue_ceiling(tension, a, b, gf))
    });

    Ok(CausalTensorWitness::fold(
        ceilings,
        (0usize, lift::<FloatType>(0.0)),
        |best, candidate| {
            if candidate.1 > best.1 {
                candidate
            } else {
                best
            }
        },
    ))
}

/// The maximum exposure at a given oxygen partial pressure, in minutes. The NOAA table starts at
/// 1.0 bar; `None` reports a pressure under that floor, where exposure runs unlimited.
pub fn max_cns_minutes(pp_o2: FloatType) -> Option<FloatType> {
    let one = lift::<FloatType>(1.0);
    if pp_o2 < one {
        return None;
    }

    for window in CNS_LIMITS.windows(2) {
        let (pp_high, time_high) = (
            lift::<FloatType>(window[0].0),
            lift::<FloatType>(window[0].1),
        );
        let (pp_low, time_low) = (
            lift::<FloatType>(window[1].0),
            lift::<FloatType>(window[1].1),
        );
        if pp_o2 >= pp_low && pp_o2 <= pp_high {
            let ratio = (pp_o2 - pp_low) / (pp_high - pp_low);
            return Some(time_low + ratio * (time_high - time_low));
        }
    }

    // Above the top of the table the tolerated time keeps shrinking in inverse proportion.
    let top_pp = lift::<FloatType>(CNS_LIMITS[0].0);
    let top_time = lift::<FloatType>(CNS_LIMITS[0].1);
    if pp_o2 > top_pp {
        return Some(top_time * top_pp / pp_o2);
    }
    Some(lift::<FloatType>(CNS_LIMITS[CNS_LIMITS.len() - 1].1))
}

/// The CNS oxygen clock accrued by spending `minutes` at `depth_m`, in percent.
pub fn cns_accumulation(depth_m: FloatType, minutes: FloatType) -> FloatType {
    match max_cns_minutes(oxygen_pp(depth_m)) {
        Some(limit) => minutes / limit * lift::<FloatType>(100.0),
        None => lift::<FloatType>(0.0),
    }
}

/// A conservative no-decompression limit for a depth, in minutes.
pub fn estimate_ndl(depth_m: FloatType) -> FloatType {
    let table = [
        (12.0, 200.0),
        (18.0, 80.0),
        (24.0, 45.0),
        (30.0, 25.0),
        (36.0, 15.0),
        (42.0, 10.0),
        (48.0, 8.0),
    ];
    for (max_depth, ndl) in table {
        if depth_m <= lift::<FloatType>(max_depth) {
            return lift::<FloatType>(ndl);
        }
    }
    lift::<FloatType>(6.0)
}

// =============================================================================
// The differentiable gas-loading curve
// =============================================================================

/// The Schreiner curve as a scalar-generic arrow, so the tangent functor differentiates it.
///
/// `run` is written once over the `Scalar` bound. Evaluated at [`FloatType`] it returns the
/// tension; evaluated at `Dual` it returns the tension together with the loading rate `dp/dt`,
/// which is the quantity a dive computer watches. The rate equals the analytic
/// `k·(p_inspired − p(t))`, and `main` prints both side by side.
pub struct SchreinerLoading {
    /// The curve's parameters, held as `f64`, the widest form a source file carries. `run` lifts
    /// them into whatever scalar it is asked for, so nothing narrows on the way to `Dual`.
    pub p_initial: f64,
    pub p_inspired: f64,
    pub half_time: f64,
}

impl SchreinerLoading {
    /// The loading curve for one compartment breathing air at `depth_m`, starting from a diver
    /// equilibrated at the surface.
    pub fn at_depth(depth_m: f64, compartment: usize) -> Self {
        Self {
            p_initial: inspired_n2_pp::<f64>(0.0),
            p_inspired: inspired_n2_pp::<f64>(depth_m),
            half_time: HALF_TIMES[compartment],
        }
    }

    /// The rate constant `k = ln2 / half_time`, at the scalar the caller asks for.
    pub fn rate_constant<S: Scalar>(&self) -> S {
        lift::<S>(2.0_f64.ln()) / lift::<S>(self.half_time)
    }
}

impl DifferentiableArrow for SchreinerLoading {
    fn run<S: Scalar>(&self, t: S) -> S {
        let k = self.rate_constant::<S>();
        let p_initial = lift::<S>(self.p_initial);
        let p_inspired = lift::<S>(self.p_inspired);

        p_inspired + (p_initial - p_inspired) * (-(k * t)).exp()
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// A rank-1 tensor holding a table of `f64` constants lifted into the working scalar.
fn constant_tensor(
    values: &[f64; COMPARTMENTS],
) -> Result<CausalTensor<FloatType>, CausalTensorError> {
    let data: Vec<FloatType> = values.iter().map(|&v| lift::<FloatType>(v)).collect();
    CausalTensor::new(data, vec![COMPARTMENTS])
}

/// A rank-1 tensor of `(compartment index, a, b)`, the payload `find_ceiling` zips against.
fn coefficient_tensor() -> Result<CausalTensor<(usize, FloatType, FloatType)>, CausalTensorError> {
    let data: Vec<(usize, FloatType, FloatType)> = (0..COMPARTMENTS)
        .map(|i| {
            (
                i,
                lift::<FloatType>(A_COEFFICIENTS[i]),
                lift::<FloatType>(B_COEFFICIENTS[i]),
            )
        })
        .collect();
    CausalTensor::new(data, vec![COMPARTMENTS])
}

/// A compartment half-time as the working scalar, for display.
pub fn half_time_of(compartment: usize) -> FloatType {
    lift::<FloatType>(HALF_TIMES[compartment])
}

/// The saturation of a compartment against the inspired pressure at depth, in percent.
pub fn saturation_percent(tension: FloatType, depth_m: FloatType) -> FloatType {
    tension / inspired_n2_pp(depth_m) * lift::<FloatType>(100.0)
}

// =============================================================================
// The dive as a chain of phases
// =============================================================================

/// Plans one dive as a chain of phases.
///
/// Each phase takes the diver state and returns the next one, and `try_step` sequences them. Every
/// phase loads sixteen compartments through a fallible tensor construction, so a failure anywhere
/// leaves the chain in the error channel and `finish` reports it.
pub fn plan_dive(
    max_depth_m: FloatType,
    bottom_minutes: FloatType,
) -> Result<DiveProfile, CausalityError> {
    let surface = DiverState::at_surface().map_err(|e| failed("surface state", &e))?;

    CausalFlow::value(surface)
        .try_step(move |state| descend(state, max_depth_m))
        .try_step(move |state| hold_bottom(state, max_depth_m, bottom_minutes))
        .try_step(ascend)
        .try_step(move |state| summarise(state, max_depth_m, bottom_minutes))
        .finish()
}

/// Phase 1. Descent loads the tissues at the average depth passed through on the way down.
fn descend(state: DiverState, max_depth_m: FloatType) -> Result<DiverState, CausalityError> {
    let descent_minutes = max_depth_m / lift::<FloatType>(DESCENT_RATE);
    let average_depth = max_depth_m / lift::<FloatType>(2.0);

    let mut next = state
        .advance(average_depth, descent_minutes)
        .map_err(|e| failed("descent", &e))?;
    next.depth_m = max_depth_m;
    Ok(next)
}

/// Phase 2. The bottom phase holds depth, which is where tissue loading peaks. The ceiling read
/// here is the one the dive plan quotes.
fn hold_bottom(
    state: DiverState,
    max_depth_m: FloatType,
    bottom_minutes: FloatType,
) -> Result<DiverState, CausalityError> {
    let mut next = state
        .advance(max_depth_m, bottom_minutes)
        .map_err(|e| failed("bottom phase", &e))?;

    let (controlling, ceiling) = find_ceiling(&next.tissue_tensions, lift::<FloatType>(GF_HIGH))
        .map_err(|e| failed("bottom ceiling", &e))?;
    next.controlling_at_bottom = controlling;
    next.ceiling_at_bottom_m = ceiling;
    Ok(next)
}

/// Phase 3. Ascent proceeds in three-metre steps. Before each step the controlling compartment's
/// ceiling is read, and a step that would breach it becomes a decompression stop at the current
/// depth. A dive shallower than the safety-stop threshold surfaces directly.
fn ascend(state: DiverState) -> Result<DiverState, CausalityError> {
    let zero = lift::<FloatType>(0.0);
    let step = lift::<FloatType>(ASCENT_STEP_M);
    let clearance = lift::<FloatType>(DECO_CLEARANCE_M);
    let gf_high = lift::<FloatType>(GF_HIGH);

    let mut current = state;
    let mut depth = current.depth_m;

    while depth > zero {
        let (_, ceiling) = find_ceiling(&current.tissue_tensions, gf_high)
            .map_err(|e| failed("ascent ceiling", &e))?;

        let next_depth = if depth > step { depth - step } else { zero };

        // A stop is required when the ceiling sits deeper than where the next step would land.
        if ceiling > next_depth && depth > clearance {
            let stop = DecoStop {
                depth_m: depth,
                minutes: lift::<FloatType>(MIN_STOP_MINUTES),
            };
            current = current
                .advance(stop.depth_m, stop.minutes)
                .map_err(|e| failed("deco stop", &e))?;
            current.deco_stops.push(stop);
        }

        let segment = depth - next_depth;
        let segment_minutes = segment / lift::<FloatType>(ASCENT_RATE);
        let average_depth = next_depth + segment / lift::<FloatType>(2.0);

        current = current
            .advance(average_depth, segment_minutes)
            .map_err(|e| failed("ascent segment", &e))?;
        depth = next_depth;
    }

    current.depth_m = zero;
    Ok(current)
}

/// Phase 4. The safety stop, then the finished profile.
fn summarise(
    state: DiverState,
    max_depth_m: FloatType,
    bottom_minutes: FloatType,
) -> Result<DiveProfile, CausalityError> {
    let safety_stop = if max_depth_m >= lift::<FloatType>(SAFETY_STOP_DEPTH_THRESHOLD_M) {
        Some(DecoStop {
            depth_m: lift::<FloatType>(SAFETY_STOP_M),
            minutes: lift::<FloatType>(SAFETY_STOP_MINUTES),
        })
    } else {
        None
    };

    let final_state = match safety_stop {
        Some(stop) => state
            .advance(stop.depth_m, stop.minutes)
            .map_err(|e| failed("safety stop", &e))?,
        None => state.clone(),
    };

    Ok(DiveProfile {
        max_depth_m,
        bottom_minutes,
        total_minutes: final_state.elapsed_minutes,
        cns_percent: final_state.cns_percent,
        final_tensions: final_state.tissue_tensions,
        controlling: state.controlling_at_bottom,
        ceiling_m: state.ceiling_at_bottom_m,
        deco_stops: state.deco_stops,
        safety_stop,
    })
}

/// Names the phase that failed and carries the underlying reason into the error channel.
fn failed(phase: &str, cause: &dyn core::fmt::Debug) -> CausalityError {
    CausalityError::new(CausalityErrorEnum::Custom(format!("{phase}: {cause:?}")))
}

/// One planned dive per depth in [`TABLE_DEPTHS_M`], for the printed table.
///
/// Bottom time is the no-decompression limit for that depth, held to a cap so every row runs in
/// the same handful of milliseconds.
pub fn dive_table_rows() -> Result<Vec<DiveTableRow>, CausalityError> {
    let cap = lift::<FloatType>(TABLE_BOTTOM_CAP_MINUTES);

    TABLE_DEPTHS_M
        .iter()
        .map(|&depth| {
            let depth_m = lift::<FloatType>(depth);
            let ndl_minutes = estimate_ndl(depth_m);
            let bottom = if ndl_minutes < cap { ndl_minutes } else { cap };
            plan_dive(depth_m, bottom).map(|profile| DiveTableRow {
                depth_m,
                ndl_minutes,
                profile,
            })
        })
        .collect()
}
