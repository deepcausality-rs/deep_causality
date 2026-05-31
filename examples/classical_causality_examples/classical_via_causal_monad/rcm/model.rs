/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{EffectValue, PropagatingEffect, PropagatingProcess};

/// Treatment assignment carried in the Context channel. Alternation between
/// treatment and control is the only thing that differs between the two
/// runs; the chain itself, the patient's baseline, and the dose model are
/// invariant across worlds.
#[derive(Clone, Debug, PartialEq)]
pub struct TreatmentContext {
    pub drug_administered: bool,
    pub drug_effect_if_administered: f64,
}

pub const PATIENT_INITIAL_BP: f64 = 145.0;

/// Build the seed carrier (no binds applied yet). The caller can either
/// run the binds directly to get the factual outcome, or call
/// `.alternate_context(other)` first and *then* the binds to get the
/// counterfactual outcome.
pub fn start(treatment: TreatmentContext) -> PropagatingProcess<f64, (), TreatmentContext> {
    let seed = PropagatingEffect::pure(PATIENT_INITIAL_BP);
    PropagatingProcess::with_state(seed, (), Some(treatment))
}

/// Stage 1: produce the drug-induced BP change for the current Context.
pub fn apply_drug_effect(
    _value: EffectValue<f64>,
    state: (),
    context: Option<TreatmentContext>,
) -> PropagatingProcess<f64, (), TreatmentContext> {
    let ctx = context.expect("TreatmentContext must be set before stage 1");
    let drug_effect = if ctx.drug_administered {
        ctx.drug_effect_if_administered
    } else {
        0.0
    };
    let next = PropagatingEffect::pure(drug_effect);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

/// Stage 2: add the drug effect to the baseline blood pressure.
pub fn compute_final_bp(
    value: EffectValue<f64>,
    state: (),
    context: Option<TreatmentContext>,
) -> PropagatingProcess<f64, (), TreatmentContext> {
    let drug_effect = value
        .into_value()
        .expect("apply_drug_effect must produce a numeric drug-effect Value");
    let final_bp = PATIENT_INITIAL_BP + drug_effect;
    let next = PropagatingEffect::pure(final_bp);
    PropagatingProcess::with_state(next, state, context)
}
