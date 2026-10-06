/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess};

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
pub type FloatType = f64;

/// The world one patient is reasoned about in: numeric data only, no space and no time.
pub type PatientContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node index of the treatment assignment: `1.0` treated, `0.0` control.
const ASSIGNMENT: usize = 0;
/// Node index of the dose model: the BP change the drug produces when administered.
const DOSE: usize = 1;
/// Node index of the patient's baseline blood pressure.
pub const INITIAL_BP: usize = 2;

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
    let mut context = Context::with_capacity(1, "treatment world", 3);

    let assignment = if drug_administered { 1.0 } else { 0.0 };
    for (id, value) in [
        (1, assignment),
        (2, drug_effect_if_administered),
        (3, initial_bp),
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}

/// Read one `Data` contextoid's payload out of the carried context.
pub fn read(context: &PatientContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| CausalityError::ModelError(format!("context node {index} is not a Datoid")))
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
