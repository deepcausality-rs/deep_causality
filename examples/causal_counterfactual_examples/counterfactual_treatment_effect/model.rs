/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types and stage logic for the CATE chain.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalFlow, CausalityError, PropagatingProcess};

/// Switch this alias to `f32` for low precision, `f64` for standard precision,
/// or `Float106` for high precision. Literals in this file would need lifting
/// through `deep_causality_num::lift` to switch to `Float106`.
pub type FloatType = f64;

/// One patient and the drug's dose response, one `Data` contextoid per quantity. The context id is
/// the patient id. The context holds no position, clock or event, so its spatial, temporal and
/// spacetime slots are empty.
pub type PatientContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: age in years.
pub const AGE: ContextoidId = 1;
/// Contextoid id: baseline systolic blood pressure in mmHg.
pub const BASELINE_BP: ContextoidId = 2;
/// Contextoid id: blood-pressure reduction under treatment at or below `RESPONSE_ONSET_AGE`, in
/// mmHg.
pub const BASE_REDUCTION: ContextoidId = 3;
/// Contextoid id: additional reduction per year of age above `RESPONSE_ONSET_AGE`, in mmHg per
/// year.
pub const REDUCTION_PER_YEAR: ContextoidId = 4;
/// Contextoid id: age above which the reduction grows with age, in years.
pub const RESPONSE_ONSET_AGE: ContextoidId = 5;
/// Contextoid id: drift of the blood pressure independent of treatment, in mmHg.
pub const NATURAL_DRIFT: ContextoidId = 6;

/// Read the payload of the `Data` contextoid `id` out of a patient context.
pub fn read(context: &PatientContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    context.get_data_by_id(id).ok_or_else(|| {
        CausalityError::MissingParameter(format!(
            "the patient context holds no Datoid with contextoid id {id}"
        ))
    })
}

/// Build one patient: one `Data` contextoid per quantity, keyed by its contextoid id. Every
/// patient carries the same dose response; it is stronger for older patients, and that
/// heterogeneity is what makes the CATE non-trivial. Pharmacology is deliberately shallow; the
/// example is about the causal structure.
fn patient(
    id: u64,
    age: FloatType,
    baseline_bp: FloatType,
) -> Result<PatientContext, ContextIndexError> {
    let facts = [
        (AGE, age),
        (BASELINE_BP, baseline_bp),
        (BASE_REDUCTION, 5.0),
        (REDUCTION_PER_YEAR, 0.3),
        (RESPONSE_ONSET_AGE, 50.0),
        (NATURAL_DRIFT, 0.5),
    ];
    let mut context = Context::with_capacity(id, "patient", facts.len());
    for (contextoid_id, value) in facts {
        context.add_node(Contextoid::new(
            contextoid_id,
            ContextoidType::Datoid(Data::new(contextoid_id, value)),
        ))?;
    }
    Ok(context)
}

/// Post-treatment blood pressure under `do(T = treatment)`.
///
/// The flow begins with a NaN treatment value and carries the patient as its
/// context; `.alternate_value(treatment)` fires before the step that consumes
/// it. The step reads the alternate value as `t` and the patient's covariates
/// and dose response from the context, and computes the resulting
/// post-treatment blood pressure. `into_process` hands the underlying process
/// back so the caller can read both its value and its audit log.
pub fn evaluate_under(
    patient: &PatientContext,
    treatment: FloatType,
) -> PropagatingProcess<FloatType, (), PatientContext> {
    CausalFlow::value(FloatType::NAN)
        .context(patient.clone())
        .alternate_value(treatment)
        .try_step_with(|t, _state, patient| {
            let patient = patient.ok_or(CausalityError::MissingContext())?;
            let age = read(patient, AGE)?;
            let baseline = read(patient, BASELINE_BP)?;

            let drug_reduction = if t > 0.5 {
                read(patient, BASE_REDUCTION)?
                    + read(patient, REDUCTION_PER_YEAR)?
                        * (age - read(patient, RESPONSE_ONSET_AGE)?).max(0.0)
            } else {
                0.0
            };

            Ok(baseline - drug_reduction + read(patient, NATURAL_DRIFT)?)
        })
        .into_process()
}

pub fn potential_outcomes(
    patient: &PatientContext,
) -> Result<(FloatType, FloatType), CausalityError> {
    let y1 = CausalFlow::from(evaluate_under(patient, 1.0)).finish()?;
    let y0 = CausalFlow::from(evaluate_under(patient, 0.0)).finish()?;
    Ok((y1, y0))
}

pub fn synthetic_cohort() -> Result<Vec<PatientContext>, ContextIndexError> {
    [
        (1, 55.0, 140.0),
        (2, 62.0, 135.0),
        (3, 68.0, 150.0),
        (4, 71.0, 145.0),
        (5, 58.0, 138.0),
        (6, 74.0, 155.0),
        (7, 80.0, 160.0),
        (8, 48.0, 130.0),
    ]
    .into_iter()
    .map(|(id, age, baseline_bp)| patient(id, age, baseline_bp))
    .collect()
}
