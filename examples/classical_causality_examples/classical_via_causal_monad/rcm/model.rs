/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
};
use deep_causality_core::{CausalEffect, PropagatingEffect, PropagatingProcess};

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
pub type FloatType = f64;

/// Node index of the treatment assignment: `1.0` treated, `0.0` control.
const ASSIGNMENT: usize = 0;
/// Node index of the dose model: the BP change the drug produces when administered.
const DOSE: usize = 1;

pub const PATIENT_INITIAL_BP: FloatType = 145.0;

/// Build the world the chain reasons against.
///
/// The treatment assignment and the dose model are two `Data` contextoids in one typed
/// [`BaseContext`]. Alternating between worlds swaps the whole context, so the counterfactual
/// differs from the factual in exactly one contextoid — the assignment — while the dose model,
/// the patient's baseline and the chain itself stay invariant.
pub fn treatment_world(
    drug_administered: bool,
    drug_effect_if_administered: FloatType,
) -> BaseContext {
    let mut context = Context::with_capacity(1, "treatment world", 2);

    let assignment = if drug_administered { 1.0 } else { 0.0 };
    context
        .add_node(Contextoid::new(
            1,
            ContextoidType::Datoid(Data::new(1, assignment)),
        ))
        .expect("assignment contextoid is accepted");
    context
        .add_node(Contextoid::new(
            2,
            ContextoidType::Datoid(Data::new(2, drug_effect_if_administered)),
        ))
        .expect("dose contextoid is accepted");

    context
}

/// Read one `Data` contextoid's payload out of the carried context.
fn read(context: &BaseContext, index: usize) -> FloatType {
    context
        .get_node(index)
        .expect("contextoid is present")
        .vertex_type()
        .dataoid()
        .expect("contextoid is a Datoid")
        .get_data()
}

/// Build the seed carrier (no binds applied yet). The caller can either
/// run the binds directly to get the factual outcome, or call
/// `.alternate_context(other)` first and *then* the binds to get the
/// counterfactual outcome.
pub fn start(world: BaseContext) -> PropagatingProcess<FloatType, (), BaseContext> {
    let seed = PropagatingEffect::pure(PATIENT_INITIAL_BP);
    PropagatingProcess::with_state(seed, (), Some(world))
}

/// Stage 1: produce the drug-induced BP change for the current Context.
pub fn apply_drug_effect(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<BaseContext>,
) -> PropagatingProcess<FloatType, (), BaseContext> {
    let ctx = context.expect("the treatment world must be set before stage 1");
    let drug_effect = if read(&ctx, ASSIGNMENT) > 0.5 {
        read(&ctx, DOSE)
    } else {
        0.0
    };
    let next = PropagatingEffect::pure(drug_effect);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

/// Stage 2: add the drug effect to the baseline blood pressure.
pub fn compute_final_bp(
    value: CausalEffect<FloatType>,
    state: (),
    context: Option<BaseContext>,
) -> PropagatingProcess<FloatType, (), BaseContext> {
    let drug_effect = value
        .into_value()
        .expect("apply_drug_effect must produce a numeric drug-effect Value");
    let final_bp = PATIENT_INITIAL_BP + drug_effect;
    let next = PropagatingEffect::pure(final_bp);
    PropagatingProcess::with_state(next, state, context)
}
