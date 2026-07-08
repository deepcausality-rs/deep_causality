---
title: Hello, Causal Flow
description: The high-level DSL that reads causal reasoning as a pipeline. Compose steps, loops, branches, and interventions over the causal monad without the wrapping ceremony.
sidebar:
  order: 2
---

This is the friendly front door. `CausalFlow` is a fluent facade over the [Causal Monad](/concepts/causal-monad/), and it lets you write a causal pipeline the way you read it: top to bottom, one verb per line. The monad underneath is doing the work, but you do not see its wrapping ceremony. Start here, then the [next page](/getting-started/hello-causal-monad/) opens the engine.

## The smallest possible program

```rust
use deep_causality::CausalFlow;

fn main() {
    let outcome = CausalFlow::value(2_i64)
        .try_step(|x| Ok(x + 3)) // 2 + 3 = 5
        .map(|x| x * 10)         //     5 * 10 = 50
        .finish();

    println!("outcome = {:?}", outcome); // prints: outcome = Ok(50)
}
```

Four lines tell the whole story. `value(2)` seeds the flow with a starting value. `try_step` runs a fallible step. `map` transforms the value. `finish` ends the flow and hands back a `Result`. No `CausalEffect` to unwrap, no `pure`/`with_state` constructors, no manual error checks between steps.

Run it:

```bash
cargo new hello_flow
cd hello_flow
cargo add deep_causality
# paste the code above into src/main.rs
cargo run --release
```

You should see `outcome = Ok(50)`.

## Reading the pipeline

Each verb is a step the value flows through. The common ones:

- **`value(v)`** seeds a flow with a starting value.
- **`map(|v| ...)`** transforms the value and passes it on. Use it when a step only changes the value.
- **`try_step(|v| Ok(u))`** runs a fallible step. An `Ok` becomes the next value; an `Err` moves the flow into its error channel.
- **`finish()`** ends the flow and returns `Result<Value, CausalityError>`. Its sibling `run(on_ok, on_err)` dispatches to a handler instead of returning.

Every verb lowers to a single Causal Monad operation, so the flow has the exact semantics of the monad. The DSL adds reading clarity, not new behavior.

## The error channel is automatic

A step that fails short-circuits the rest of the flow. You do not write `?` between steps, and you do not check for failure after each line.

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
    .map(|n| n * 2) // skipped: the flow is already in its error channel
    .finish();

assert!(outcome.is_err());
```

The `map` after the failing `try_step` does not run. The first failure carries straight through to `finish`, with the audit log of every step that did run still intact.

## Loops and branches

Two more verbs give a flow real control flow. `iterate_n(n, step)` runs a step `n` times. `branch(cond, on_true, on_false)` routes the flow by a test on the current value. Both arms of a branch are themselves flows, so a branch arm can be a whole sub-pipeline.

```rust
use deep_causality::CausalFlow;

let total = CausalFlow::value(0_i64)
    .iterate_n(5, |tick| {
        tick.branch(
            |n| n % 2 == 0,            // is the value even?
            |even| even.map(|n| n + 10), // yes: add 10
            |odd| odd.map(|n| n + 1),    // no: add 1
        )
    })
    .finish();

assert_eq!(total, Ok(50)); // the value stays even, so +10 runs five times
```

Two more loop verbs handle open-ended iteration: `iterate_until(pred, max, step)` runs until a predicate holds, and `iterate_to_fixpoint(max, step)` runs until the value stops changing. Both take a step bound and fail with a `MaxStepsExceeded` error rather than looping forever.

## Value substitution: `alternate_value`

`alternate_value(v)` force-substitutes the carried value and records the override in the audit log. It is the value-level substitution counterfactual reasoning is built from: factual and counterfactual runs share the same flow and the same engine, differing only in a substituted value. Pearl's full `do()` operator — graph surgery over the causal hypergraph — lives at the `deep_causality` graph layer; `alternate_value` is the value-level substitution it builds on.

```rust
use deep_causality::CausalFlow;

let counterfactual = CausalFlow::value(8_i64)
    .alternate_value(0) // substitute value = 0
    .map(|n| n + 1)
    .finish();

assert_eq!(counterfactual, Ok(1)); // the 8 is gone; the flow continues from 0
```

`alternate_value_if(cond, f)` is the conditional form: it overrides the value only when a test holds, which is how a closed-loop controller fires a correction only when a monitor trips.

## State when you need it

The flows above carry a value and nothing else. When a pipeline needs memory, seed it with `process(state)` instead of `value(v)` and evolve the state as it runs.

```rust
use deep_causality::CausalFlow;

let final_process = CausalFlow::process(0_i64) // state = 0
    .update_state(|state, _value| state + 1)   // the state evolves and carries forward
    .into_process();

assert_eq!(final_process.state, 1);
```

This is the same flow, now Markovian: each step sees the state the previous step left. `value(v)` is the stateless form; `process(s)` is the stateful one. The verbs are identical.

## What it lowers to

Nothing here is new machinery. `CausalFlow` is sugar over the [Causal Monad](/concepts/causal-monad/): `value` lowers to `pure`, `map` to `fmap`, `try_step` and `branch` and `iterate_n` to `bind`. The [Causal Flow](/concepts/causal-flow/) concept page lays out the full verb set and what each one lowers to.

## Where this goes next

The [next page](/getting-started/hello-causal-monad/) drops one level down to `pure` and `bind`, the two operations the whole flow is built on. After that, [Hello, Causaloid](/getting-started/hello-causaloid/) wraps a step as a named, composable causal unit, and [Hello, Context](/getting-started/hello-context/) gives a step a world to read from. For a complete runnable program that walks Pearl's three rungs through a flow, see [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example).
