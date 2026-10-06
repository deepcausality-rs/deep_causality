/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Domain types and stage logic for the CATE chain.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
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

/// Node index: age in years.
pub const AGE: usize = 0;
/// Node index: baseline systolic blood pressure in mmHg.
pub const BASELINE_BP: usize = 1;
/// Node index: blood-pressure reduction under treatment at or below `RESPONSE_ONSET_AGE`, in mmHg.
pub const BASE_REDUCTION: usize = 2;
/// Node index: additional reduction per year of age above `RESPONSE_ONSET_AGE`, in mmHg per year.
pub const REDUCTION_PER_YEAR: usize = 3;
/// Node index: age above which the reduction grows with age, in years.
pub const RESPONSE_ONSET_AGE: usize = 4;
/// Node index: drift of the blood pressure independent of treatment, in mmHg.
pub const NATURAL_DRIFT: usize = 5;

/// Read one `Data` contextoid's payload out of a patient context.
pub fn read(context: &PatientContext, index: usize) -> Result<FloatType, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(|data| data.get_data())
        .ok_or_else(|| {
            CausalityError::MissingParameter(format!(
                "the patient context holds no Datoid at node index {index}"
            ))
        })
}

/// Build one patient, added in node-index order: node `i` holds contextoid id `i + 1`. Every
/// patient carries the same dose response; it is stronger for older patients, and that
/// heterogeneity is what makes the CATE non-trivial. Pharmacology is deliberately shallow; the
/// example is about the causal structure.
fn patient(
    id: u64,
    age: FloatType,
    baseline_bp: FloatType,
) -> Result<PatientContext, ContextIndexError> {
    let mut context = Context::with_capacity(id, "patient", 6);
    for (node_id, value) in [
        (1, age),         // AGE
        (2, baseline_bp), // BASELINE_BP
        (3, 5.0),         // BASE_REDUCTION
        (4, 0.3),         // REDUCTION_PER_YEAR
        (5, 50.0),        // RESPONSE_ONSET_AGE
        (6, 0.5),         // NATURAL_DRIFT
    ] {
        context.add_node(Contextoid::new(
            node_id,
            ContextoidType::Datoid(Data::new(node_id, value)),
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
