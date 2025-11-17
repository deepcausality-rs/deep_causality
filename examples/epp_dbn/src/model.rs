/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::{DBNCausaloid, DBNGraph};
use crate::{RAIN_CAUSE_ID, RAIN_ID, TIME_ID, UMBRELLA_CAUSE_ID};
use deep_causality::{
    BaseCausaloid, BaseContext, CausableGraph, CausalEffectLog, CausalFnOutput, CausalityError,
    CausaloidGraph, Contextoid, ContextoidType, ContextuableGraph, CurrentDataIndex,
    CurrentTimeIndex, Data, EffectValue, PropagatingEffect,
};
use std::collections::HashMap;

pub(crate) fn get_context() -> BaseContext {
    let mut context = BaseContext::with_capacity(1, "Umbrella World Context", 10);
    // Initial state: Day 0, Rained
    let initial_rain_datoid =
        Contextoid::new(RAIN_ID, ContextoidType::Datoid(Data::new(RAIN_ID, 1.0))); // 1.0 for Rain
    let initial_time_datoid =
        Contextoid::new(TIME_ID, ContextoidType::Datoid(Data::new(TIME_ID, 0.0))); // Day 0 for no rain

    let initial_rain_id = context.add_node(initial_rain_datoid).unwrap();

    let initial_time_id = context.add_node(initial_time_datoid).unwrap();

    // These are index accesors for the context imported from the CurrentDataIndex and
    // CurrentTimeIndex extensions to simplify current/previous time and data access

    // Set the initial day index
    context.set_current_day_index(initial_time_id);
    // Set the initial data index to
    context.set_current_data_index(initial_rain_id);

    // There area mapped to the constants RAIN_ID and TIME_ID at the top for global access.
    println!("Initial State (Day 0): Rained: {initial_rain_id}");
    println!("Initial State (Day 0): Time: {initial_time_id}");

    context
}

pub(crate) fn get_causaloid_graph(
    rain_causaloid: DBNCausaloid,
    umbrella_causaloid: DBNCausaloid,
) -> DBNGraph {
    let mut causaloid_graph = CausaloidGraph::new(0);
    let rain_idx = causaloid_graph.add_causaloid(rain_causaloid).unwrap();
    let umbrella_idx = causaloid_graph.add_causaloid(umbrella_causaloid).unwrap();
    causaloid_graph.add_edge(rain_idx, umbrella_idx).unwrap();
    causaloid_graph.freeze();

    causaloid_graph
}

pub(crate) fn get_umbrella_causaloid() -> DBNCausaloid {
    let umbrella_causaloid_description =
        "Decides whether to take umbrella based on rain probability";

    // Function signature: CausalFn<I: IntoEffectValue, O: IntoEffectValue> = fn(value: I) -> Result<CausalFnOutput<O>, CausalityError>
    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let mut log = CausalEffectLog::new();

        let prob_rain_today = match &effect {
            EffectValue::Map(map) => map
                .get(&RAIN_ID)
                .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                .ok_or_else(|| {
                    CausalityError("Rain probability not found in input effect map".into())
                })?,
            _ => {
                return Err(CausalityError(
                    "Expected Map effect for umbrella_causaloid".into(),
                ));
            }
        };
        log.add_entry(format!("Probability of rain today: {}", prob_rain_today).as_str());

        let take_umbrella = *prob_rain_today > 0.5;

        let mut result_map = HashMap::new();
        result_map.insert(
            RAIN_ID,
            Box::new(PropagatingEffect::from_numerical(*prob_rain_today)),
        );
        result_map.insert(
            UMBRELLA_CAUSE_ID,
            Box::new(PropagatingEffect::from_boolean(take_umbrella)),
        );

        // Return result effect and log
        Ok(CausalFnOutput::new(EffectValue::Map(result_map), log))
    }

    BaseCausaloid::new(UMBRELLA_CAUSE_ID, causal_fn, umbrella_causaloid_description)
}

pub(crate) fn get_rain_causaloid() -> DBNCausaloid {
    // rain_causaloid: P(Rain_t | Rain_t-1)
    let rain_causaloid_description = "Determines probability of rain based on previous day";

    fn causal_fn(effect: EffectValue) -> Result<CausalFnOutput<EffectValue>, CausalityError> {
        let log = CausalEffectLog::new();

        let (prev_rain_val, current_time_val) = match &effect {
            EffectValue::Map(map) => {
                let prev_rain = map
                    .get(&RAIN_ID)
                    .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                    .ok_or_else(|| {
                        CausalityError("Previous rain value not found in effect map".into())
                    })?;
                let current_time = map
                    .get(&TIME_ID)
                    .and_then(|boxed_effect| boxed_effect.value.as_numerical())
                    .ok_or_else(|| CausalityError("Current time not found in effect map".into()))?;
                (prev_rain, current_time)
            }
            _ => {
                return Err(CausalityError(
                    "Expected Map effect for rain_causaloid".into(),
                ));
            }
        };

        // Simple DBN logic: if it rained yesterday (1.0), 70% chance today. Else, 20% chance.
        let prob_rain_today = if *prev_rain_val == 1.0 { 0.7 } else { 0.2 };

        let mut result_map = HashMap::new();
        result_map.insert(
            RAIN_ID,
            Box::new(PropagatingEffect::from_numerical(prob_rain_today)),
        );
        result_map.insert(
            TIME_ID,
            Box::new(PropagatingEffect::from_numerical(*current_time_val)),
        );

        // Return result effect and log
        Ok(CausalFnOutput::new(EffectValue::Map(result_map), log))
    }

    BaseCausaloid::new(RAIN_CAUSE_ID, causal_fn, rain_causaloid_description)
}
