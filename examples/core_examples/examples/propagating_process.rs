/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, Data, Datable,
    NoSpace, NoSpaceTime, NoTime,
};
use deep_causality_core::{CausalEffect, CausalityError, PropagatingEffect, PropagatingProcess};

/// The scalar of the context's empty space and spacetime slots.
type FloatType = f64;

// Define a custom state for our process
#[derive(Debug, Clone, Default)]
struct SystemState {
    counter: i32,
    last_op: String,
}

/// The context: the process configuration as integer `Data` contextoids. The process models no
/// space and no time, so those slots hold `NoSpace`, `NoTime` and `NoSpaceTime`.
type ConfigContext = Context<Data<i32>, NoSpace<FloatType>, NoTime, NoSpaceTime<FloatType>>;

/// Node index of the multiplier contextoid.
const MULTIPLIER: usize = 0;

/// Build the configuration context: one `Data` contextoid holding the multiplier.
fn config(multiplier: i32) -> Result<ConfigContext, ContextIndexError> {
    let mut context = Context::with_capacity(1, "config", 1);
    context.add_node(Contextoid::new(
        1,
        ContextoidType::Datoid(Data::new(1, multiplier)),
    ))?;
    Ok(context)
}

/// Read one `Data` contextoid's payload out of the configuration.
fn read(context: &ConfigContext, index: usize) -> Result<i32, CausalityError> {
    context
        .get_node(index)
        .and_then(|node| node.vertex_type().dataoid())
        .map(Datable::get_data)
        .ok_or_else(|| CausalityError::Custom(format!("No Datoid at context index {index}")))
}

/// The value a process carries: its error if it failed, `ValueNotAvailable` if it holds no value.
fn value_of(
    process: &PropagatingProcess<i32, SystemState, ConfigContext>,
) -> Result<i32, CausalityError> {
    match process.outcome() {
        Ok(effect) => effect
            .as_value()
            .copied()
            .ok_or(CausalityError::ValueNotAvailable()),
        Err(error) => Err(error.clone()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- PropagatingProcess Example ---");

    // --------------------------------------------------------------------------------------------
    // ENGINEERING VALUE: Stateful System Modeling
    //
    // Real-world systems are rarely stateless. Decisions depend on history (Markovian properties)
    // and context (Configuration).
    //
    // `PropagatingProcess` extends the `Effect` concept to include:
    // 1. **State (S)**: Mutable memory that evolves with the process (e.g., counters, history).
    // 2. **Context (C)**: Read-only reference data available to all nodes (e.g., config).
    //
    // This encapsulates "Business Transaction" logic into a single, movable unit that carries
    // its own history and configuration, making it easy to serialize, replay, or debug.
    // --------------------------------------------------------------------------------------------

    // 1. Start with a stateless effect
    let initial_effect = PropagatingEffect::pure(10);

    // 2. Lift into a Stateful Process
    // We provide an initial state and context.
    let initial_state = SystemState {
        counter: 0,
        last_op: "Init".into(),
    };
    let config = config(3)?;

    let process = PropagatingProcess::with_state(initial_effect, initial_state, Some(config));

    println!(
        "Initial Process: Value={:?}, State={:?}",
        value_of(&process)?,
        process.state()
    );

    // 3. Chain Stateful Computations using inherent `bind`
    // The inherent bind allows us to access and modify state/context.
    let process_step1 = process.bind(|val, mut state, ctx| {
        // A missing value, context or multiplier becomes the error of the returned process.
        let product = val
            .into_value()
            .ok_or(CausalityError::ValueNotAvailable())
            .and_then(|v| {
                let config = ctx.as_ref().ok_or(CausalityError::MissingContext())?;
                Ok(v * read(config, MULTIPLIER)?)
            });

        // Update State
        state.counter += 1;
        state.last_op = "Multiply".into();

        // Return new process with updated value and state
        deep_causality_core::CausalEffectPropagationProcess::new(
            product.map(CausalEffect::value),
            state,
            ctx, // Pass context along
            Default::default(),
        )
    });

    println!(
        "Step 1: Value={:?}, State={:?}",
        value_of(&process_step1)?,
        process_step1.state()
    );

    let process_step2 = process_step1.bind(|val, mut state, ctx| {
        let sum = val
            .into_value()
            .map(|v| v + 5)
            .ok_or(CausalityError::ValueNotAvailable());

        // Update State
        state.counter += 1;
        state.last_op = "Add".into();

        deep_causality_core::CausalEffectPropagationProcess::new(
            sum.map(CausalEffect::value),
            state,
            ctx,
            Default::default(),
        )
    });

    println!(
        "Step 2: Value={:?}, State={:?}",
        value_of(&process_step2)?,
        process_step2.state()
    );

    Ok(())
}
