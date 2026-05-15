---
title: Causal Monad
description: The pure/bind algebra that makes effect propagation composable, auditable, and short-circuiting on error.
section: concepts
order: 6
---

The Causal Monad is the algebra that lets [Effect Propagation Processes](/docs/concepts/effect-propagation-process/) compose without losing their properties. Each Causaloid is a step. The monad is the law for how the steps combine.

The library exposes the monad through a single type and two operations: `pure` and `bind`. The rest follows.

## The axiom, again

From the monograph chapter `causality_as_epp.tex`:

> A causal relation is a monadic dependency, in which one propagating effect is obtained from another by composition with a causal function in a monadic context of the causal process.

The equation:

```
m₂ = m₁ >>= f
```

Here `>>=` is the monad's `bind`, `m₁` and `m₂` are propagating effects, and `f` is a causal function. The Causal Monad implements that operator over the five-field `CausalEffectPropagationProcess` struct.

## The type

```rust
pub struct CausalMonad<S = (), C = ()>(PhantomData<(S, C)>);
```

A zero-sized marker. The work happens in trait impls. The two parameters fix the state type `S` and the context type `C` for a given monad instance. The default is `()` for both, which yields the stateless monad most everyday code uses.

The trait that powers it is `MonadEffect5` over the `CausalSystem<S, C>` registry; it provides the two operations below.

## `pure`

```rust
fn pure<T>(value: T)
    -> CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>
```

Lifts a plain value into the monad. The returned process has:

- `value = EffectValue::Value(value)`
- `state = S::default()`
- `context = None`
- `error = None`
- `logs = EffectLog::default()`

This is the seed for a chain. Most chains start with `CausalMonad::pure(input)` and immediately bind.

## `bind`

```rust
fn bind<T, U, F>(
    process: CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>,
    mut f: F,
) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>
where
    F: FnMut(T) -> CausalEffectPropagationProcess<U, S, C, CausalityError, EffectLog>,
    U: Default,
```

Chains the next step. `bind` does four things, in this order:

1. If `process.error.is_some()`, short-circuit. Return a fresh process with the same error and the existing logs preserved.
2. Otherwise, unwrap `process.value`. If the inner variant is `Value(t)`, call `f(t)` to produce the next process.
3. Merge the upstream logs into the next process's logs via `LogAppend::append`. The chain's audit trail grows; entries do not vanish across binds.
4. Return the merged process.

The shape is intentionally close to what you would write in Haskell. Substitute `>>=` for `bind`, and the algebra is the same. The Rust version is type-checked against the five-field record, so the state and context types stay consistent across the chain at compile time.

## A minimal example

```rust
use deep_causality_core::*;

let final_process = CausalMonad::<i32, String>::bind(
    CausalMonad::<i32, String>::pure(10),
    |value| {
        let mut next = CausalMonad::<i32, String>::pure(value + 1);
        next.logs.add_entry("step 1");
        next
    },
);

assert_eq!(unwrap_value(final_process.value), 11);
assert_eq!(final_process.logs.len(), 1);
```

This is the test case `tests/types/causal_monad/causal_monad_tests.rs::test_bind` distilled to its essentials. Two binds and you have a chain. Five binds and you have a pipeline. Five hundred and you have a system.

## The monad laws

A monad earns the name by satisfying three identities. The Causal Monad satisfies them.

**Left identity.** `pure(a) >>= f` is equal to `f(a)`. Wrapping a value and immediately binding is the same as just calling the function.

**Right identity.** `m >>= pure` is equal to `m`. Binding `pure` at the end is a no-op.

**Associativity.** `(m >>= f) >>= g` is equal to `m >>= (\x -> f(x) >>= g)`. Grouping does not change the result.

The library's test suite covers these explicitly. The point of the laws in practice: you can refactor a chain freely — pull a step out, inline a step in, regroup — and the meaning does not change.

## The stateless alias

When you do not need a custom state or context, the `PropagatingEffect<T>` alias narrows the parameter space:

```rust
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
```

Its monad is `CausalMonad<(), ()>`, and the operations are the same `pure` and `bind` with `S = C = ()`. This is what a Causaloid returning a `PropagatingEffect` is actually returning.

## Why this matters

Three concrete payoffs.

**Short-circuiting on error costs nothing.** The first failed step turns into an `error.is_some()` on the carried process, and every subsequent `bind` is a no-op that preserves the logs. You do not write `?` propagation by hand inside the chain.

**Logs accumulate without instrumentation.** `LogAppend::append` runs inside every bind. A consumer that wants to print or persist the trace gets the full ordered sequence without any side-channel.

**Refactoring stays safe.** The laws guarantee that breaking a long chain into helper functions, or composing several chains into a larger one, does not change the result. You get the kind of refactoring confidence that pure functional code usually offers.

## What it is not

The Causal Monad is not the only monad on the planet. It is one specific structure tailored to causal effect propagation, with five fixed parameter slots. It does not subsume `Result`, `Option`, or `Future`.

The Causal Monad is not implicit. There is no `do` block. Rust does not have monadic sugar; the calls to `pure` and `bind` are visible at the call site. That visibility is sometimes annoying and sometimes useful; the library cannot remove the cost of Rust's surface syntax for you.

The Causal Monad is not free of cost. `bind` performs a struct move and a log append per step. Negligible for inference chains; nontrivial if you call it in a tight inner loop a billion times. Profile if you suspect the latter.

## Where to look next

[HKT](/docs/concepts/hkt/) explains how the monad's signature stays generic across the five parameters without runtime cost. [Effect Propagation Process](/docs/concepts/effect-propagation-process/) is the type the monad operates over. [Causaloid](/docs/concepts/causaloid/) is what produces the values that flow through.
