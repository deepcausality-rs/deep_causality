/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Glucose dynamics stage and corrective driver loops.

use crate::model_types::{
    FloatType, HYPERGLYCEMIC_THRESHOLD, INSULIN_SENSITIVITY, KETOACIDOSIS_THRESHOLD, PatientState,
    PumpContext, PumpProcess, TARGET_GLUCOSE, TICK_MINUTES, perturbation_at, read,
};
use deep_causality_core::{AlternatableValue, CausalEffect, CausalityError, EffectLog};
use deep_causality_haft::LogAddEntry;

/// One simulation tick. The value channel carries the current blood
/// glucose level; the stage adds the scheduled perturbation, advances
/// `state.tick`, records the trajectory, and flags ketoacidosis the
/// first time the catastrophic threshold is crossed. The perturbation and
/// both thresholds are read from the pump context the process carries. A
/// missing glucose value, context or pump fact ends the process in error
/// with the state unchanged.
pub fn simulate_step(
    value: CausalEffect<FloatType>,
    mut state: PatientState,
    ctx: Option<PumpContext>,
) -> PumpProcess<FloatType> {
    match advance(value, &mut state, ctx.as_ref()) {
        Ok((next, logs)) => {
            PumpProcess::<FloatType>::new(Ok(CausalEffect::value(next)), state, ctx, logs)
        }
        Err(err) => PumpProcess::<FloatType>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// The tick itself: reads every input first, then advances `state` and
/// returns the new glucose level with its log entry.
fn advance(
    value: CausalEffect<FloatType>,
    state: &mut PatientState,
    ctx: Option<&PumpContext>,
) -> Result<(FloatType, EffectLog), CausalityError> {
    let prev = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)?;
    let pump = ctx.ok_or_else(CausalityError::MissingContext)?;
    let perturbation = perturbation_at(state.tick, pump)?;
    let ketoacidosis_threshold = read(pump, KETOACIDOSIS_THRESHOLD)?;
    let hyperglycemic_threshold = read(pump, HYPERGLYCEMIC_THRESHOLD)?;

    let next = prev + perturbation;
    state.tick += 1;
    state.trajectory.push(next);
    if next > state.max_glucose_observed {
        state.max_glucose_observed = next;
    }
    if state.ketoacidosis_at.is_none() && next > ketoacidosis_threshold {
        state.ketoacidosis_at = Some(state.tick);
    }

    let mut logs = EffectLog::new();
    let marker = if state.ketoacidosis_at.is_some() {
        " [KETOACIDOSIS]"
    } else if next > hyperglycemic_threshold {
        " [hyperglycemic]"
    } else {
        ""
    };
    logs.add_entry(&format!(
        "tick {:>2} ({:>2} min): glucose = {:>5.1} mg/dL{}",
        state.tick,
        state.tick * TICK_MINUTES,
        next,
        marker
    ));

    Ok((next, logs))
}

/// Corrective bolus calculation. The pump infuses enough fast-acting
/// insulin to bring glucose from the current reading down to
/// `target_glucose`, at `insulin_sensitivity` mg/dL per unit. Returns the
/// post-bolus glucose value (the intervened-with value) and the units
/// administered.
pub fn corrective_bolus(
    current: FloatType,
    target_glucose: FloatType,
    insulin_sensitivity: FloatType,
) -> (FloatType, FloatType) {
    let overshoot = (current - target_glucose).max(0.0);
    let units = overshoot / insulin_sensitivity;
    let corrected = target_glucose;
    (corrected, units)
}

/// The corrective step. Reads the target and the insulin sensitivity from
/// the pump context the process carries, records the bolus and the units
/// delivered, and `alternate_value`s the glucose with the post-bolus level,
/// which logs the substitution. A missing glucose value, context or pump
/// fact ends the process in error with the state unchanged.
pub fn bolus(
    value: CausalEffect<FloatType>,
    mut state: PatientState,
    ctx: Option<PumpContext>,
) -> PumpProcess<FloatType> {
    let dose = value
        .into_value()
        .ok_or_else(CausalityError::ValueNotAvailable)
        .and_then(|glucose| {
            let pump = ctx.as_ref().ok_or_else(CausalityError::MissingContext)?;
            let target = read(pump, TARGET_GLUCOSE)?;
            let sensitivity = read(pump, INSULIN_SENSITIVITY)?;
            Ok((glucose, corrective_bolus(glucose, target, sensitivity)))
        });
    match dose {
        Ok((glucose, (corrected, units))) => {
            state.bolus_count += 1;
            state.total_insulin_units += units;
            PumpProcess::<FloatType>::new(
                Ok(CausalEffect::value(glucose)),
                state,
                ctx,
                EffectLog::new(),
            )
            .alternate_value(corrected)
        }
        Err(err) => PumpProcess::<FloatType>::new(Err(err), state, ctx, EffectLog::new()),
    }
}

/// Initial process at the fasting baseline, carrying `pump` as its context.
pub fn initial_process(pump: PumpContext) -> PumpProcess<FloatType> {
    PumpProcess::<FloatType>::new(
        Ok(CausalEffect::value(100.0)), // fasting baseline
        PatientState::default(),
        Some(pump),
        EffectLog::new(),
    )
}
