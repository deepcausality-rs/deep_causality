/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AGE_ID, DRUG_ADMINISTERED_ID, INITIAL_BP_ID};
use deep_causality::{
    BaseContext, CausalityError, CausalityErrorEnum, Contextoid, ContextoidType, ContextuableGraph,
    Data, Datable, EffectValue, Identifiable, NumericalValue, PropagatingProcess,
};
use std::sync::{Arc, RwLock};

/// The causal logic for the drug's effect.
/// This function checks the context to see if the drug was administered and returns the effect on blood pressure.
///
/// New API Signature: fn(EffectValue<I>, S, Option<C>) -> PropagatingProcess<O, S, C>
pub(crate) fn drug_effect_logic(
    _effect: EffectValue<NumericalValue>,
    _state: (),
    context: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<NumericalValue, (), Arc<RwLock<BaseContext>>> {
    // Handle missing context
    let ctx_arc = match context {
        Some(c) => c,
        None => {
            return PropagatingProcess::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Context is missing".into(),
            )));
        }
    };

    let ctx = ctx_arc.read().unwrap();
    let mut drug_administered = false;

    // Search the context for the DRUG_ADMINISTERED_ID flag.
    for i in 0..ctx.number_of_nodes() {
        if let Some(node) = ctx.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
            && data_node.id() == DRUG_ADMINISTERED_ID
            && data_node.get_data() == 1.0
        {
            drug_administered = true;
            break;
        }
    }

    if drug_administered {
        // If the drug was given, it causes a 10-point drop in blood pressure.
        PropagatingProcess::pure(-10.0)
    } else {
        // If no drug was given, there is no effect.
        PropagatingProcess::pure(0.0)
    }
}

/// Creates a sample population of patients with different ages and blood pressures.
pub(crate) fn create_patient_population() -> Vec<BaseContext> {
    let mut population = Vec::new();
    let mut patient_id_counter = 1;

    // Tuples of (age, initial_bp)
    let patient_data = vec![
        (55.0, 145.0),
        (70.0, 150.0),
        (68.0, 155.0),
        (45.0, 130.0),
        (80.0, 160.0),
        (72.0, 148.0),
        (60.0, 140.0),
    ];

    for (age, bp) in patient_data {
        let mut context = BaseContext::with_capacity(patient_id_counter, "Patient", 5);
        patient_id_counter += 1;

        let age_datoid = Contextoid::new(
            patient_id_counter,
            ContextoidType::Datoid(Data::new(AGE_ID, age)),
        );
        context.add_node(age_datoid).unwrap();
        patient_id_counter += 1;

        let bp_datoid = Contextoid::new(
            patient_id_counter,
            ContextoidType::Datoid(Data::new(INITIAL_BP_ID, bp)),
        );
        context.add_node(bp_datoid).unwrap();
        patient_id_counter += 1;

        population.push(context);
    }

    population
}

/// Helper to extract the initial blood pressure from a patient's context.
pub(crate) fn get_patient_bp(context: &BaseContext) -> Option<f64> {
    for i in 0..context.number_of_nodes() {
        if let Some(node) = context.get_node(i)
            && let ContextoidType::Datoid(data_node) = node.vertex_type()
            && data_node.id() == INITIAL_BP_ID
        {
            return Some(data_node.get_data());
        }
    }
    None
}
