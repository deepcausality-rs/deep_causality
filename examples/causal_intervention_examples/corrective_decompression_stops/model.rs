/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tissue dynamics stage and corrective driver loops.

use crate::model_types::{
    DiveConfig, DiveProcess, DiveState, FloatType, ambient_pressure, inspired_n2_pp,
    nominal_dive_config,
};
use deep_causality_core::{EffectLog, EffectValue};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the *ascent command*
/// for this tick (metres to ascend). The stage applies the command,
/// equilibrates the tissue compartment toward the inspired N2 partial
/// pressure at the new depth, and updates the supersaturation ratio.
/// By default the carrier value is reset to the normal ascent rate at
/// the end of the stage. Interventions overwrite this to insert a stop.
pub fn simulate_step(
    value: EffectValue<FloatType>,
    mut state: DiveState,
    ctx: Option<DiveConfig>,
) -> DiveProcess<FloatType> {
    let cfg = ctx.clone().expect("DiveConfig required");
    let commanded_ascent = value.into_value().unwrap_or(cfg.normal_ascent_m_per_tick);

    state.depth_m = (state.depth_m - commanded_ascent).max(0.0);
    let ambient = ambient_pressure(state.depth_m);
    let p_insp_n2 = inspired_n2_pp(state.depth_m);
    let k = 1.0 - (-cfg.tick_minutes / cfg.half_time_min).exp();
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
    if state.dcs_at.is_none() && ratio > cfg.dcs_ratio_threshold {
        state.dcs_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.dcs_at == Some(state.tick) {
        " [DCS RISK]"
    } else if ratio > cfg.safety_ratio_threshold {
        " [supersaturated]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2}: depth = {:>4.1} m, tissue = {:>4.2} bar, ratio = {:>4.2}{}",
        state.tick, state.depth_m, state.tissue_n2_bar, ratio, marker
    ));

    DiveProcess::<FloatType> {
        // Carry the normal ascent rate forward. The closed-loop driver
        // overwrites this with 0.0 whenever a stop is needed.
        value: EffectValue::Value(cfg.normal_ascent_m_per_tick),
        state,
        context: ctx,
        error: None,
        logs,
    }
}

pub fn initial_process() -> DiveProcess<FloatType> {
    let cfg = nominal_dive_config();
    let state = DiveState {
        depth_m: cfg.starting_depth_m,
        tissue_n2_bar: cfg.starting_tissue_n2_bar,
        ..Default::default()
    };
    DiveProcess::<FloatType> {
        value: EffectValue::Value(cfg.normal_ascent_m_per_tick),
        state,
        context: Some(cfg),
        error: None,
        logs: EffectLog::new(),
    }
}
