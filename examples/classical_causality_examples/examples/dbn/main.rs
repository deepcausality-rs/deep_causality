/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod model;
mod types;

use crate::types::WeatherState;
use deep_causality::*;
use deep_causality_rand::Rng;
use std::sync::{Arc, RwLock};
use std::{thread, time::Duration};

// Contextoid IDs for Rain and Time
const RAIN_ID: IdentificationValue = 0;
const TIME_ID: IdentificationValue = 1;

const RAIN_CAUSE_ID: IdentificationValue = 0;
const UMBRELLA_CAUSE_ID: IdentificationValue = 1;

fn main() {
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

    // 3. Execute the Simulation (The "Filtering" Process)
    let num_days = 2;
    let mut prev_rain_state = 1.0; // Day 0: It rained

    for day in 1..=num_days {
        thread::sleep(Duration::from_millis(50)); // Simulate time passing
        println!("\n--- Simulating Day {} ---", day);

        // Create input effect with previous day's rain state
        let input_state = WeatherState {
            rain_probability: prev_rain_state,
            current_day: day as f64,
        };
        let input_effect: PropagatingEffect<WeatherState> = PropagatingEffect::pure(input_state);

        // Evaluate the rain_causaloid for today
        let rain_res =
            causal_graph_arc.evaluate_subgraph_from_cause(RAIN_CAUSE_ID as usize, &input_effect);

        if rain_res.is_err() {
            eprintln!("Rain evaluation failed: {:?}", rain_res.error);
            continue;
        }

        let output_state = rain_res.value.into_value().unwrap_or(input_state);
        let prob_rain_today = output_state.rain_probability;

        // Sample from the probability to determine if it actually rained
        let mut rng = deep_causality_rand::rng();
        let random_sample: f64 = rng.random();
        let did_it_rain_today = random_sample < prob_rain_today;

        println!("Day {}: Probability of Rain: {:.2}", day, prob_rain_today);

        // Determine umbrella decision based on the output from umbrella causaloid
        // The umbrella causaloid gets the rain probability and logs whether to take umbrella
        let take_umbrella = prob_rain_today > 0.5;

        println!("Day {}: Take Umbrella: {}", day, take_umbrella);

        // Update context with today's actual rain state (for tracking)
        {
            let mut guard = context_arc
                .write()
                .expect("Could not acquire write lock on context");

            let rain_datoid = Data::new(RAIN_ID, if did_it_rain_today { 1.0 } else { 0.0 });
            guard
                .update_node(
                    RAIN_ID,
                    Contextoid::new(RAIN_ID, ContextoidType::Datoid(rain_datoid)),
                )
                .unwrap();
        }

        // Update prev_rain_state for next iteration
        prev_rain_state = if did_it_rain_today { 1.0 } else { 0.0 };
    }

    println!("\n--- Simulation Complete ---");
}
