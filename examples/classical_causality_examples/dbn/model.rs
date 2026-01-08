/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::WeatherState;
use crate::{RAIN_CAUSE_ID, UMBRELLA_CAUSE_ID};
use deep_causality::{
    BaseContext, CausableGraph, Causaloid, CausaloidGraph, Contextoid, ContextoidType,
    ContextuableGraph, CurrentDataIndex, CurrentTimeIndex, Data, PropagatingEffect,
};

use crate::{RAIN_ID, TIME_ID};

/// Type aliases for cleaner code
pub type DBNCausaloid = Causaloid<WeatherState, WeatherState, (), ()>;
pub type DBNGraph = CausaloidGraph<DBNCausaloid>;

pub(crate) fn get_context() -> BaseContext {
    let mut context = BaseContext::with_capacity(1, "Umbrella World Context", 10);
    // Initial state: Day 0, Rained
    let initial_rain_datoid =
        Contextoid::new(RAIN_ID, ContextoidType::Datoid(Data::new(RAIN_ID, 1.0))); // 1.0 for Rain
    let initial_time_datoid =
        Contextoid::new(TIME_ID, ContextoidType::Datoid(Data::new(TIME_ID, 0.0))); // Day 0

    let initial_rain_id = context.add_node(initial_rain_datoid).unwrap();
    let initial_time_id = context.add_node(initial_time_datoid).unwrap();

    // Set the initial day index
    context.set_current_day_index(initial_time_id);
    // Set the initial data index
    context.set_current_data_index(initial_rain_id);

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

/// Creates the umbrella causaloid.
/// Decides whether to take umbrella based on rain probability.
/// Input: WeatherState { rain_probability, current_day }
/// Output: WeatherState (pass-through)
pub(crate) fn get_umbrella_causaloid() -> DBNCausaloid {
    let umbrella_causaloid_description =
        "Decides whether to take umbrella based on rain probability";

    fn causal_fn(input: WeatherState) -> PropagatingEffect<WeatherState> {
        // Just pass through the state; the decision is made in main.rs
        PropagatingEffect::pure(input)
    }

    Causaloid::new(UMBRELLA_CAUSE_ID, causal_fn, umbrella_causaloid_description)
}

/// Creates the rain causaloid.
/// Determines probability of rain based on previous day's rain state.
/// Input: WeatherState { rain_probability (1.0 = rained, 0.0 = no rain), current_day }
/// Output: WeatherState with new rain_probability
pub(crate) fn get_rain_causaloid() -> DBNCausaloid {
    let rain_causaloid_description = "Determines probability of rain based on previous day";

    fn causal_fn(input: WeatherState) -> PropagatingEffect<WeatherState> {
        // Simple DBN logic: if it rained yesterday (1.0), 70% chance today. Else, 20% chance.
        let prob_rain_today = if input.rain_probability == 1.0 {
            0.7
        } else {
            0.2
        };

        let output = WeatherState {
            rain_probability: prob_rain_today,
            current_day: input.current_day,
        };

        PropagatingEffect::pure(output)
    }

    Causaloid::new(RAIN_CAUSE_ID, causal_fn, rain_causaloid_description)
}
