---
title: Hello, Causal Monad
description: The smallest program that exercises pure and bind. Walk a value through a three-step chain, then look at what flowed.
section: getting-started
order: 2
---

The Causaloid is the unit. The Context is the environment. The thing that flows between Causaloids, threading through them with value, state, context, errors, and an audit log, is the Causal Monad.

This page introduces the monad on its own, before any Causaloid appears, because everything else in the library composes on top of `pure` and `bind`. Understanding those two operations once is enough to read every other example.

## What a monad is, without the academic preamble

A monad is two operations:

- **`pure(v)`**: wrap a plain value in the monad. Trivial.
- **`bind(m, f)`**: given a wrapped value and a function that *also* produces a wrapped value, chain them. The function gets to see the unwrapped value, and its result becomes the next link in the chain.

That's it. Everything else follows from those two operations satisfying three identities (the *monad laws*). The library encodes both operations on the `CausalEffectPropagationProcess` struct, which is what the Causal Monad threads through.

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

`PropagatingEffect<T>` is the everyday alias for the Causal Monad. The full name is `CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>`: five type parameters, three of them pinned to defaults. The defaults are sane for almost every starting program, which is why the alias is what most code reaches for.

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

## The static form: `CausalMonad<S, C>`

When the chain needs a real state or a real context type, the static form of the monad is the cleaner path:

```rust
use deep_causality_core::{CausalMonad, EffectValue};

let initial = CausalMonad::<i32, String>::pure(10);

let next = CausalMonad::<i32, String>::bind(initial, |val| {
    let mut p = CausalMonad::<i32, String>::pure(val + 1);
    p.logs.add_entry("step1");
    p
});

if let EffectValue::Value(v) = next.value {
    println!("v = {}", v); // 11
}
println!("log entries: {}", next.logs.len()); // 1
```

The two type parameters are the state type (`i32` here) and the context type (`String`). The closure takes one argument, the unwrapped value, because state and context are threaded automatically by the monad's internals. The fluent `.bind(|v, s, c| ...)` form on `PropagatingEffect` exists because the stateless case is common enough to deserve a shortcut; the static form is more explicit when state or context carry real information.

Both forms are the same monad. They differ only in how they spell the call site.

## The three laws: a quick check

A monad earns its name by satisfying three identities. The Causal Monad satisfies them.

- **Left identity.** `pure(a).bind(f)` is the same as `f(a)`.
- **Right identity.** `m.bind(pure)` is the same as `m`.
- **Associativity.** `(m.bind(f)).bind(g)` is the same as `m.bind(|x| f(x).bind(g))`.

In practice this means you can freely refactor a chain. Pull a step out into a helper. Inline a step back in. Regroup three steps into two-then-one or one-then-two. The chain still computes the same answer. The library's test suite covers all three laws explicitly; the laws are not a marketing claim.

## Where this goes next

The next page wraps a function in a [Causaloid](/docs/concepts/causaloid/) and evaluates it. A Causaloid is essentially a named, identified, composable causal function whose evaluation returns a `PropagatingEffect`. Once you can write a `PropagatingEffect`-producing closure, and you can now, wrapping it as a Causaloid is one constructor call.

The page after that adds a [Context](/docs/concepts/context/). The Context is the third argument the bind closure has been ignoring on this page. When the rule needs to read from the world, the Context is where the world lives.

## What to keep in your head

Two operations. `pure` lifts a value. `bind` threads it through a function. The chain carries value, state, context, error, and log together as one structured record. Everything else in the library composes on top.
