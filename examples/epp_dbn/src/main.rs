/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use rand::Rng;
use std::sync::{Arc, RwLock};
use std::{thread, time::Duration};

// Contextoid IDs for Rain and Time
const RAIN_ID: IdentificationValue = 0;
const TIME_ID: IdentificationValue = 1;

const RAIN_CAUSE_ID: IdentificationValue = 0;
const UMBRELLA_CAUSE_ID: IdentificationValue = 1;

fn main() {
    let umbrella_causaloid_id = UMBRELLA_CAUSE_ID;
    let rain_causaloid_id = RAIN_CAUSE_ID;

    println!("\n--- DBN Example: Umbrella World ---");

    // 1. Define the Causal Logic (Causaloids)

    // rain_causaloid: P(Rain_t | Rain_t-1)
    let rain_causaloid = get_rain_causaloid();

    // umbrella_causaloid: P(Umbrella_t | Rain_t)
    let umbrella_causaloid = get_umbrella_causaloid();

    // Create the CausaloidGraph
    let causal_graph = get_causaloid_graph(rain_causaloid, umbrella_causaloid);
    let causal_graph_arc = Arc::new(causal_graph);

    // 2. Build the Context (The Timeline)
    let context_arc = Arc::new(RwLock::new(get_context()));

    println!("\nInitial State (Day 0): Rained");

    // 4. Execute the Simulation (The "Filtering" Process)
    let num_days = 3;
    for day in 1..=num_days {
        thread::sleep(Duration::from_millis(50)); // Simulate time passing
        println!("\n--- Simulating Day {} ---", day);

        // Prepare input effect map for the current day
        let mut current_day_effect_map = PropagatingEffect::new_map();
        let prev_rain_state = context_arc
            .read()
            .unwrap()
            .get_node(RAIN_ID as usize)
            .expect("Rain node not found in context")
            .vertex_type()
            .dataoid()
            .expect("Rain node has no data")
            .clone()
            .get_data() as f64;

        current_day_effect_map.insert(RAIN_ID, PropagatingEffect::Numerical(prev_rain_state));
        current_day_effect_map.insert(TIME_ID, PropagatingEffect::Numerical(day as f64));

        // Evaluate the rain_causaloid for today
        let rain_res = causal_graph_arc
            .evaluate_subgraph_from_cause(rain_causaloid_id as usize, &current_day_effect_map);
        let rain_output_map = rain_res.unwrap();
        let prob_rain_today = rain_output_map.get_numerical_from_map(RAIN_ID).unwrap();

        let mut rng = rand::rng();
        let random_sample: f64 = rng.random(); // Generates a float between 0.0 and 1.0

        // println!("Random Sample: {}", random_sample);
        let did_it_rain_today = random_sample < prob_rain_today;

        // Update the context with today's sampled rain probability state (for next iteration)
        let mut rain_datoid = *context_arc
            .read()
            .unwrap()
            .get_node(RAIN_ID as usize)
            .expect("Rain node not found in context")
            .vertex_type()
            .dataoid()
            .expect("Rain node has no data"); // Clone to get an owned mutable copy

        // Update the data
        rain_datoid.set_data(did_it_rain_today as u64 as f64);

        // Update the context with today's rain probability (for next iteration)
        rain_datoid.set_data(prob_rain_today);

        let mut guard = context_arc
            .write()
            .expect("Could not acquire write lock on context");

        guard
            .update_node(
                RAIN_ID,
                Contextoid::new(RAIN_ID, ContextoidType::Datoid(rain_datoid)),
            )
            .unwrap();

        drop(guard);

        println!("Day {}: Probability of Rain: {:.2}", day, prob_rain_today);

        // Evaluate the umbrella_causaloid for today
        let umbrella_res = causal_graph_arc.evaluate_subgraph_from_cause(
            umbrella_causaloid_id.try_into().unwrap(),
            &rain_output_map,
        );

        // Uncomment to see the output of the umbrella_causaloid
        // dbg!(&umbrella_res);

        let umbrella_output_map = umbrella_res.unwrap();
        let take_umbrella = umbrella_output_map
            .get_deterministic_from_map(umbrella_causaloid_id)
            .unwrap();

        println!("Day {}: Take Umbrella: {}", day, take_umbrella);
    }
}

fn get_context() -> BaseContext {
    let mut context = BaseContext::with_capacity(1, "Umbrella World Context", 10);
    // Initial state: Day 0, Rained
    let initial_rain_datoid =
        Contextoid::new(RAIN_ID, ContextoidType::Datoid(Data::new(RAIN_ID, 1.0))); // 1.0 for Rained
    let initial_time_datoid =
        Contextoid::new(TIME_ID, ContextoidType::Datoid(Data::new(TIME_ID, 0.0))); // Day 0 for no rain

    let initial_rain_id = context.add_node(initial_rain_datoid).unwrap();

    let initial_time_id = context.add_node(initial_time_datoid).unwrap();

    // These are index accesors for the context to simplify current/previous time and data access

    // Set the initial day index
    context.set_current_day_index(initial_time_id);
    // Set the initial data index to
    context.set_current_data_index(initial_rain_id);

    // There area mapped to the constants RAIN_ID and TIME_ID at the top for global access.
    println!("Initial State (Day 0): Rained: {initial_rain_id}");
    println!("Initial State (Day 0): Time: {initial_time_id}");

    context
}

fn get_causaloid_graph(
    rain_causaloid: BaseCausaloid,
    umbrella_causaloid: BaseCausaloid,
) -> CausaloidGraph<BaseCausaloid> {
    let mut causaloid_graph = CausaloidGraph::new(0);
    let rain_idx = causaloid_graph.add_causaloid(rain_causaloid).unwrap();
    let umbrella_idx = causaloid_graph.add_causaloid(umbrella_causaloid).unwrap();
    causaloid_graph.add_edge(rain_idx, umbrella_idx).unwrap();
    causaloid_graph.freeze();

    causaloid_graph
}

fn get_umbrella_causaloid() -> BaseCausaloid {
    let umbrella_causaloid_id = UMBRELLA_CAUSE_ID;
    let umbrella_causaloid_description =
        "Decides whether to take umbrella based on rain probability";

    let umbrella_causaloid_fn =
        |effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
            let umbrella_causaloid_id = UMBRELLA_CAUSE_ID;

            let prob_rain_today = match effect {
                PropagatingEffect::Map(map) => {
                    let rain_id = RAIN_ID;
                    map.get(&rain_id)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError("Rain probability not found in effect map".into())
                        })?
                }
                _ => {
                    return Err(CausalityError(
                        "Expected Map effect for umbrella_causaloid".into(),
                    ));
                }
            };

            let take_umbrella = prob_rain_today > 0.5;

            let mut result_map = PropagatingEffect::new_map();
            result_map.insert(RAIN_ID, PropagatingEffect::Numerical(prob_rain_today)); // Pass through rain prob
            result_map.insert(
                umbrella_causaloid_id,
                PropagatingEffect::Deterministic(take_umbrella),
            );
            Ok(result_map)
        };

    BaseCausaloid::new(
        umbrella_causaloid_id,
        umbrella_causaloid_fn,
        umbrella_causaloid_description,
    )
}

fn get_rain_causaloid() -> BaseCausaloid {
    // rain_causaloid: P(Rain_t | Rain_t-1)
    let rain_causaloid_id = RAIN_CAUSE_ID;
    let rain_causaloid_description = "Determines probability of rain based on previous day";
    let rain_causaloid_fn =
        |effect: &PropagatingEffect| -> Result<PropagatingEffect, CausalityError> {
            let (prev_rain_val, current_time_val) = match effect {
                PropagatingEffect::Map(map) => {
                    let prev_rain = map
                        .get(&RAIN_ID)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError("Previous rain value not found in effect map".into())
                        })?;
                    let current_time = map
                        .get(&TIME_ID)
                        .and_then(|boxed_effect| boxed_effect.as_numerical())
                        .ok_or_else(|| {
                            CausalityError("Current time not found in effect map".into())
                        })?;
                    (prev_rain, current_time)
                }
                _ => {
                    return Err(CausalityError(
                        "Expected Map effect for rain_causaloid".into(),
                    ));
                }
            };

            // Simple DBN logic: if it rained yesterday (1.0), 70% chance today. Else, 20% chance.
            let prob_rain_today = if prev_rain_val == 1.0 { 0.7 } else { 0.2 };

            let mut result_map = PropagatingEffect::new_map();
            result_map.insert(RAIN_ID, PropagatingEffect::Numerical(prob_rain_today));
            result_map.insert(TIME_ID, PropagatingEffect::Numerical(current_time_val));
            Ok(result_map)
        };

    BaseCausaloid::new(
        rain_causaloid_id,
        rain_causaloid_fn,
        rain_causaloid_description,
    )
}
