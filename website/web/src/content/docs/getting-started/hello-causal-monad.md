---
title: Hello, Causal Monad
description: The smallest program that exercises pure and bind. Walk a value through a three-step chain, then look at what flowed.
section: getting-started
order: 2
---

This page introduces the monad on its own, before any Causaloid appears, because everything else in the library composes on top of `pure` and `bind`. Understanding those two operations is important to read every other example.

## What a monad is

A monad is two operations:

- **`pure(v)`**: wrap a plain value in the monad.
- **`bind(m, f)`**: given a wrapped value and a function that *also* produces a wrapped value, chain them. The function gets to see the unwrapped value, and its result becomes the next link in the chain.

Everything else follows from those two operations satisfying three identities (the *monad laws*). These operations are not a separate type; they are a trait, `CausalMonad`, implemented by the carrier effect [`CausalEffectPropagationProcess`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core). The carrier *is* the monad.

## The smallest possible program

```rust
use deep_causality::PropagatingEffect;

fn main() {
    let result = PropagatingEffect::pure(10_i32)
        .bind(|v, _state, _ctx| {
            let n = v.into_value().unwrap_or_default();
            PropagatingEffect::pure(n + 1)
        })
        .bind(|v, _state, _ctx| {
            let n = v.into_value().unwrap_or_default();
            PropagatingEffect::pure(n * 2)
        });

    let final_value = result.value.into_value().unwrap_or_default();
    println!("result = {}", final_value); // prints: result = 22
}
```

Three lines of substance. `pure(10)` lifts the integer into a `PropagatingEffect<i32>`. The first `bind` unwraps the value, increments it, and re-wraps. The second `bind` unwraps again, doubles, re-wraps. The final value is read off `result.value`.

`PropagatingEffect<T>` is the everyday alias for the carrier effect, exported from both [`deep_causality`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) and [`deep_causality_core`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core). The full name is `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`: five type parameters, three of them pinned to defaults. The defaults are sane for almost every starting program, which is why the alias is what most code reaches for.

Run this:

```bash
cargo new hello_monad
cd hello_monad
cargo add deep_causality
# paste the code above into src/main.rs
cargo run --release
```

You should see `result = 22`.

## What just happened

Each `bind` closure takes three arguments. The first is the wrapped `EffectValue` from the upstream link. The second is the threaded `state`. The third is the threaded `context`. The example ignores state and context because both are `()` at this scale. They become useful when the chain has a reason to carry them, and the type system makes that promotion explicit.

The closures return *new* `PropagatingEffect`s rather than bare values. That is the whole point. Returning a `PropagatingEffect` means a step can also signal:

- *I produced no value.* Return `PropagatingEffect` carrying `EffectValue::None`.
- *I failed.* Return `PropagatingEffect::from_error(err)`. The chain short-circuits; downstream binds are no-ops.
- *I want to relay to a different rule.* Return `EffectValue::RelayTo(idx, sub)`. The chain reroutes.

The bare `Result` your everyday Rust code uses gives you the failure branch alone. The Causal Monad gives you all three branches plus a structured log that survives across every link.

## Look at the log

Add one line to the first closure:

```rust
.bind(|v, _state, _ctx| {
    let n = v.into_value().unwrap_or_default();
    let mut next = PropagatingEffect::pure(n + 1);
    next.logs.add_entry("incremented");
    next
})
```

Now read the log out at the end:

```rust
for entry in &result.logs {
    println!("log: {:?}", entry);
}
```

Every step's log entries accumulate in `result.logs`. The chain has an audit trail by construction; you did not write a logger. The trail survives errors. If a later bind fails, the log of every step that ran before the failure is still there.

## Carrying state and context

The example above ignored the second and third arguments because a `PropagatingEffect` pins both to `()`. When the chain needs real state or a real context, reach for `PropagatingProcess<T, S, C>`. It is the same carrier and the same `bind`; the only change is that `State` and `Context` now carry information, so the closure can read the running state and return an evolved one.

```rust
use deep_causality::{EffectLog, EffectValue, PropagatingProcess};

// value: i32, Markovian state: i32, context: String
let process = PropagatingProcess::<i32, i32, String>::pure(10)
    .bind(|value, state, context| {
        let n = value.into_value().unwrap_or_default();
        PropagatingProcess {
            value: EffectValue::Value(n + 1),
            state: state + 1, // the state evolves and carries forward
            context,          // the context threads through
            error: None,
            logs: EffectLog::new(),
        }
    });
```

`bind` keeps the state and context of the process the closure returns, so the chain is Markovian: each step sees the state the previous step left. The stateless `PropagatingEffect<T>` is exactly this with `State = Context = ()`. Same trait, same `bind`; the type parameters decide whether the chain carries memory.

## The three laws: a quick check

A monad earns its name by satisfying three identities. The Causal Monad satisfies them.

- **Left identity.** `pure(a).bind(f)` is the same as `f(a)`.
- **Right identity.** `m.bind(pure)` is the same as `m`.
- **Associativity.** `(m.bind(f)).bind(g)` is the same as `m.bind(|v, s, c| f(v, s, c).bind(g))`.

In practice this means you can freely refactor a chain. Pull a step out into a helper. Inline a step back in. Regroup three steps into two-then-one or one-then-two. The chain still computes the same answer. The library's test suite covers all three laws explicitly.

## Where this goes next

The [next page](/docs/getting-started/hello-causaloid/) wraps a function in a [Causaloid](/docs/concepts/causaloid/) and evaluates it. A Causaloid is a named, identified, composable causal function whose evaluation returns a `PropagatingEffect`. Wrapping the kind of closure you wrote above as a Causaloid is one constructor call. For a complete, runnable end-to-end version that walks Pearl's Ladder of Causation through `pure`, `bind`, and `intervene`, see [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example).

The page after that adds a [Context](/docs/concepts/context/). The Context is the third argument the bind closure has been ignoring on this page. When the rule needs to read from the world, the Context is where the world lives.
