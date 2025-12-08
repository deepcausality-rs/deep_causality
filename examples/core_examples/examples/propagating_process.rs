/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{EffectValue, PropagatingEffectWitness, PropagatingProcess};
use deep_causality_haft::Applicative;

// Define a custom state for our process
#[derive(Debug, Clone, Default)]
struct SystemState {
    counter: i32,
    last_op: String,
}

// Define a context (e.g., configuration)
#[derive(Debug, Clone)]
struct Config {
    multiplier: i32,
}

fn main() {
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
    let initial_effect = PropagatingEffectWitness::pure(10);

    // 2. Lift into a Stateful Process
    // We provide an initial state and context.
    let initial_state = SystemState {
        counter: 0,
        last_op: "Init".into(),
    };
    let config = Config { multiplier: 3 };

    let process = PropagatingProcess::with_state(initial_effect, initial_state, Some(config));

    println!(
        "Initial Process: Value={:?}, State={:?}",
        process.value, process.state
    );

    // 3. Chain Stateful Computations using inherent `bind`
    // The inherent bind allows us to access and modify state/context.
    let process_step1 = process.bind(|val, mut state, ctx| {
        let v = val.into_value().unwrap_or(0);
        let mult = ctx.as_ref().map(|c| c.multiplier).unwrap_or(1);

        // Update State
        state.counter += 1;
        state.last_op = "Multiply".into();

        // Return new process with updated value and state
        deep_causality_core::CausalEffectPropagationProcess {
            value: EffectValue::Value(v * mult),
            state,
            context: ctx, // Pass context along
            error: None,
            logs: Default::default(),
        }
    });

    println!(
        "Step 1: Value={:?}, State={:?}",
        process_step1.value, process_step1.state
    );

    let process_step2 = process_step1.bind(|val, mut state, ctx| {
        let v = val.into_value().unwrap_or(0);

        // Update State
        state.counter += 1;
        state.last_op = "Add".into();

        deep_causality_core::CausalEffectPropagationProcess {
            value: EffectValue::Value(v + 5),
            state,
            context: ctx,
            error: None,
            logs: Default::default(),
        }
    });

    println!(
        "Step 2: Value={:?}, State={:?}",
        process_step2.value, process_step2.state
    );
}
