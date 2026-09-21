/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # CATE via the Causal Monad
//!
//! Conditional Average Treatment Effect on
//! `PropagatingProcess<f64, (), PatientContext>` using the `Alternatable`
//! family. The CATE for a subgroup `S` is the mean of per-patient
//! individual treatment effects:
//!
//! ```text
//! CATE(S) = E[ Y(do(T=1)) - Y(do(T=0)) | X in S ]
//! ```
//!
//! For each patient in the subgroup, the chain runs twice:
//!
//! 1. Factually under the **treatment context** (drug_administered = true).
//! 2. Counterfactually via `.alternate_context(control)` (drug_administered
//!    = false).
//!
//! The same two-stage bind chain runs for both worlds. The difference is
//! the patient's individual treatment effect; their mean across the
//! subgroup is the CATE.
//!
//! Per-patient audit logs contain one `!!ContextAlternation!!` entry
//! pinpointing the switch from treatment to control.

use deep_causality_context::{
    BaseContext, Context, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, PropagatingEffect, PropagatingProcess,
};

/// Node indices of the three `Data` contextoids each patient world carries.
const AGE: usize = 0;
const INITIAL_BP: usize = 1;
const ASSIGNMENT: usize = 2;

pub type FloatType = f64;

fn main() {
    println!("\n=== CATE via the Causal Monad: drug effect on BP for patients age > 65 ===\n");

    let population = create_patient_population();
    println!("Population: {} patients.", population.len());

    let subgroup: Vec<&BaseContext> = population
        .iter()
        .filter(|p| read(p, AGE) > AGE_THRESHOLD)
        .collect();
    println!(
        "Subgroup (age > {AGE_THRESHOLD}): {} patients.",
        subgroup.len()
    );
    println!();

    // No patient in the subgroup means no individual treatment effect to average, and `E[·]` over
    // an empty set is not a treatment effect of zero — it is no answer at all. Refused here rather
    // than at the mean, so the reason is the empty subgroup and not a statistic that declined.
    if subgroup.is_empty() {
        println!("No patient is over {AGE_THRESHOLD}: this subgroup has no CATE to estimate.");
        return;
    }

    let ites: Vec<FloatType> = subgroup
        .iter()
        .map(|p| individual_treatment_effect(p))
        .collect();

    let cate = deep_causality_stats::mean(&ites).expect("the subgroup is non-empty");
    println!("\n--- CATE = mean(ITE over subgroup) = {:.2} ---", cate);
    println!(
        "Interpretation: for the over-{AGE_THRESHOLD} subgroup, administering the drug is\n\
         predicted to lower BP by {:.2} points on average.",
        -cate
    );
}

const AGE_THRESHOLD: FloatType = 65.0;
const DRUG_EFFECT_IF_ADMINISTERED: FloatType = -10.0;

/// Compute the individual treatment effect for one patient by running the
/// same chain twice: factually under treatment, then via
/// `alternate_context(control)`. The difference is `Y(1) - Y(0)`.
fn individual_treatment_effect(patient: &BaseContext) -> FloatType {
    let age = read(patient, AGE);
    let initial_bp = read(patient, INITIAL_BP);
    let treatment = patient_world(age, initial_bp, true);
    let control = patient_world(age, initial_bp, false);

    let y1 = run_binds(start(treatment.clone()));
    let y0 = run_binds(start(treatment).alternate_context(control));

    let y1_bp = y1.value_cloned().unwrap();
    let y0_bp = y0.value_cloned().unwrap();
    let ite = y1_bp - y0_bp;

    println!(
        "  patient age={:>4.1} initial_bp={:>5.1}  Y(1)={:>5.1}  Y(0)={:>5.1}  ITE={:+.1}",
        age, initial_bp, y1_bp, y0_bp, ite
    );

    ite
}

// --- Model: patient context, chain seed, bind stages, population ---

/// Build the world one patient is reasoned about in: age, baseline BP and treatment assignment,
/// as three `Data` contextoids in one typed [`BaseContext`]. The counterfactual world differs in
/// exactly one contextoid — the assignment.
fn patient_world(age: FloatType, initial_bp: FloatType, drug_administered: bool) -> BaseContext {
    let mut context = Context::with_capacity(1, "patient", 3);
    let assignment = if drug_administered { 1.0 } else { 0.0 };

    for (id, value) in [(1, age), (2, initial_bp), (3, assignment)] {
        context
            .add_node(Contextoid::new(
                id,
                ContextoidType::Datoid(Data::new(id, value)),
            ))
            .expect("patient contextoid is accepted");
    }

    context
}

/// Read one `Data` contextoid's payload out of a patient world.
fn read(context: &BaseContext, index: usize) -> FloatType {
    context
        .get_node(index)
        .expect("contextoid is present")
        .vertex_type()
        .dataoid()
        .expect("contextoid is a Datoid")
        .get_data()
}

/// Run the two bind stages on a seed carrier. The caller decides whether
/// to call `.alternate_context(...)` on the seed before passing it in;
/// the alternation must land *before* the binds so both stages read the
/// alternated context.
fn run_binds(
    seeded: PropagatingProcess<FloatType, (), BaseContext>,
) -> PropagatingProcess<FloatType, (), BaseContext> {
    seeded.bind(stage_drug_effect).bind(stage_final_bp)
}

fn start(patient: BaseContext) -> PropagatingProcess<FloatType, (), BaseContext> {
    let initial_bp = read(&patient, INITIAL_BP);
    let seed = PropagatingEffect::pure(initial_bp);
    PropagatingProcess::with_state(seed, (), Some(patient))
}

/// Stage 1: drug effect from the treatment assignment.
fn stage_drug_effect(
    _value: CausalEffect<FloatType>,
    state: (),
    context: Option<BaseContext>,
) -> PropagatingProcess<FloatType, (), BaseContext> {
    let ctx = context.expect("the patient world must be set before stage 1");
    let drug_effect = if read(&ctx, ASSIGNMENT) > 0.5 {
        DRUG_EFFECT_IF_ADMINISTERED
    } else {
        0.0
    };
    let next = PropagatingEffect::pure(drug_effect);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

/// Stage 2: add the drug effect to the patient's initial BP.
fn stage_final_bp(
    value: CausalEffect<FloatType>,
    state: (),
    context: Option<BaseContext>,
) -> PropagatingProcess<FloatType, (), BaseContext> {
    let drug_effect = value
        .into_value()
        .expect("stage_drug_effect must produce a numeric drug-effect Value");
    let ctx = context.expect("the patient world must be set before stage 2");
    let final_bp = read(&ctx, INITIAL_BP) + drug_effect;
    let next = PropagatingEffect::pure(final_bp);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

fn create_patient_population() -> Vec<BaseContext> {
    [
        (55.0, 145.0),
        (70.0, 150.0),
        (68.0, 155.0),
        (45.0, 130.0),
        (80.0, 160.0),
        (72.0, 148.0),
        (60.0, 140.0),
    ]
    .into_iter()
    .map(|(age, initial_bp)| patient_world(age, initial_bp, false))
    .collect()
}
