/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// ============================================================================
// Adjunction: Global Configuration Access
// ============================================================================

// ENGINEERING VALUE:
// Adjunctions describe a relationship between two functors: Left (L) and Right (R).
// A classic example is (Writer -| Reader) or (Product -| Exponential).
//
// Here we demonstrate the "Reader Adjunction" pattern (Product -| Reader).
// It allows us to convert a function that takes a Context (Reader) into a
// simple value pair (Product), and vice versa.
//
// Practical Use: "Currying" configuration.
// Instead of passing the configuration to every function, we can "adjunct" the function
// to lock in the config, producing a standalone value.
//
// The configuration is the Reader's environment, and it is a `Context`: one `Data<String>`
// node per setting, read by contextoid id.

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Data,
    NoSpace, NoSpaceTime, NoTime,
};

/// The Reader environment: the configuration a reader runs against. It holds settings only, so
/// the spatial, temporal and spacetime slots are empty.
type ConfigContext = Context<Data<String>, NoSpace<f64>, NoTime, NoSpaceTime<f64>>;

/// Contextoid id: the API key.
const API_KEY: ContextoidId = 1;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // Scenario: We have a function that fetches data given a ConfigContext and an ID.
    // fetch_data: (ConfigContext, i32) -> Result<String, ContextIndexError>
    let fetch_data = |cfg: ConfigContext, id: i32| -> Result<String, ContextIndexError> {
        Ok(format!(
            "Data for ID {} using Key {}",
            id,
            read(&cfg, API_KEY)?
        ))
    };

    // We want to "bake in" the ID first, creating a reusable reader that needs only the
    // ConfigContext. That is the left adjunct: (ConfigContext, A) -> B becomes
    // A -> (ConfigContext -> B).
    let id_to_fetch = 42;
    let reader = ConfigAdjunction::left_adjunct(fetch_data, id_to_fetch);

    // Now 'reader' is a function ConfigContext -> Result<String, ContextIndexError>.
    // We can pass this 'reader' around to a component that holds the ConfigContext.

    let my_config = config("SECRET_KEY")?;

    let result = reader(my_config.clone())?;
    print_result(&result);

    // The right adjunct goes back: from "an id gives a reader" to "the configuration and an id
    // give the value". The round trip returns what the reader returned.
    let fetch_again =
        ConfigAdjunction::right_adjunct(|id| ConfigAdjunction::left_adjunct(fetch_data, id));
    let round_trip = fetch_again(my_config, id_to_fetch)?;
    print_round_trip(&round_trip);
    Ok(())
}

/// The configuration as a `Context`: the API key as its one `Data<String>` node.
fn config(api_key: &str) -> Result<ConfigContext, ContextIndexError> {
    let facts = [(API_KEY, api_key.to_string())];

    let mut context = Context::with_capacity(1, "config", facts.len());
    for (id, value) in facts {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))?;
    }
    Ok(context)
}

/// Read one setting out of the configuration.
fn read(context: &ConfigContext, id: ContextoidId) -> Result<String, ContextIndexError> {
    context
        .get_data_by_id(id)
        .ok_or_else(|| ContextIndexError::new(format!("no setting with contextoid id {id}")))
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Adjunction Pattern ===\n");
    println!("--- Reader/Writer Duality ---");
    println!("(Demonstration of concept)");
}

fn print_result(result: &str) {
    println!("Adjunction Result: {result}");
}

fn print_round_trip(result: &str) {
    println!("Right adjunct round trip: {result}");
}

/// The product ⊣ reader adjunction over the configuration, built by hand: a function of the
/// configuration and a value corresponds to a function from the value to a reader of the
/// configuration.
struct ConfigAdjunction;

impl ConfigAdjunction {
    /// Left adjunct: `(ConfigContext, A) -> B` becomes `A -> (ConfigContext -> B)`; this returns
    /// the reader for `a`.
    fn left_adjunct<A, B, F>(f: F, a: A) -> impl Fn(ConfigContext) -> B
    where
        A: Clone,
        F: Fn(ConfigContext, A) -> B,
    {
        move |cfg: ConfigContext| f(cfg, a.clone())
    }

    /// Right adjunct: `A -> (ConfigContext -> B)` becomes `(ConfigContext, A) -> B`.
    fn right_adjunct<A, B, R, G>(g: G) -> impl Fn(ConfigContext, A) -> B
    where
        G: Fn(A) -> R,
        R: Fn(ConfigContext) -> B,
    {
        move |cfg: ConfigContext, a: A| g(a)(cfg)
    }
}
