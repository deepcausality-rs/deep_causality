# Specification: The `PropagatingProcess` Monad

## 1. Overview

This document specifies the `PropagatingProcess`, a monadic structure designed for stateful, context-aware, and sequential causal process modeling.

### 1.1. Motivation

The existing `PropagatingEffect` serves as a powerful tool for non-markovian causal chains, where the outcome of a step depends solely on the value of the preceding one. It follows the pattern of a `Writer` monad (for logging) combined with an `Either` monad (for error handling).

However, many real-world causal systems are **Markovian**: the next state of the system depends on its current state, not just the immediate value. `PropagatingProcess` is designed to fill this gap. It extends the concept of `PropagatingEffect` by explicitly including fields for `State` and `Context`, enabling the modeling of such state-dependent processes.

### 1.2. Core Concept

`PropagatingProcess` is a monad that encapsulates a 5-tuple: `(Value, State, Context, Error, Log)`. It allows chaining operations where each subsequent step receives the `Value`, `State`, and `Context` from the previous one, and produces a new tuple to be passed forward.

## 2. Data Structure Definition

The core data structure is `CausalProcessEffect`, with `PropagatingProcess` as its type alias.

```rust
// In deep_causality_core::types::propagating_process::mod.rs

use crate::EffectValue;

#[derive(Debug, PartialEq, Clone)]
pub struct CausalProcessEffect<Value, State, Context, Error, Log> {
    pub value: EffectValue<Value>,
    pub state: State,
    pub context: Option<Context>,
    pub error: Option<Error>,
    pub logs: Log,
}

pub type PropagatingProcess<Value, State, Context, Error, Log> =
    CausalProcessEffect<Value, State, Context, Error, Log>;
```

### 2.1. Fields

- **`value: EffectValue<Value>`**: The primary outcome of the current process step.
- **`state: State`**: The internal state carried *out* of the current step, to be passed to the next.
- **`context: Option<Context>`**: The external environment for the computation, which can be read and modified.
- **`error: Option<Error>`**: The error propagation channel for short-circuiting.
- **`logs: Log`**: An append-only log for full traceability.

## 3. Monadic Framework & Implementation Strategy

The implementation must integrate with the existing HKT framework in `deep_causality_haft`, which presents a significant design constraint.

### 3.1. The `MonadEffect5` Constraint

The `deep_causality_haft` crate provides an arity-5 monadic trait, `MonadEffect5`. Its `bind` signature is:
```rust
fn bind<T, U, Func>(
    effect: HKT5::Type<T>,
    f: Func,
) -> HKT5::Type<U>
where
    Func: FnMut(T) -> HKT5::Type<U>
```
The function `f` passed to `bind` only receives the `value` (`T`) of the previous step. It has no direct access to the `state` or `context` from the incoming `effect`. This makes the trait unsuitable for implementing a true state-passing (Markovian) monad, as `f` cannot make decisions based on the previous state.

To address this, we propose a dual implementation strategy: a powerful, ergonomic method for developers, and a trait implementation for framework compatibility.

### 3.2. Strategy 1: Fluent `bind` Method on the Struct

The primary API for developers will be a `bind` method implemented directly on the `CausalProcessEffect` struct. This method's signature **will** allow for state and context passing, enabling true stateful computation.

```rust
// Proposed for CausalProcessEffect
impl<V, S, C, E, L> CausalProcessEffect<V, S, C, E, L>
where
    // appropriate trait bounds...
    L: LogAppend,
{
    pub fn bind<F, VNew>(self, f: F) -> CausalProcessEffect<VNew, S, C, E, L>
    where
        F: FnOnce(EffectValue<V>, S, Option<C>) -> CausalProcessEffect<VNew, S, C, E, L>,
    {
        // 1. Error short-circuiting
        if let Some(error) = self.error {
            return CausalProcessEffect {
                value: EffectValue::default(), // VNew must implement Default
                state: self.state,
                context: self.context,
                error: Some(error),
                logs: self.logs,
            };
        }

        // 2. Unpack and apply function with state and context
        let mut next_process = f(self.value, self.state, self.context);

        // 3. Aggregate logs
        let mut combined_logs = self.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}
```
This method provides the expressive power needed for Markovian processes and will be the recommended way to chain `PropagatingProcess` operations.

### 3.3. Strategy 2: Framework Compatibility via `CausalProcessMonad`

To integrate with the rest of the `deep_causality` ecosystem, a `CausalProcessMonad` will be created to implement the `MonadEffect5` trait. This implementation will be "shallow" regarding state.

```rust
// Proposed CausalProcessMonad
pub struct CausalProcessMonad;

impl MonadEffect5<CausalProcessSystem> for CausalProcessMonad {
    fn pure<V>(value: V) -> PropagatingProcess<V, ...>
    where
        // State must be defaultable
        MyState: Default,
    {
        PropagatingProcess {
            value: EffectValue::Value(value),
            state: MyState::default(), // Initialize with default state
            context: None,
            error: None,
            logs: EffectLog::new(),
        }
    }

    fn bind<V, VNew, F>(
        process: PropagatingProcess<V, ...>,
        mut f: F,
    ) -> PropagatingProcess<VNew, ...>
    where
        F: FnMut(V) -> PropagatingProcess<VNew, ...>,
        VNew: Default,
    {
        if let Some(error) = process.error {
            // Error propagation
        }

        // f only receives the value
        let mut next_process = f(process.value.into_value().unwrap());
        
        // State from the new process is ignored; the old state is preserved.
        next_process.state = process.state;
        next_process.context = process.context;

        // Logs are aggregated as usual
        let mut combined_logs = process.logs;
        combined_logs.append(&mut next_process.logs);
        next_process.logs = combined_logs;

        next_process
    }
}
```
This trait implementation satisfies the framework's requirements but does not perform state-passing. It ensures that `PropagatingProcess` can be used in generic contexts that expect a `MonadEffect5`, while the developer-facing `bind` method on the struct provides the true stateful functionality.