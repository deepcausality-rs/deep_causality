---
title: Causal Flow
description: The fluent high-level DSL over the causal monad. It reads a causal pipeline as a sequence of verbs and adds sugar, not new semantics.
sidebar:
  order: 7
---

`CausalFlow` is the fluent API over the [Causal Monad](/concepts/causal-monad/). The monad is the algebra: `pure` and `bind` over the [carrier effect](/concepts/effect-propagation-process/). Written out by hand, a monadic pipeline exhibits real complexity. You wrap values in `EffectValue`, call `pure` and `with_state`, unwrap with `into_value().unwrap_or_default()`, and check the error channel between steps. `CausalFlow` hides all of that behind a much simplified fluent API.

```rust
use deep_causality::CausalFlow;

let outcome = CausalFlow::value(2_i64)
    .try_step(|x| Ok(x + 3))
    .map(|x| x * 10)
    .finish();

assert_eq!(outcome, Ok(50));
```

That is the same chain you would write with `PropagatingEffect::pure(2).bind(...).bind(...)`, with all the wrapping removed.

## Causal Monad, Simplified

Every `CausalFlow` verb lowers to an existing monad operation. `value` lowers to `pure`. `map` lowers to `fmap`. `try_step`, `branch`, `iterate_n`, and the rest lower to `bind`. Under the hood, everything is still the causal monad with all its expressiveness. It is just the interface that has been simplified.

Two consequences follow. First, the [monad laws](/concepts/causal-monad/#the-monad-laws) still hold, so a flow refactors as freely as the chain underneath it. Second, you can drop in and out of the DSL at any point: `from(process)` lifts an existing process into a flow, and `into_process()` / `into_effect()` drop back to the concrete carrier for code that expects it.

## The Fluent API

The surface groups into six families.

**Seed.** `value(v)` starts a stateless flow carrying a value. `process(s)` starts a stateful flow seeding the state channel. `effect()` seeds the unit value. `from(process)` lifts an existing carrier, and `.context(c)` attaches a read-only context.

**Step.** `map(|v| u)` transforms the value. `try_step(|v| Result)` runs a fallible step. `and_then` / `next` compose a whole sub-pipeline (`Value -> CausalFlow<U>`). `guard(|&v| Result)` validates without changing the value. `update_state` / `update_context` evolve a single channel. `bind` is the raw monad passthrough for an existing stage signature.

**Branch.** `branch(cond, on_true, on_false)` routes by a test on the value. `branch_with` tests the value, state, and context together. `either` routes a flow whose value is `Either<L, R>`. Each arm is itself a flow, so a branch arm can be a full sub-pipeline.

**Loop.** `iterate_n(n, step)` runs a step a fixed number of times. `iterate_until(pred, max, step)` runs until a predicate holds. `iterate_to_fixpoint(max, step)` runs until the value stops changing. The two open-ended forms take a step bound and fail with a `MaxStepsExceeded` error rather than spinning forever.

**Intervene.** `intervene(v)` force-substitutes the value, Pearl's `do(v)`, and records the override in the audit log. `intervene_if(cond, f)` does it only when a test holds.

**Finish.** `finish()` returns `Result<Value, CausalityError>`. `run(on_ok, on_err)` dispatches to handlers. `is_err()` peeks at the error channel. `into_process()` / `into_effect()` return the concrete carrier.

## A control loop

`iterate_n` and `branch` together express a bounded loop whose body decides what to do each tick. The arms are flows, so the branch reads as two small pipelines.

```rust
use deep_causality::CausalFlow;

let total = CausalFlow::value(0_i64)
    .iterate_n(5, |tick| {
        tick.branch(
            |n| n % 2 == 0,              // is the value even?
            |even| even.map(|n| n + 10), // yes: add 10
            |odd| odd.map(|n| n + 1),    // no: add 1
        )
    })
    .finish();

assert_eq!(total, Ok(50));
```

This is the shape the avionics and corrective-control examples take: one `iterate_n` for the loop, `next` to wire per-tick stages, and a `branch` for the conditional intervention.

## Factual and counterfactual


```rust
use deep_causality::CausalFlow;

// factual: the reading is 8, the pipeline scales it
let factual = CausalFlow::value(8_i64).map(|x| x * 2).finish();

// counterfactual: what if the reading had been clamped to 0?
let counterfactual = CausalFlow::value(8_i64)
    .intervene(0) // do(reading = 0)
    .map(|x| x * 2)
    .finish();

assert_eq!(factual, Ok(16));
assert_eq!(counterfactual, Ok(0));
```

## Named stages compose

`next` composes a sub-pipeline, a function `Value -> CausalFlow<U>`. Pulling each step into a named function keeps a long pipeline readable, and the laws guarantee the meaning does not change.

```rust
use deep_causality::CausalFlow;

fn scale(x: i64) -> CausalFlow<i64> {
    CausalFlow::value(x * 10)
}

let out = CausalFlow::value(5_i64).next(scale).map(|x| x + 1).finish();
assert_eq!(out, Ok(51));
```

## The error channel is automatic

A failing step moves the flow into its error channel, and every later verb becomes a no-op that carries the error and the accumulated log forward. You do not write `?` between steps.

```rust
use deep_causality::{CausalFlow, CausalityError, CausalityErrorEnum};

let outcome = CausalFlow::value(-1_i64)
    .try_step(|n| {
        if n >= 0 {
            Ok(n)
        } else {
            Err(CausalityError::new(CausalityErrorEnum::Custom("negative input".into())))
        }
    })
    .map(|n| n * 2) // skipped
    .finish();

assert!(outcome.is_err());
```

## Stateless and stateful

`CausalFlow<Value>` defaults its state and context to `()`, the stateless form that lowers to [`PropagatingEffect`](/concepts/effect-propagation-process/). Seed it with `value(v)`. When a pipeline needs memory, seed it with `process(s)` and evolve the state; the flow now lowers to `PropagatingProcess` and threads state Markovian-style.

```rust
use deep_causality::CausalFlow;

let final_process = CausalFlow::process(0_i64) // state = 0
    .update_state(|state, _value| state + 1)
    .into_process();

assert_eq!(final_process.state, 1);
```

The verbs are identical across both forms. The type parameters decide whether the chain carries memory, exactly as they do for the monad underneath.

## Where to look next

[Causal Monad](/concepts/causal-monad/) is the `pure`/`bind` algebra this DSL is sugar over. [Effect Propagation Process](/concepts/effect-propagation-process/) is the carrier both operate on. [Counterfactuals](/concepts/counterfactuals/) covers `intervene` and Pearl's Ladder in full. For the hands-on introduction, start with [Hello, Causal Flow](/getting-started/hello-causal-flow/).
