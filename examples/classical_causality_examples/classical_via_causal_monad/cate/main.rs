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

use deep_causality_core::{
    AlternatableContext, EffectValue, PropagatingEffect, PropagatingProcess,
};

fn main() {
    println!("\n=== CATE via the Causal Monad: drug effect on BP for patients age > 65 ===\n");

    let population = create_patient_population();
    println!("Population: {} patients.", population.len());

    let subgroup: Vec<&PatientContext> = population
        .iter()
        .filter(|p| p.age > AGE_THRESHOLD)
        .collect();
    println!(
        "Subgroup (age > {AGE_THRESHOLD}): {} patients.",
        subgroup.len()
    );
    println!();

    let ites: Vec<f64> = subgroup
        .iter()
        .map(|p| individual_treatment_effect(p))
        .collect();

    let cate = ites.iter().sum::<f64>() / ites.len() as f64;
    println!("\n--- CATE = mean(ITE over subgroup) = {:.2} ---", cate);
    println!(
        "Interpretation: for the over-{AGE_THRESHOLD} subgroup, administering the drug is\n\
         predicted to lower BP by {:.2} points on average.",
        -cate
    );
}

const AGE_THRESHOLD: f64 = 65.0;
const DRUG_EFFECT_IF_ADMINISTERED: f64 = -10.0;

/// Compute the individual treatment effect for one patient by running the
/// same chain twice: factually under treatment, then via
/// `alternate_context(control)`. The difference is `Y(1) - Y(0)`.
fn individual_treatment_effect(patient: &PatientContext) -> f64 {
    let treatment = PatientContext {
        drug_administered: true,
        ..patient.clone()
    };
    let control = PatientContext {
        drug_administered: false,
        ..patient.clone()
    };

    let y1 = run_binds(start(treatment.clone()));
    let y0 = run_binds(start(treatment).alternate_context(control));

    let y1_bp = y1.value_cloned().unwrap();
    let y0_bp = y0.value_cloned().unwrap();
    let ite = y1_bp - y0_bp;

    println!(
        "  patient age={:>4.1} initial_bp={:>5.1}  Y(1)={:>5.1}  Y(0)={:>5.1}  ITE={:+.1}",
        patient.age, patient.initial_bp, y1_bp, y0_bp, ite
    );

    ite
}

// --- Model: patient context, chain seed, bind stages, population ---

/// Patient + treatment-assignment state, carried in the Context channel.
#[derive(Clone, Debug, PartialEq)]
struct PatientContext {
    age: f64,
    initial_bp: f64,
    drug_administered: bool,
}

/// Run the two bind stages on a seed carrier. The caller decides whether
/// to call `.alternate_context(...)` on the seed before passing it in;
/// the alternation must land *before* the binds so both stages read the
/// alternated context.
fn run_binds(
    seeded: PropagatingProcess<f64, (), PatientContext>,
) -> PropagatingProcess<f64, (), PatientContext> {
    seeded.bind(stage_drug_effect).bind(stage_final_bp)
}

fn start(patient: PatientContext) -> PropagatingProcess<f64, (), PatientContext> {
    let initial_bp = patient.initial_bp;
    let seed = PropagatingEffect::pure(initial_bp);
    PropagatingProcess::with_state(seed, (), Some(patient))
}

/// Stage 1: drug effect from the treatment assignment.
fn stage_drug_effect(
    _value: EffectValue<f64>,
    state: (),
    context: Option<PatientContext>,
) -> PropagatingProcess<f64, (), PatientContext> {
    let ctx = context.expect("PatientContext must be set before stage 1");
    let drug_effect = if ctx.drug_administered {
        DRUG_EFFECT_IF_ADMINISTERED
    } else {
        0.0
    };
    let next = PropagatingEffect::pure(drug_effect);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

/// Stage 2: add the drug effect to the patient's initial BP.
fn stage_final_bp(
    value: EffectValue<f64>,
    state: (),
    context: Option<PatientContext>,
) -> PropagatingProcess<f64, (), PatientContext> {
    let drug_effect = value
        .into_value()
        .expect("stage_drug_effect must produce a numeric drug-effect Value");
    let ctx = context.expect("PatientContext must be set before stage 2");
    let final_bp = ctx.initial_bp + drug_effect;
    let next = PropagatingEffect::pure(final_bp);
    PropagatingProcess::with_state(next, state, Some(ctx))
}

fn create_patient_population() -> Vec<PatientContext> {
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
    .map(|(age, initial_bp)| PatientContext {
        age,
        initial_bp,
        drug_administered: false,
    })
    .collect()
}
