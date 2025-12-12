# DeepCausality Core Concepts

This document explains the fundamental concepts in `deep_causality_core`, the foundational crate for causal computation in DeepCausality.

---

## Overview

`deep_causality_core` provides a **monadic framework** for building causal reasoning systems. At its heart is the idea that causality can be modeled as a functional dependency:

```
E₂ = f(E₁)
```

Where an **effect** `E₂` is produced by applying a causal function `f` to an incoming effect `E₁`. The core crate provides types and traits to:

1. **Propagate effects** through chains of causal functions
2. **Handle errors** automatically (short-circuiting)
3. **Maintain audit logs** for explainability
4. **Support interventions** for counterfactual reasoning
5. **Carry state and context** for Markovian processes

---

## The Causal Monad Pattern

A **monad** is a design pattern from functional programming that allows composable, chainable computations. In DeepCausality, the monad pattern enables:

- **Composition**: Chain multiple causal steps with `bind`
- **Error Propagation**: Errors automatically short-circuit subsequent steps
- **Logging**: Every operation appends to an audit trail
- **Type Safety**: The compiler enforces correct usage

### Core Operations

| Operation | Description |
|-----------|-------------|
| `pure(value)` | Lifts a plain value into the monadic context |
| `bind(f)` | Chains a computation, passing value/state/context to `f` |
| `intervene(new_value)` | Forces a new value mid-chain (for counterfactuals) |

---

## `CausalEffectPropagationProcess<V, S, C, E, L>`

This is the **fundamental type** in `deep_causality_core`. It's a 5-arity container that unifies:

| Field | Type Parameter | Description |
|-------|----------------|-------------|
| `value` | `V` | The primary data being transformed (wrapped in `EffectValue<V>`) |
| `state` | `S` | Mutable state that evolves through the chain (Markovian) |
| `context` | `C` | Read-only configuration or environment data |
| `error` | `E` | Error state that short-circuits further computation |
| `logs` | `L` | Append-only audit history |

### Why 5 Parameters?

In complex causal reasoning, we need more than just a value:

1. **Value**: The data flowing through the pipeline
2. **State**: Information that accumulates (e.g., counters, risk scores)
3. **Context**: Global configuration that doesn't change (e.g., thresholds)
4. **Error**: Failure information that preserves logs even when computation fails
5. **Logs**: Complete history for explainability and compliance

---

## Type Aliases

To simplify common use cases, two type aliases are provided:

### `PropagatingEffect<T>` — Stateless Effects

```rust
pub type PropagatingEffect<T> = 
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
```

Use when:
- You don't need state across steps
- You don't need external context
- You want simple, functional transformations

**Example:**
```rust
use deep_causality_core::PropagatingEffect;

let result = PropagatingEffect::pure(10.0)
    .bind(|val, _, _| PropagatingEffect::pure(val.into_value().unwrap() * 2.0))
    .bind(|val, _, _| PropagatingEffect::pure(val.into_value().unwrap() + 5.0));
// Result: 25.0
```

### `PropagatingProcess<T, S, C>` — Stateful Processes

```rust
pub type PropagatingProcess<T, S, C> = 
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

Use when:
- You need to accumulate state (Markov property)
- You need global configuration
- You're building complex, multi-stage pipelines

**Example:**
```rust
use deep_causality_core::PropagatingProcess;

#[derive(Clone, Default)]
struct RiskState { total_risk: f64 }

let process: PropagatingProcess<f64, RiskState, ()> = 
    PropagatingProcess::pure(0.5);
```

---

## `EffectValue<T>` — The Payload Enum

`EffectValue` wraps the actual data being propagated. It's an enum with variants for different scenarios:

| Variant | Description |
|---------|-------------|
| `None` | Absence of a value (like `Option::None`) |
| `Value(T)` | A concrete value of type `T` |
| `ContextualLink(id1, id2)` | Reference to data in a Context |
| `RelayTo(index, effect)` | Dispatch command for adaptive routing |
| `Map(HashMap)` | Collection of named sub-effects |

### Common Pattern

```rust
// Extracting the inner value
let effect = PropagatingEffect::pure(42);
let inner: Option<i32> = effect.value.into_value();
```

---

## `EffectLog` — Audit Trail

Every `CausalEffectPropagationProcess` carries an `EffectLog` that records:

- Timestamped entries for each operation
- Intervention markers (when `intervene()` is called)
- Error messages when failures occur

### Purpose

1. **Explainability**: Trace back *why* a result was reached
2. **Auditability**: Compliance-ready record of decisions
3. **Debugging**: Understand flow through complex graphs

### API

```rust
use deep_causality_haft::LogAddEntry;

let mut log = EffectLog::new();
log.add_entry("Step 1: Validated input");
log.add_entry("Step 2: Applied transformation");
```

---

## `Intervenable` Trait — Counterfactual Reasoning

The `Intervenable` trait enables **intervention**, a key operation for counterfactual analysis.

### What is Intervention?

In causal inference, an **intervention** forces a variable to a specific value, breaking its natural dependencies. This is Pearl's `do()` operator.

### The `intervene()` Method

```rust
pub trait Intervenable<T> {
    fn intervene(self, new_value: T) -> Self;
}
```

**Behavior:**
1. **Replaces** the current value with `new_value`
2. **Preserves** the log history (adds intervention marker)
3. **Propagates** any existing error (intervention can't fix errors)

### Example: Counterfactual Analysis

```rust
use deep_causality_core::{Intervenable, PropagatingEffect};

// Factual: Natural causal chain
let factual = PropagatingEffect::pure(0.8)
    .bind(|x, _, _| PropagatingEffect::pure(x.into_value().unwrap() * 2.0));
// Result: 1.6

// Counterfactual: What if the input had been 0.2?
let counterfactual = PropagatingEffect::pure(0.8)
    .intervene(0.2)  // Force value to 0.2
    .bind(|x, _, _| PropagatingEffect::pure(x.into_value().unwrap() * 2.0));
// Result: 0.4

// Causal effect = Factual - Counterfactual = 1.2
```

---

## `CausalMonad` — The Monad Implementation

`CausalMonad` implements the `MonadEffect5` trait from `deep_causality_haft`, providing:

| Method | Description |
|--------|-------------|
| `pure(value)` | Lifts a value into `CausalEffectPropagationProcess` |
| `bind(process, f)` | Chains a function, handling errors and logs |

### Higher-Kinded Types (HKT)

DeepCausality uses HKT patterns via the `deep_causality_haft` crate. The `CausalMonad` works with:

- `Effect5` trait: Declares the 5-arity effect structure
- `MonadEffect5` trait: Provides `pure` and `bind`
- Witness types: `PropagatingEffectWitness`, `PropagatingProcessWitness`

This enables generic programming over different effect types while maintaining type safety.

---

## Error Handling

Errors in `deep_causality_core` use `CausalityError`:

```rust
pub struct CausalityError(pub CausalityErrorEnum);

pub enum CausalityErrorEnum {
    InternalLogicError,
    Custom(String),
    // ... other variants
}
```

### Error Propagation

When an error occurs:
1. The error is stored in the `error` field
2. Subsequent `bind()` calls **skip** execution
3. Logs are **preserved** up to the error point
4. The final result contains the error and complete log

```rust
let result = PropagatingEffect::pure(10)
    .bind(|_, _, _| PropagatingEffect::from_error(
        CausalityError::new(CausalityErrorEnum::Custom("Failed".into()))
    ))
    .bind(|x, _, _| {
        // This never executes!
        PropagatingEffect::pure(x.into_value().unwrap() * 2)
    });

// result.error is Some(...)
// result.logs contains entries up to failure
```

---

## `bind_or_error` — Convenience Method

A common pattern is to unwrap `EffectValue` and error if it's `None`:

```rust
let result = PropagatingEffect::pure(Some(42))
    .bind_or_error(|val, _, _| {
        // val is already unwrapped from EffectValue!
        PropagatingEffect::pure(val * 2)
    }, "Expected a value, got None");
```

This avoids manual pattern matching on every step.

---

## Control Flow Builder

For complex graphs, `deep_causality_core` provides a builder pattern:

```rust
use deep_causality_core::{ControlFlowBuilder, ExecutableNode, NodeType};

let graph = ControlFlowBuilder::new()
    .add_node(ExecutableNode::new(0, NodeType::Start, my_start_fn))
    .add_node(ExecutableNode::new(1, NodeType::Process, my_process_fn))
    .add_edge(0, 1)
    .build();
```

This is useful for constructing explicit causal graphs at runtime.

---

## Summary

| Concept | Type/Trait | Purpose |
|---------|------------|---------|
| Core Container | `CausalEffectPropagationProcess` | 5-arity monad (V, S, C, E, L) |
| Stateless Alias | `PropagatingEffect<T>` | Simple functional chains |
| Stateful Alias | `PropagatingProcess<T, S, C>` | Markovian processes |
| Payload | `EffectValue<T>` | Wrapped value with dispatch variants |
| Audit Log | `EffectLog` | Timestamped operation history |
| Monad Impl | `CausalMonad` | `pure` and `bind` operations |
| Intervention | `Intervenable` | Counterfactual `do()` operator |
| Errors | `CausalityError` | Short-circuiting error propagation |

---

## Next Steps

- See [examples/starter_example](../examples/starter_example) for Pearl's Ladder demonstration
- See [examples/core_examples](../examples/core_examples) for more `PropagatingEffect` patterns
- See [deep_causality_haft](../deep_causality_haft) for the HKT foundation
