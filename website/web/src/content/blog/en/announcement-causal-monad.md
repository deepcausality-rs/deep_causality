---
title: "DeepCausality Introduces the Causal Monad"
description: "This post introduces the new Causal Monad in DeepCausality, enabling clean, composable, and robust causal inference chains."
date: 2025-12-12
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

We are excited to introduce the **Causal Monad**, a powerful new core feature in DeepCausality that radically simplifies how you write complex causal logic.

If you have ever built a complex data processing pipeline, you know the pain: you start with a simple value, but soon you are juggling error handling, audit logging, configuration contexts, and mutable state across a dozen function calls buried under boilerplate.

The Causal Monad solves this by abstracting away the plumbing. It allows you to chain causal operations together while automatically handling:

*   **Error Propagation**: Like Rust's `Result`, it short-circuits on failure.
*   **Audit Logging**: Every step is automatically recorded for explainability.
*   **Context & State**: Configuration and state are carried through the chain implicitly.

## 💡 The Problem: Boilerplate Overload

Imagine you are building a system to monitor engine temperature. You need to:
1.  Read the sensor.
2.  Validate the reading.
3.  Check if it exceeds a threshold (from a config context).
4.  Update a moving average (state).
5.  Log every step for debugging.

Without a monad, your function signatures become messy, constantly passing around `&Config`, `&mut State`, and `&mut Vec<Log>`. You also have to manually check `if result.is_err() { return ... }` at every step.

## ⚡ The Solution: `PropagatingEffect` and `PropagatingProcess`

DeepCausality provides two key types to clean this up, functioning very similar to Rust's standard `Option` and `Result` types.

### 1. Stateless Logic: `PropagatingEffect`

For pure logic where you don't need to maintain state between steps, use `PropagatingEffect`. It behaves like a supercharged `Result` type that keeps a log of what happened.

```rust
use deep_causality_core::{PropagatingEffect, EffectValue};

fn main() {
    // Start with a value
    let result = PropagatingEffect::pure(10)
        // Chain a calculation. 
        // bind passes: (value, state, context)
        .bind(|val, _, _| {
            // Unpack the EffectValue (similar to Option::unwrap)
            let v = val.into_value().expect("Expected a value");
            PropagatingEffect::pure(v + 5) 
        })
        // Chain another step
        .bind(|val, _, _| {
            let v = val.into_value().expect("Expected a value");
            
            if v > 20 {
                PropagatingEffect::pure(v)
            } else {
                 // Return an error (short-circuits future steps)
                 PropagatingEffect::from_error(
                     deep_causality_core::CausalityError::new(
                         deep_causality_core::CausalityErrorEnum::Custom("Value too low".into())
                     )
                 )
            }
        });
        
    // If an error occurred, result.error is set.
    // If successful, result.value contains the final data.
    // In ALL cases, result.logs contains the history.
}
```

### 2. Stateful Logic: `PropagatingProcess`

For Markovian processes where you need to track state (like a running counter or risk score) and access configuration, use `PropagatingProcess`.

```rust
use deep_causality_core::PropagatingProcess;

// Your custom state
#[derive(Clone, Default)]
struct SystemState {
    counter: i32,
}

// Your custom configuration
#[derive(Clone)]
struct Config {
    threshold: i32,
}

fn main() {
    let initial_val = 10;
    let state = SystemState::default();
    let config = Config { threshold: 100 };

    // Initialize the process
    let process = PropagatingProcess::with_state(
        deep_causality_core::PropagatingEffect::pure(initial_val),
        state,
        Some(config)
    );

    let final_process = process.bind(|val, mut current_state, ctx| {
        // We have access to:
        // val: The EffectValue from the previous step
        // current_state: The accumulated state
        // ctx: The read-only configuration
        
        let v = val.into_value().expect("Value expected");
        current_state.counter += 1;
        
        // Return the new value, potentially modified state, and pass context through
        deep_causality_core::CausalEffectPropagationProcess {
            value: deep_causality_core::EffectValue::Value(v * 2),
            state: current_state,
            context: ctx,
            error: None,
            logs: Default::default(), // Logs are merged automatically
        }
    });
}
```

## 🛠️ Mission-Critical Safety: `ControlFlowBuilder`

While the monadic approach offers flexibility, some systems—like flight control or medical devices—require absolute, compile-time guarantees. For these use cases, we introduced the **ControlFlowBuilder**.

This tool allows you to define a **static execution graph** where:
*   Connections are verified at compile-time (Output Type `A` must match Input Type `A`).
*   Execution is zero-allocation (in the hot loop).
*   Logic can be restricted to stateless functions for formal verification.

```rust
use deep_causality_core::ControlFlowBuilder;

// Define a type-safe graph builder
let mut builder = ControlFlowBuilder::<MyProtocol>::default();

// Add nodes (functions)
let sensor = builder.add_node(read_sensor);
let filter = builder.add_node(filter_noise);

// Connect them (compiler checks types!)
builder.connect(sensor, filter);

// Build an optimized executable graph
let graph = builder.build();
```

## Conclusion

The Causal Monad brings the elegance of functional programming to causal inference. By treating causality as a computation chain that implicitly carries state, context, and logs, DeepCausality allows you to write code that is clean, readable, and incredibly robust.

Get Started with the Causal Monad today!

*   Explore the [DeepCausality Core documentation](https://docs.rs/deep_causality_core).
*   Check out the [examples on GitHub](https://github.com/deepcausality-rs/deep_causality).
*   Join the [community](https://deepcausality.com/community/).
*   Join the [Discord Server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).