/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    CausalEffect, CausalityError, CausalityErrorEnum, PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift};
use std::sync::{Arc, RwLock};

/// The scalar this example works in. Declared here, per example, so changing the shared alias in
/// `deep_causality_core` cannot silently reconfigure every example that names one.
pub type FloatType = f64;

/// The world one patient is reasoned about in: numeric data only, no space and no time.
pub(crate) type PatientContext =
    Context<Data<FloatType>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Contextoid id: patient age, years.
pub(crate) const AGE: ContextoidId = 1;
/// Contextoid id: initial blood pressure, BP points.
pub(crate) const INITIAL_BP: ContextoidId = 2;
/// Contextoid id: the BP change the drug produces when administered, BP points.
const DOSE: ContextoidId = 3;
/// Contextoid id: the treatment assignment of one trial arm, [`ADMINISTERED`] when the drug is
/// administered and [`NOT_ADMINISTERED`] when it is not.
const DRUG_ADMINISTERED: ContextoidId = 4;

/// Treatment assignment of an arm in which the drug is administered.
const ADMINISTERED: FloatType = const_scalar_from_int!(FloatType, 1);
/// Treatment assignment of an arm in which the drug is not administered.
const NOT_ADMINISTERED: FloatType = const_scalar_from_int!(FloatType, 0);
/// Midpoint between [`NOT_ADMINISTERED`] and [`ADMINISTERED`]: an assignment above it reads as
/// administered, so the decision does not rest on an exact float equality.
const ASSIGNMENT_MIDPOINT: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The causal logic for the drug's effect.
/// This function reads the treatment assignment and the dose from the context and returns the
/// effect on blood pressure.
///
/// New API Signature: fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
pub(crate) fn drug_effect_logic(
    _effect: CausalEffect<FloatType>,
    _state: (),
    context: Option<Arc<RwLock<PatientContext>>>,
) -> PropagatingProcess<FloatType, (), Arc<RwLock<PatientContext>>> {
    // Handle missing context
    let ctx_arc = match context {
        Some(c) => c,
        None => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context is missing".into(),
            )));
        }
    };

    let ctx = match ctx_arc.read() {
        Ok(guard) => guard,
        Err(_) => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context lock is poisoned".into(),
            )));
        }
    };

    let drug_effect = read(&ctx, DRUG_ADMINISTERED).and_then(|drug_administered| {
        if drug_administered > ASSIGNMENT_MIDPOINT {
            // If the drug was given, blood pressure changes by the patient's dose.
            read(&ctx, DOSE)
        } else {
            // If no drug was given, there is no effect.
            Ok(lift::<FloatType>(0.0))
        }
    });

    match drug_effect {
        Ok(effect) => PropagatingProcess::pure(effect),
        Err(error) => PropagatingProcess::from_error(error),
    }
}

/// The value an evaluation carries, or the error that ended it.
pub(crate) fn value_of(effect: &PropagatingEffect<FloatType>) -> Result<FloatType, CausalityError> {
    match effect.error() {
        Some(error) => Err(error.clone()),
        None => effect
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
}

/// Creates a sample population of patients with different ages and blood pressures. The drug
/// lowers every patient's blood pressure by 10 points when administered.
pub(crate) fn create_patient_population() -> Result<Vec<PatientContext>, ContextIndexError> {
    // Tuples of (age, initial_bp)
    let patient_data = [
        (55.0, 145.0),
        (70.0, 150.0),
        (68.0, 155.0),
        (45.0, 130.0),
        (80.0, 160.0),
        (72.0, 148.0),
        (60.0, 140.0),
    ];

    (1..)
        .zip(patient_data)
        .map(|(patient_id, (age, bp))| {
            let facts = [
                (AGE, lift(age)),
                (INITIAL_BP, lift(bp)),
                (DOSE, lift(-10.0)),
            ];
            let mut context = Context::with_capacity(patient_id, "Patient", facts.len());
            for (id, value) in facts {
                add_datoid(&mut context, id, value)?;
            }
            Ok(context)
        })
        .collect()
}

/// The patient's world in one arm of the trial: the patient's context plus the treatment
/// assignment, [`ADMINISTERED`] when the drug is administered and [`NOT_ADMINISTERED`] when it is
/// not.
pub(crate) fn arm(
    patient: &PatientContext,
    drug_administered: bool,
) -> Result<PatientContext, ContextIndexError> {
    let mut context = patient.clone();
    let assignment = if drug_administered {
        ADMINISTERED
    } else {
        NOT_ADMINISTERED
    };
    add_datoid(&mut context, DRUG_ADMINISTERED, assignment)?;
    Ok(context)
}

fn add_datoid(
    context: &mut PatientContext,
    id: ContextoidId,
    value: FloatType,
) -> Result<(), ContextIndexError> {
    context.add_node(Contextoid::new(
        id,
        ContextoidType::Datoid(Data::new(id, value)),
    ))?;
    Ok(())
}

/// Read the `Data` contextoid with contextoid id `id` out of a patient world.
pub(crate) fn read(
    context: &PatientContext,
    id: ContextoidId,
) -> Result<FloatType, CausalityError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| CausalityError::ModelError(format!("no Datoid with contextoid id {id}")))
}
