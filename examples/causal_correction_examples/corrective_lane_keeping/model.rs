/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Simulation stage and corrective driver loop for the lane-keeping example.

use crate::model_types::{
    FloatType, LaneConfig, LaneProcess, VehicleState, drift_at, nominal_lane_config,
};
use deep_causality_core::{CausalEffect, EffectLog};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the current lateral
/// offset; the stage adds the per-tick drift, advances `state.tick`,
/// appends the new offset to `state.trajectory`, and records whether the
/// vehicle has now left the lane.
pub fn simulate_step(
    value: CausalEffect<FloatType>,
    mut state: VehicleState,
    ctx: Option<LaneConfig>,
) -> LaneProcess<FloatType> {
    let prev_offset = value.into_value().unwrap_or(0.0);
    let cfg = ctx.clone().expect("LaneConfig required");

    let next_offset = prev_offset + drift_at(state.tick);
    state.tick += 1;
    state.trajectory.push(next_offset);
    if next_offset.abs() > state.max_offset_observed.abs() {
        state.max_offset_observed = next_offset;
    }
    if state.catastrophic_at.is_none() && next_offset.abs() > cfg.lane_half_width {
        state.catastrophic_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.catastrophic_at.is_some() {
        " [OFF-ROAD]"
    } else if next_offset.abs() > cfg.anomaly_threshold {
        " [anomaly]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2}: offset = {:>+5.2} m{}",
        state.tick, next_offset, marker
    ));

    LaneProcess::<FloatType>::new(Ok(CausalEffect::value(next_offset)), state, ctx, logs)
}

/// The corrective P-controller. Given the post-step offset, return the
/// value to intervene with. With `p_gain = 0.85`, an offset of +0.40 m
/// becomes +0.06 m after correction (85% of the deviation cancelled).
pub fn correction(offset: FloatType, cfg: &LaneConfig) -> FloatType {
    offset * (1.0 - cfg.p_gain)
}

/// Initial process at offset 0 (vehicle centred at the start).
pub fn initial_process() -> LaneProcess<FloatType> {
    LaneProcess::<FloatType>::new(
        Ok(CausalEffect::value(0.0)),
        VehicleState::default(),
        Some(nominal_lane_config()),
        EffectLog::new(),
    )
}
