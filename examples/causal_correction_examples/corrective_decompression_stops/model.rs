/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tissue dynamics stage and corrective driver loops.

use crate::model_types::{
    ASCENT_RATE, DCS_RATIO_THRESHOLD, DiveContext, DiveProcess, DiveState, FloatType, HALF_TIME,
    SAFETY_RATIO_THRESHOLD, STARTING_DEPTH_M, TICK_MINUTES, ambient_pressure, inspired_n2_pp, read,
};
use deep_causality_core::{CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the *ascent command*
/// for this tick (metres to ascend). The stage applies the command,
/// equilibrates the tissue compartment toward the inspired N2 partial
/// pressure at the new depth, and updates the supersaturation ratio.
/// By default the carrier value is reset to the planned ascent per tick at
/// the end of the stage. Interventions overwrite this to insert a stop.
/// The plan and the thresholds are read from the dive context the process
/// carries. A missing ascent command, context or dive fact ends the process
/// in error with the state unchanged.
pub fn simulate_step(
    value: CausalEffect<FloatType>,
    mut state: DiveState,
    ctx: Option<DiveContext>,
) -> DiveProcess<FloatType> {
    match advance(value, &mut state, ctx.as_ref()) {
        // Carry the planned ascent forward. The closed-loop driver
        // overwrites this with 0.0 whenever a stop is needed.
        Ok((planned_ascent, logs)) => {
            DiveProcess::<FloatType>::new(Ok(CausalEffect::value(planned_ascent)), state, ctx, logs)
        }
        Err(err) => DiveProcess::<FloatType>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// The tick itself: reads every input first, then advances `state` and
/// returns the planned ascent for the next tick with this tick's log entry.
fn advance(
    value: CausalEffect<FloatType>,
    state: &mut DiveState,
    ctx: Option<&DiveContext>,
) -> Result<(FloatType, EffectLog), CausalityError> {
    let commanded_ascent = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)?;
    let dive = ctx.ok_or_else(CausalityError::MissingContext)?;
    let planned_ascent = planned_ascent_per_tick(dive)?;
    let half_time = read(dive, HALF_TIME)?;
    let dcs_ratio_threshold = read(dive, DCS_RATIO_THRESHOLD)?;
    let safety_ratio_threshold = read(dive, SAFETY_RATIO_THRESHOLD)?;

    state.depth_m = (state.depth_m - commanded_ascent).max(0.0);
    let ambient = ambient_pressure(state.depth_m);
    let p_insp_n2 = inspired_n2_pp(state.depth_m);
    let k = 1.0 - (-TICK_MINUTES / half_time).exp();
    state.tissue_n2_bar += (p_insp_n2 - state.tissue_n2_bar) * k;

    let ratio = state.tissue_n2_bar / ambient;
    state.last_ratio = ratio;
    if ratio > state.max_ratio_observed {
        state.max_ratio_observed = ratio;
    }
    state.tick += 1;
    state.depth_trajectory.push(state.depth_m);
    state.tissue_trajectory.push(state.tissue_n2_bar);
    state.ratio_trajectory.push(ratio);
    if state.dcs_at.is_none() && ratio > dcs_ratio_threshold {
        state.dcs_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.dcs_at == Some(state.tick) {
        " [DCS RISK]"
    } else if ratio > safety_ratio_threshold {
        " [supersaturated]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2}: depth = {:>4.1} m, tissue = {:>4.2} bar, ratio = {:>4.2}{}",
        state.tick, state.depth_m, state.tissue_n2_bar, ratio, marker
    ));

    Ok((planned_ascent, logs))
}

/// Metres of ascent per tick under the continuous-ascent plan: the plan's
/// ascent rate over one tick.
fn planned_ascent_per_tick(dive: &DiveContext) -> Result<FloatType, CausalityError> {
    Ok(read(dive, ASCENT_RATE)? * TICK_MINUTES)
}

/// Initial process at the bottom, tissue fully saturated to the bottom
/// depth, carrying `dive` as its context. A missing ascent rate starts the
/// process in error.
pub fn initial_process(dive: DiveContext) -> DiveProcess<FloatType> {
    let state = DiveState {
        depth_m: STARTING_DEPTH_M,
        tissue_n2_bar: inspired_n2_pp(STARTING_DEPTH_M),
        ..Default::default()
    };
    DiveProcess::<FloatType>::new(
        planned_ascent_per_tick(&dive).map(CausalEffect::value),
        state,
        Some(dive),
        EffectLog::new(),
    )
}
