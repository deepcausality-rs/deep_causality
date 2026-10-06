/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Simulation stage and corrective driver loop for the lane-keeping example.

use crate::model_types::{
    ANOMALY_THRESHOLD, FloatType, LANE_HALF_WIDTH, LaneContext, LaneProcess, P_GAIN, VehicleState,
    drift_at, read,
};
use deep_causality_core::{AlternatableValue, CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the current lateral
/// offset; the stage adds the per-tick drift, advances `state.tick`,
/// appends the new offset to `state.trajectory`, and records whether the
/// vehicle has now left the lane. The drift coefficients, the lane
/// half-width and the anomaly threshold are read from the lane context the
/// process carries. A missing offset, context or lane fact ends the process
/// in error with the state unchanged.
pub fn simulate_step(
    value: CausalEffect<FloatType>,
    mut state: VehicleState,
    ctx: Option<LaneContext>,
) -> LaneProcess<FloatType> {
    match advance(value, &mut state, ctx.as_ref()) {
        Ok((next_offset, logs)) => {
            LaneProcess::<FloatType>::new(Ok(CausalEffect::value(next_offset)), state, ctx, logs)
        }
        Err(err) => LaneProcess::<FloatType>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// The tick itself: reads every input first, then advances `state` and
/// returns the new offset with its log entry.
fn advance(
    value: CausalEffect<FloatType>,
    state: &mut VehicleState,
    ctx: Option<&LaneContext>,
) -> Result<(FloatType, EffectLog), CausalityError> {
    let prev_offset = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)?;
    let lane = ctx.ok_or_else(CausalityError::MissingContext)?;
    let drift = drift_at(state.tick, lane)?;
    let lane_half_width = read(lane, LANE_HALF_WIDTH)?;
    let anomaly_threshold = read(lane, ANOMALY_THRESHOLD)?;

    let next_offset = prev_offset + drift;
    state.tick += 1;
    state.trajectory.push(next_offset);
    if next_offset.abs() > state.max_offset_observed.abs() {
        state.max_offset_observed = next_offset;
    }
    if state.catastrophic_at.is_none() && next_offset.abs() > lane_half_width {
        state.catastrophic_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.catastrophic_at.is_some() {
        " [OFF-ROAD]"
    } else if next_offset.abs() > anomaly_threshold {
        " [anomaly]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2}: offset = {:>+5.2} m{}",
        state.tick, next_offset, marker
    ));

    Ok((next_offset, logs))
}

/// The corrective P-controller. Given the post-step offset, return the
/// value to intervene with. With `p_gain = 0.85`, an offset of +0.40 m
/// becomes +0.06 m after correction (85% of the deviation cancelled).
pub fn correction(offset: FloatType, p_gain: FloatType) -> FloatType {
    offset * (1.0 - p_gain)
}

/// The corrective step. Reads the gain from the lane context the process
/// carries and `alternate_value`s the offset with the P-controller's output,
/// which logs the substitution. A missing offset, context or gain ends the
/// process in error.
pub fn correct(
    value: CausalEffect<FloatType>,
    state: VehicleState,
    ctx: Option<LaneContext>,
) -> LaneProcess<FloatType> {
    let offsets = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)
        .and_then(|offset| {
            let lane = ctx.as_ref().ok_or_else(CausalityError::MissingContext)?;
            Ok((offset, correction(offset, read(lane, P_GAIN)?)))
        });
    match offsets {
        Ok((offset, corrected)) => LaneProcess::<FloatType>::new(
            Ok(CausalEffect::value(offset)),
            state,
            ctx,
            EffectLog::new(),
        )
        .alternate_value(corrected),
        Err(err) => LaneProcess::<FloatType>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// Initial process at offset 0 (vehicle centred at the start), carrying `lane` as its context.
pub fn initial_process(lane: LaneContext) -> LaneProcess<FloatType> {
    LaneProcess::<FloatType>::new(
        Ok(CausalEffect::value(0.0)),
        VehicleState::default(),
        Some(lane),
        EffectLog::new(),
    )
}
