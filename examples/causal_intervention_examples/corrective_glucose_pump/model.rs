/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Glucose dynamics stage and corrective driver loops.

use crate::model_types::{
    FloatType, PatientState, PumpConfig, PumpProcess, nominal_pump_config, perturbation_at,
};
use deep_causality_core::{EffectLog, EffectValue};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the current blood
/// glucose level; the stage adds the scheduled perturbation, advances
/// `state.tick`, records the trajectory, and flags ketoacidosis the
/// first time the catastrophic threshold is crossed.
pub fn simulate_step(
    value: EffectValue<FloatType>,
    mut state: PatientState,
    ctx: Option<PumpConfig>,
) -> PumpProcess<FloatType> {
    let prev = value.into_value().unwrap_or(100.0);
    let cfg = ctx.clone().expect("PumpConfig required");

    let next = prev + perturbation_at(state.tick);
    state.tick += 1;
    state.trajectory.push(next);
    if next > state.max_glucose_observed {
        state.max_glucose_observed = next;
    }
    if state.ketoacidosis_at.is_none() && next > cfg.ketoacidosis_threshold {
        state.ketoacidosis_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.ketoacidosis_at.is_some() {
        " [KETOACIDOSIS]"
    } else if next > cfg.hyperglycemic_threshold {
        " [hyperglycemic]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2} ({:>2} min): glucose = {:>5.1} mg/dL{}",
        state.tick,
        state.tick * 15,
        next,
        marker
    ));

    PumpProcess::<FloatType> {
        value: EffectValue::Value(next),
        state,
        context: ctx,
        error: None,
        logs,
    }
}

/// Corrective bolus calculation. The pump infuses enough fast-acting
/// insulin to bring glucose from the current reading down to the
/// configured target. Returns the post-bolus glucose value (the
/// intervened-with value) and the units administered.
pub fn corrective_bolus(current: FloatType, cfg: &PumpConfig) -> (FloatType, FloatType) {
    let overshoot = (current - cfg.target_glucose).max(0.0);
    let units = overshoot / cfg.units_per_mg_dl;
    let corrected = cfg.target_glucose;
    (corrected, units)
}

pub fn initial_process() -> PumpProcess<FloatType> {
    PumpProcess::<FloatType> {
        value: EffectValue::Value(100.0), // fasting baseline
        state: PatientState::default(),
        context: Some(nominal_pump_config()),
        error: None,
        logs: EffectLog::new(),
    }
}
