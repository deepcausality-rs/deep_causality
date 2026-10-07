/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::WeatherState;
use crate::{RAIN_CAUSE_ID, UMBRELLA_CAUSE_ID};
use deep_causality::{CausableGraph, CausalityError, Causaloid, CausaloidGraph, PropagatingEffect};
use deep_causality_context::{
    BaseContext, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph,
    CurrentDataIndex, CurrentTimeIndex, Data,
};
use std::error::Error;

use crate::{RAIN_ID, TIME_ID};

/// Type aliases for cleaner code
pub type DBNCausaloid = Causaloid<WeatherState, WeatherState, (), ()>;
pub type DBNGraph = CausaloidGraph<DBNCausaloid>;

pub(crate) fn get_context() -> Result<BaseContext, ContextIndexError> {
    // Initial state: Day 0, Rained
    let rained = 1.0; // 1.0 for Rain
    let day = 0.0; // Day 0
    let initial_nodes = [
        Contextoid::new(RAIN_ID, ContextoidType::Datoid(Data::new(RAIN_ID, rained))),
        Contextoid::new(TIME_ID, ContextoidType::Datoid(Data::new(TIME_ID, day))),
    ];
    let mut context = BaseContext::with_capacity(1, "Umbrella World Context", initial_nodes.len());

    let [initial_rain_datoid, initial_time_datoid] = initial_nodes;
    let initial_rain_index = context.add_node(initial_rain_datoid)?;
    let initial_time_index = context.add_node(initial_time_datoid)?;

    // Set the initial day index
    context.set_current_day_index(initial_time_index);
    // Set the initial data index
    context.set_current_data_index(initial_rain_index);

    println!("Initial State (Day 0): Rained: {rained}");
    println!("Initial State (Day 0): Time: {day}");

    Ok(context)
}

pub(crate) fn get_causaloid_graph(
    rain_causaloid: DBNCausaloid,
    umbrella_causaloid: DBNCausaloid,
) -> Result<DBNGraph, Box<dyn Error>> {
    let mut causaloid_graph = CausaloidGraph::new(0);
    let rain_idx = causaloid_graph.add_causaloid(rain_causaloid)?;
    let umbrella_idx = causaloid_graph.add_causaloid(umbrella_causaloid)?;
    causaloid_graph.add_edge(rain_idx, umbrella_idx)?;
    causaloid_graph.freeze();

    Ok(causaloid_graph)
}

/// The value an evaluation carries, or the error that ended it.
pub(crate) fn value_of<T: Clone>(effect: &PropagatingEffect<T>) -> Result<T, CausalityError> {
    match effect.error() {
        Some(error) => Err(error.clone()),
        None => effect
            .value_cloned()
            .ok_or(CausalityError::ValueNotAvailable()),
    }
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
