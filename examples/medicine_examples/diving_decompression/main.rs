/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCUBA decompression planner
//!
//! A dive plan answers one question: how does a diver surface while the dissolved nitrogen stays
//! in solution? The Bühlmann ZH-L16C algorithm answers it by tracking sixteen tissue compartments,
//! each absorbing and releasing nitrogen at its own rate, and holding the ascent whenever the
//! fastest-loaded compartment reaches its tolerance.
//!
//! Four categorical operations carry the program, and all four are in this file:
//!
//! ```text
//! try_step   diver state → the next one     the dive as a chain of phases
//! zip_with   tension × half-time → tension  sixteen compartments, one loading law
//! zip_with   tension × M-values  → ceiling  sixteen compartments, one ceiling law
//! fold       sixteen ceilings → the binding one
//! ```
//!
//! Each compartment carries its own constants, so every compartment computation is a **pairing**:
//! a tension against the half-time that governs it, or against the M-value coefficients that bound
//! it. `ZipTensorWitness::zip_with` walks two tensors slot by slot and combines each pair, so the
//! laws in `model` are written once for one compartment and the witness applies them to all
//! sixteen. No compartment index appears in either law.
//!
//! The dive itself is a sequence: descend, hold, ascend to the safety stop, surface, each phase
//! taking the diver state and returning the next. `CausalFlow::try_step` sequences them and routes
//! any failure to the error channel, so the phases below hold physiology and no error plumbing.
//!
//! The fifth abstraction is the tangent functor. The gas-loading rate `dp/dt` is what a dive
//! computer watches, and it comes from evaluating the loading curve over `Dual`: `model` states
//! the curve once, and the derivative follows from the type.

mod model;
mod utils_print;

use deep_causality_core::{CausalFlow, CausalityError};
use deep_causality_haft::{Foldable, Semigroupal};
use deep_causality_num::{Float106, const_scalar_from_int};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness};
use model::{
    ASCENT_RATE, ASCENT_STEP_M, DECO_CLEARANCE_M, DESCENT_RATE, DecoStop, DiveProfile, DiverState,
    GF_HIGH, MAX_STOP_MINUTES, MIN_STOP_MINUTES, SchreinerCurve, T_MINUTES, TWO, ZERO,
    ascent_floor, ceiling_coefficients, cns_accumulation, dive_table_rows, failed,
    half_time_tensor, inspired_n2_pp, safety_stop_for, tissue_ceiling, tissue_loading,
};
use utils_print::{print_dive_table, print_gas_loading_rate, print_header, print_simulation};

/// The dive this run plans: maximum depth in metres, time held at that depth in minutes.
///
/// Fifty minutes at thirty metres is twice the no-decompression limit for that depth, so the
/// ascent carries a real obligation and the planner has a schedule to produce. The table above it
/// plans each depth *at* its limit, where the correct answer is no mandatory stop at all.
const MAX_DEPTH_M: FloatType = const_scalar_from_int!(FloatType, 30);
const BOTTOM_MINUTES: FloatType = const_scalar_from_int!(FloatType, 50);

/// Where the gas-loading rate is sampled: compartment index, and the elapsed time in minutes.
const RATE_SAMPLE_COMPARTMENT: usize = 0;
const RATE_SAMPLE_MINUTES: FloatType = const_scalar_from_int!(FloatType, 10);

/// The working scalar. Switch it to `f32`, `f64` or `deep_causality_num::BFloat16`; the constants,
/// the tissue tensions, the ceilings, the CNS clock and the autodiff rate all recompute at that
/// precision.
pub type FloatType = Float106;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // Every row of the table is its own dive, planned by the same chain of phases below.
    print_dive_table(&dive_table_rows()?);

    // ── The causal monad ────────────────────────────────────────────────────────────────────
    // Four phases, each taking the diver state and returning the next. `try_step` sequences them
    // and short-circuits to the error channel, so no phase carries error plumbing of its own.
    let profile = CausalFlow::value(DiverState::at_surface()?)
        .try_step(|diver| descend(diver, MAX_DEPTH_M))
        .try_step(|diver| hold_bottom(diver, MAX_DEPTH_M, BOTTOM_MINUTES))
        .try_step(|diver| ascend(diver, ascent_floor(MAX_DEPTH_M)))
        .try_step(|diver| surface(diver, MAX_DEPTH_M, BOTTOM_MINUTES))
        .finish()?;

    print_simulation(&profile);

    // ── The tangent functor ─────────────────────────────────────────────────────────────────
    // One evaluation over `Dual` returns the tension and the rate at which it is loading. The
    // analytic rate k·(p_inspired − p) is printed beside it as a check.
    let mut at = SchreinerCurve::inputs_at(MAX_DEPTH_M, RATE_SAMPLE_COMPARTMENT);
    at[T_MINUTES] = RATE_SAMPLE_MINUTES;
    let (tension, rate) = SchreinerCurve.value_and_rate(&at);
    print_gas_loading_rate(&at, tension, rate);

    Ok(())
}

// ============================================================================================
// The two compartment-wide operations. Both are pairings, and both are one `zip_with`.
// ============================================================================================

/// Loads all sixteen compartments for `minutes` spent at `depth_m`.
///
/// `zip_with` pairs each tension with its own half-time and applies the Schreiner law to the pair.
/// The law is stated once, for one compartment, in `model::tissue_loading`; the witness carries it
/// across all sixteen and no compartment index is written anywhere.
fn load_compartments(
    tensions: &CausalTensor<FloatType>,
    depth_m: FloatType,
    minutes: FloatType,
) -> Result<CausalTensor<FloatType>, CausalityError> {
    let p_inspired = inspired_n2_pp(depth_m);
    let half_times = half_time_tensor().map_err(|e| failed("half-time table", &e))?;

    Ok(ZipTensorWitness::zip_with(
        tensions.clone(),
        half_times,
        |tension, half_time| tissue_loading(tension, p_inspired, minutes, half_time),
    ))
}

/// The compartment governing the ascent, and the ceiling it imposes in metres.
///
/// Two steps and no loop. `zip_with` pairs each tension with the M-value coefficients that bound
/// it and returns that compartment's ceiling; `fold` reduces the sixteen ceilings to the highest,
/// which is the shallowest depth the diver may go. The index rides along in the payload so the
/// reduction can name the compartment it picked.
fn governing_compartment(
    tensions: &CausalTensor<FloatType>,
    gf: FloatType,
) -> Result<(usize, FloatType), CausalityError> {
    let coefficients = ceiling_coefficients().map_err(|e| failed("M-value table", &e))?;

    let ceilings =
        ZipTensorWitness::zip_with(tensions.clone(), coefficients, |tension, (index, a, b)| {
            (index, tissue_ceiling(tension, a, b, gf))
        });

    Ok(CausalTensorWitness::fold(
        ceilings,
        (0usize, ZERO),
        |binding, candidate| {
            if candidate.1 > binding.1 {
                candidate
            } else {
                binding
            }
        },
    ))
}

// ============================================================================================
// The four phases of the dive
// ============================================================================================

/// Phase 1. Descent loads the tissues at the average depth passed through on the way down.
fn descend(diver: DiverState, max_depth_m: FloatType) -> Result<DiverState, CausalityError> {
    let minutes = max_depth_m / DESCENT_RATE;
    let average_depth = max_depth_m / TWO;

    let mut next = spend(diver, average_depth, minutes)?;
    next.depth_m = max_depth_m;
    Ok(next)
}

/// Phase 2. The bottom phase holds depth, which is where tissue loading peaks. The ceiling read
/// here is the one a dive plan quotes.
fn hold_bottom(
    diver: DiverState,
    max_depth_m: FloatType,
    bottom_minutes: FloatType,
) -> Result<DiverState, CausalityError> {
    let mut next = spend(diver, max_depth_m, bottom_minutes)?;

    let (controlling, ceiling) = governing_compartment(&next.tissue_tensions, GF_HIGH)?;
    next.controlling_at_bottom = controlling;
    next.ceiling_at_bottom_m = ceiling;
    Ok(next)
}

/// Phase 3. The staged ascent, in three-metre steps down to `floor_m`: the safety-stop depth when
/// the dive carries one, the surface otherwise.
///
/// Before each step the governing compartment's ceiling is read. A step that would breach it
/// becomes a decompression stop at the current depth, and the stop is held, one
/// [`MIN_STOP_MINUTES`] at a time, until the ceiling has risen past the depth the step lands on.
/// The stop is recorded once, with the minutes it took.
fn ascend(diver: DiverState, floor_m: FloatType) -> Result<DiverState, CausalityError> {
    let mut current = diver;
    let mut depth = current.depth_m;

    while depth > floor_m {
        let next_depth = if depth - ASCENT_STEP_M > floor_m {
            depth - ASCENT_STEP_M
        } else {
            floor_m
        };

        // A stop is required while the ceiling sits deeper than where the next step would land.
        // Above the clearance depth the ascent runs on and the safety stop covers the rest.
        let mut stop_minutes = ZERO;
        if depth > DECO_CLEARANCE_M {
            loop {
                let (_, ceiling) = governing_compartment(&current.tissue_tensions, GF_HIGH)?;
                if ceiling <= next_depth {
                    break;
                }
                if stop_minutes >= MAX_STOP_MINUTES {
                    return Err(failed(
                        "decompression stop",
                        &format!(
                            "the ceiling holds at {ceiling:?} m after {stop_minutes:?} min at \
                             {depth:?} m, so the ascent to {next_depth:?} m stays closed"
                        ),
                    ));
                }
                current = spend(current, depth, MIN_STOP_MINUTES)?;
                stop_minutes += MIN_STOP_MINUTES;
            }
        }
        if stop_minutes > ZERO {
            current.deco_stops.push(DecoStop {
                depth_m: depth,
                minutes: stop_minutes,
            });
        }

        current = ascend_segment(current, depth, next_depth)?;
        depth = next_depth;
    }

    Ok(current)
}

/// Phase 4. The safety stop when the dive carries one, the last stretch to the surface, and the
/// finished profile.
///
/// The profile carries the tissue state the diver surfaces with, so the phase confirms the ascent
/// finished at the surface before it reports.
fn surface(
    diver: DiverState,
    max_depth_m: FloatType,
    bottom_minutes: FloatType,
) -> Result<DiveProfile, CausalityError> {
    let safety_stop = safety_stop_for(max_depth_m);

    let held = match safety_stop {
        Some(stop) => spend(diver.clone(), stop.depth_m, stop.minutes)?,
        None => diver.clone(),
    };
    let from_m = held.depth_m;
    let surfaced = ascend_segment(held, from_m, ZERO)?;

    if surfaced.depth_m != ZERO {
        return Err(failed(
            "surfacing",
            &format!("the ascent ended at {:?} m", surfaced.depth_m),
        ));
    }

    Ok(DiveProfile {
        max_depth_m,
        bottom_minutes,
        total_minutes: surfaced.elapsed_minutes,
        cns_percent: surfaced.cns_percent,
        final_tensions: surfaced.tissue_tensions,
        controlling: diver.controlling_at_bottom,
        ceiling_m: diver.ceiling_at_bottom_m,
        deco_stops: diver.deco_stops,
        safety_stop,
    })
}

/// One ascent segment from `from_m` up to `to_m` at the ascent rate. The tissues load at the
/// segment's average depth, and the diver lands at `to_m`.
fn ascend_segment(
    diver: DiverState,
    from_m: FloatType,
    to_m: FloatType,
) -> Result<DiverState, CausalityError> {
    let segment = from_m - to_m;
    let minutes = segment / ASCENT_RATE;
    let average_depth = to_m + segment / TWO;

    let mut next = spend(diver, average_depth, minutes)?;
    next.depth_m = to_m;
    Ok(next)
}

/// Spends `minutes` at `depth_m`: loads the sixteen compartments, advances the CNS oxygen clock
/// and the elapsed time, and leaves the diver at that depth.
///
/// Every phase above is this one operation applied at a different depth for a different duration,
/// so each phase reads as the schedule it describes.
fn spend(
    diver: DiverState,
    depth_m: FloatType,
    minutes: FloatType,
) -> Result<DiverState, CausalityError> {
    Ok(DiverState {
        depth_m,
        elapsed_minutes: diver.elapsed_minutes + minutes,
        tissue_tensions: load_compartments(&diver.tissue_tensions, depth_m, minutes)?,
        cns_percent: diver.cns_percent + cns_accumulation(depth_m, minutes),
        ..diver
    })
}
