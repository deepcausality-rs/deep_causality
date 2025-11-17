/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::{RCMCausalGraph, RRMCausaloid};
use crate::{DRUG_ADMINISTERED_ID, INITIAL_BP_ID};
use deep_causality::{
    BaseCausaloid, CausableGraph, CausalEffectLog, CausalFnOutput, CausalityError, CausaloidGraph,
    EffectValue, PropagatingEffect,
};
use std::collections::HashMap;

pub(crate) fn get_causaloid_graph() -> RCMCausalGraph {
    let drug_effect_causaloid = get_drug_effect_causaloid();
    // final_bp_causaloid
    let final_bp_causaloid = get_final_bp_causaloid();

    // Create the CausaloidGraph
    let mut causaloid_graph = CausaloidGraph::new(0);

    // Add the Drug effect causaloid
    let drug_effect_idx = causaloid_graph
        .add_causaloid(drug_effect_causaloid)
        .unwrap();

    // Add the blood pressure causaloid
    let final_bp_idx = causaloid_graph.add_causaloid(final_bp_causaloid).unwrap();
    causaloid_graph
        .add_edge(drug_effect_idx, final_bp_idx)
        .unwrap();

    // Freeze the graph to ensure high performance reasoning
    causaloid_graph.freeze();

    causaloid_graph
}

fn get_drug_effect_causaloid() -> RRMCausaloid {
    let drug_effect_causaloid_id = 1;
    let drug_effect_causaloid_description = "Determines drug effect based on administration";

    // Function signature: CausalFn<I: IntoEffectValue, O: IntoEffectValue> = fn(value: I) -> Result<CausalFnOutput<O>, CausalityError>
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();
        let drug_effect_causaloid_id = 1;

        let (is_drug_administered, initial_bp) = match &effect {
            EffectValue::Map(map) => {
                let is_drug_administered = map
                    .get(&DRUG_ADMINISTERED_ID)
                    .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                    .map(|val| *val == 1.0) // Interpret 1.0 as true, 0.0 as false
                    .unwrap_or(false); // Default to false if not found or not numerical

                let initial_bp = map
                    .get(&INITIAL_BP_ID)
                    .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                    .ok_or_else(|| {
                        CausalityError(
                            "Initial BP not found in effect map for drug_effect_causaloid".into(),
                        )
                    })?;
                (is_drug_administered, initial_bp)
            }
            _ => {
                return Err(CausalityError(format!(
                    "Expected Map EffectValue as input, but got {}",
                    &effect
                )));
            }
        };

        log.add_entry(format!("Drug is administered {}", is_drug_administered).as_str());

        let drug_effect_value = if is_drug_administered { -10.0 } else { 0.0 };
        log.add_entry(format!("Drug effect is estimated as {}", drug_effect_value).as_str());

        let mut result_map = HashMap::new();
        result_map.insert(
            INITIAL_BP_ID,
            Box::new(PropagatingEffect::from_numerical(*initial_bp)),
        );
        result_map.insert(
            DRUG_ADMINISTERED_ID,
            Box::new(PropagatingEffect::from_numerical(
                is_drug_administered as u64 as f64,
            )),
        ); // Store as numerical value to return 
        result_map.insert(
            drug_effect_causaloid_id,
            Box::new(PropagatingEffect::from_numerical(drug_effect_value)),
        ); // Store the calculated drug effect

        Ok(CausalFnOutput::new(EffectValue::Map(result_map), log))
    }

    BaseCausaloid::new(
        drug_effect_causaloid_id,
        causal_fn,
        drug_effect_causaloid_description,
    )
}

fn get_final_bp_causaloid() -> RRMCausaloid {
    let final_bp_causaloid_id = 2;
    let final_bp_causaloid_description = "Calculates final blood pressure";

    // Function signature: CausalFn<I: IntoEffectValue, O: IntoEffectValue> = fn(value: I) -> Result<CausalFnOutput<O>, CausalityError>
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();
        let drug_effect_id = 1;

        let initial_bp = match &effect {
            EffectValue::Map(map) => {
                // Extract the initial blood pressure from the incoming EffectValue Map
                map.get(&INITIAL_BP_ID)
                    .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                    .ok_or_else(|| {
                        CausalityError(
                            "Initial BP not found in effect map for final_bp_causaloid".into(),
                        )
                    })?
            }
            _ => {
                return Err(CausalityError(
                    "Expected Map effect for final BP calculation".into(),
                ));
            }
        };
        log.add_entry(format!("Initial Blood pressure is: {}", initial_bp).as_str());

        let drug_effect = match &effect {
            EffectValue::Map(map) => map
                .get(&drug_effect_id)
                .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                .ok_or_else(|| {
                    CausalityError(
                        "Drug effect value not found in effect map for final_bp_causaloid".into(),
                    )
                })?,
            _ => {
                return Err(CausalityError(format!(
                    "Expected Map EffectValue as input, but got {}",
                    &effect
                )));
            }
        };
        log.add_entry(format!("Drug effect is: {}", initial_bp).as_str());

        let combined = initial_bp + drug_effect;
        log.add_entry(
            format!(" Blood pressure after drug administration is: {}", combined).as_str(),
        );

        // Return result and log
        Ok(CausalFnOutput::new(EffectValue::Numerical(combined), log))
    }

    BaseCausaloid::new(
        final_bp_causaloid_id,
        causal_fn,
        final_bp_causaloid_description,
    )
}
