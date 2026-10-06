/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # CATE via the Causal Monad
//!
//! Conditional Average Treatment Effect on
//! `PropagatingProcess<FloatType, (), PatientContext>` using the `Alternatable`
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
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{
    AlternatableContext, CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess,
};
use std::error::Error;

/// Contextoid id: patient age, years.
const AGE: ContextoidId = 1;
/// Contextoid id: baseline blood pressure, BP points.
const INITIAL_BP: ContextoidId = 2;
/// Contextoid id: the BP change the drug produces when administered, BP points.
const DOSE: ContextoidId = 3;
/// Contextoid id: treatment assignment, `1.0` treated, `0.0` control.
const ASSIGNMENT: ContextoidId = 4;

pub type FloatType = f64;

/// The world one patient is reasoned about in: numeric data only, no space and no time.
type PatientContext = Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n=== CATE via the Causal Monad: drug effect on BP for patients age > 65 ===\n");

    let population = create_patient_population()?;
    println!("Population: {} patients.", population.len());

    let ages = population
        .iter()
        .map(|p| read(p, AGE).map(|age| (p, age)))
        .collect::<Result<Vec<_>, _>>()?;
    let subgroup: Vec<&PatientContext> = ages
        .into_iter()
        .filter(|&(_, age)| age > AGE_THRESHOLD)
        .map(|(p, _)| p)
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
        return Ok(());
    }

    let ites: Vec<FloatType> = subgroup
        .iter()
        .map(|p| individual_treatment_effect(p))
        .collect::<Result<_, _>>()?;

    let cate = deep_causality_stats::mean(&ites)?;
    println!("\n--- CATE = mean(ITE over subgroup) = {:.2} ---", cate);
    println!(
        "Interpretation: for the over-{AGE_THRESHOLD} subgroup, administering the drug is\n\
         predicted to lower BP by {:.2} points on average.",
        -cate
    );
    Ok(())
}

/// The subgroup `S` the CATE is conditioned on: patients older than this age.
const AGE_THRESHOLD: FloatType = 65.0;

/// Compute the individual treatment effect for one patient by running the
/// same chain twice: factually under treatment, then via
/// `alternate_context(control)`. The difference is `Y(1) - Y(0)`.
fn individual_treatment_effect(patient: &PatientContext) -> Result<FloatType, Box<dyn Error>> {
    let age = read(patient, AGE)?;
    let initial_bp = read(patient, INITIAL_BP)?;
    let dose = read(patient, DOSE)?;
    let treatment = patient_world(age, initial_bp, dose, true)?;
    let control = patient_world(age, initial_bp, dose, false)?;

    let y1 = run_binds(start(treatment.clone()));
    let y0 = run_binds(start(treatment).alternate_context(control));

    let y1_bp = value_of(&y1)?;
    let y0_bp = value_of(&y0)?;
    let ite = y1_bp - y0_bp;

    println!(
        "  patient age={:>4.1} initial_bp={:>5.1}  Y(1)={:>5.1}  Y(0)={:>5.1}  ITE={:+.1}",
        age, initial_bp, y1_bp, y0_bp, ite
    );

    Ok(ite)
}

// --- Model: patient context, chain seed, bind stages, population ---

/// Build the world one patient is reasoned about in: age, baseline BP, the drug's dose and the
/// treatment assignment, as four `Data` contextoids in one [`PatientContext`]. The counterfactual
/// world differs in exactly one contextoid — the assignment.
fn patient_world(
    age: FloatType,
    initial_bp: FloatType,
    dose: FloatType,
    drug_administered: bool,
) -> Result<PatientContext, ContextIndexError> {
    let assignment = if drug_administered { 1.0 } else { 0.0 };
    let facts = [
        (AGE, age),
        (INITIAL_BP, initial_bp),
        (DOSE, dose),
        (ASSIGNMENT, assignment),
    ];
    let mut context = Context::with_capacity(1, "patient", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }

    Ok(context)
}

/// Read the `Data` contextoid with contextoid id `id` out of a patient world.
fn read(context: &PatientContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| CausalityError::ModelError(format!("no Datoid with contextoid id {id}")))
}

/// The value a finished chain carries, or the error that ended it.
fn value_of(
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

/// Run the two bind stages on a seed carrier. The caller decides whether
/// to call `.alternate_context(...)` on the seed before passing it in;
/// the alternation must land *before* the binds so both stages read the
/// alternated context.
fn run_binds(
    seeded: PropagatingProcess<FloatType, (), PatientContext>,
) -> PropagatingProcess<FloatType, (), PatientContext> {
    seeded.bind(stage_drug_effect).bind(stage_final_bp)
}

fn start(patient: PatientContext) -> PropagatingProcess<FloatType, (), PatientContext> {
    let initial_bp = read(&patient, INITIAL_BP);
    emit(initial_bp, (), patient)
}

/// Stage 1: drug effect from the treatment assignment and the dose.
fn stage_drug_effect(
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

/// Stage 2: add the drug effect to the patient's initial BP.
fn stage_final_bp(
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

/// The population: `(age, initial_bp)` per patient. The drug lowers every patient's BP by 10
/// points when administered.
fn create_patient_population() -> Result<Vec<PatientContext>, ContextIndexError> {
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
    .map(|(age, initial_bp)| patient_world(age, initial_bp, -10.0, false))
    .collect()
}
