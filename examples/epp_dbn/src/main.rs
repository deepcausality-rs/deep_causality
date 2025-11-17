/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod model;
mod types;

use deep_causality::*;
use deep_causality_rand::Rng;
use std::collections::HashMap;
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
    let rain_causaloid = model::get_rain_causaloid();

    // umbrella_causaloid: P(Umbrella_t | Rain_t)
    let umbrella_causaloid = model::get_umbrella_causaloid();

    // Create the CausaloidGraph
    let causal_graph = model::get_causaloid_graph(rain_causaloid, umbrella_causaloid);
    let causal_graph_arc = Arc::new(causal_graph);

    // 2. Build the Context (The Timeline)
    let context_arc = Arc::new(RwLock::new(model::get_context()));

    println!("\nInitial State (Day 0): Rained");

    // 4. Execute the Simulation (The "Filtering" Process)
    let num_days = 3;
    for day in 1..=num_days {
        thread::sleep(Duration::from_millis(50)); // Simulate time passing
        println!("\n--- Simulating Day {} ---", day);

        // Prepare input effect map for the current day
        let mut current_day_effect_map = HashMap::new();
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

        current_day_effect_map.insert(
            RAIN_ID,
            Box::new(PropagatingEffect::from_numerical(prev_rain_state)),
        );
        current_day_effect_map.insert(
            TIME_ID,
            Box::new(PropagatingEffect::from_numerical(day as f64)),
        );

        // Evaluate the rain_causaloid for today
        let rain_res = causal_graph_arc.evaluate_subgraph_from_cause(
            rain_causaloid_id as usize,
            &PropagatingEffect::from_map(current_day_effect_map),
        );
        assert!(rain_res.is_ok());

        let rain_output_map = rain_res.value.as_map().unwrap();
        let prob_rain_today = rain_output_map
            .get(&RAIN_ID)
            .unwrap()
            .value
            .as_numerical()
            .unwrap();

        let mut rng = deep_causality_rand::rng();
        let random_sample: f64 = rng.random(); // Generates a float between 0.0 and 1.0

        // println!("Random Sample: {}", random_sample);
        let did_it_rain_today = random_sample < *prob_rain_today;

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
        rain_datoid.set_data(*prob_rain_today);

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
        let umbrella_res = causal_graph_arc
            .evaluate_subgraph_from_cause(umbrella_causaloid_id.try_into().unwrap(), &rain_res);

        // Uncomment to see the full output of the umbrella_causaloid
        // dbg!(&umbrella_res);

        let umbrella_output_map = umbrella_res.value.as_map().unwrap();
        let take_umbrella = umbrella_output_map
            .get(&umbrella_causaloid_id)
            .unwrap()
            .value
            .as_bool()
            .unwrap();

        println!("Day {}: Take Umbrella: {}", day, take_umbrella);
    }
}
