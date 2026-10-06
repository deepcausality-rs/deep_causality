/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess};

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
pub type FloatType = f64;

/// The world one patient is reasoned about in: numeric data only, no space and no time.
pub type PatientContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: treatment assignment, `1.0` treated, `0.0` control.
const ASSIGNMENT: ContextoidId = 1;
/// Contextoid id: the dose model, the BP change the drug produces when administered, BP points.
const DOSE: ContextoidId = 2;
/// Contextoid id: the patient's baseline blood pressure, BP points.
pub const INITIAL_BP: ContextoidId = 3;

/// Build the world the chain reasons against.
///
/// The treatment assignment, the dose model and the patient's baseline blood pressure are three
/// `Data` contextoids in one [`PatientContext`]. Alternating between worlds swaps the whole
/// context, so the counterfactual differs from the factual in exactly one contextoid — the
/// assignment — while the dose model, the patient's baseline and the chain itself stay invariant.
pub fn treatment_world(
    initial_bp: FloatType,
    drug_administered: bool,
    drug_effect_if_administered: FloatType,
) -> Result<PatientContext, ContextIndexError> {
    let assignment = if drug_administered { 1.0 } else { 0.0 };
    let facts = [
        (ASSIGNMENT, assignment),
        (DOSE, drug_effect_if_administered),
        (INITIAL_BP, initial_bp),
    ];
    let mut context = Context::with_capacity(1, "treatment world", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}

/// Read the `Data` contextoid with contextoid id `id` out of the carried context.
pub fn read(context: &PatientContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| CausalityError::ModelError(format!("no Datoid with contextoid id {id}")))
}

/// The value a finished chain carries, or the error that ended it.
pub fn value_of(
    process: &PropagatingProcess<FloatType, (), PatientContext>,
) -> Result<FloatType, CausalityError> {
    match process.error() {
        Some(error) => Err(error.clone()),
        None => process
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
}

/// Carry a stage's result on: its value with the state and the world when it succeeded, its
/// error when it failed.
fn emit(
    result: Result<FloatType, CausalityError>,
    state: (),
    world: PatientContext,
) -> PropagatingProcess<FloatType, (), PatientContext> {
    match result {
        Ok(value) => {
            PropagatingProcess::with_state(PropagatingEffect::pure(value), state, Some(world))
        }
        Err(error) => PropagatingProcess::from_error(error),
    }
}

/// Build the seed carrier (no binds applied yet), seeded with the world's baseline blood
/// pressure. The caller can either run the binds directly to get the factual outcome, or call
/// `.alternate_context(other)` first and *then* the binds to get the counterfactual outcome.
pub fn start(world: PatientContext) -> PropagatingProcess<FloatType, (), PatientContext> {
    let initial_bp = read(&world, INITIAL_BP);
    emit(initial_bp, (), world)
}

/// Stage 1: produce the drug-induced BP change for the current Context.
pub fn apply_drug_effect(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<PatientContext>,
) -> PropagatingProcess<FloatType, (), PatientContext> {
    let Some(ctx) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };
    let drug_effect = read(&ctx, ASSIGNMENT).and_then(|assignment| {
        if assignment > 0.5 {
            read(&ctx, DOSE)
        } else {
            Ok(0.0)
        }
    });
    emit(drug_effect, state, ctx)
}

/// Stage 2: add the drug effect to the baseline blood pressure the Context holds.
pub fn compute_final_bp(
    value: CausalEffect<FloatType>,
    state: (),
    context: Option<PatientContext>,
) -> PropagatingProcess<FloatType, (), PatientContext> {
    let Some(drug_effect) = value.into_value() else {
        return PropagatingProcess::from_error(CausalityError::ValueNotAvailable());
    };
    let Some(ctx) = context else {
        return PropagatingProcess::from_error(CausalityError::MissingContext());
    };
    let final_bp = read(&ctx, INITIAL_BP).map(|initial_bp| initial_bp + drug_effect);
    emit(final_bp, state, ctx)
}
