---
title: Causal Monad
description: The pure/bind algebra the carrier effect implements, making effect propagation composable, auditable, and short-circuiting on error.
section: concepts
order: 6
---

The Causal Monad is not a separate type you reach for. It is the algebra the [carrier effect](/docs/concepts/effect-propagation-process/) already carries. `PropagatingEffect<T>` and `PropagatingProcess<T, S, C>` implement the `CausalMonad` trait, and that trait is what lets a chain of [Causaloids](/docs/concepts/causaloid/) compose without losing their properties. Each Causaloid is a step. The trait is the law for how the steps combine.

The whole algebra is two operations: `pure` and `bind`. The rest follows.

## The axiom

From the [EPP preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf):

> A causal relation is a monadic dependency, in which one propagating effect is obtained from another by composition with a causal function in a monadic context of the causal process.

The equation:

```
m₂ = m₁ >>= f
```

Here `>>=` is `bind`, `m₁` and `m₂` are propagating effects, and `f` is a causal function. The carrier effect implements that operator over the five-field `CausalEffectPropagationProcess` struct.

## A trait, not a primitive

`CausalMonad` is a trait. There is no `CausalMonad` value, no struct to instantiate, no third primitive sitting beside the Causaloid and the Context. The carrier effect *is* the monad:

```rust
pub trait CausalMonad: Sized {
    type Value;
    type State;
    type Context;

    fn pure(value: Self::Value) -> Self;

    fn bind<NewValue, F>(self, f: F)
        -> CausalEffectPropagationProcess<NewValue, Self::State, Self::Context, CausalityError, EffectLog>
    where
        F: FnOnce(
            EffectValue<Self::Value>,
            Self::State,
            Option<Self::Context>,
        ) -> CausalEffectPropagationProcess<NewValue, Self::State, Self::Context, CausalityError, EffectLog>;
}
```

It is implemented once, for `CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>`. That single impl covers both aliases: the stateless `PropagatingEffect<T>` (where `State = Context = ()`) and the stateful `PropagatingProcess<T, S, C>`. There is exactly one `bind`, and it threads state.

The same two operations are also exposed as inherent methods on the carrier, so everyday code writes `PropagatingEffect::pure(x)` and `effect.bind(...)` without importing the trait. The trait exists so generic code can bind against the contract, and so the type signature states the intent: this is a state-threading monad.

## `pure`

`pure` lifts a plain value into the carrier. The returned process has:

- `value = EffectValue::Value(value)`
- `state = State::default()`
- `context = None`
- `error = None`
- `logs = EffectLog::default()`

This is the seed for a chain. Most chains start with `PropagatingEffect::pure(input)` and immediately bind.

## `bind`

`bind` chains the next step. Its continuation receives three things: the upstream value wrapped in `EffectValue`, the threaded state, and the optional context. It returns the next process, and that process's state and context carry forward.

```rust
let next = effect.bind(|value, state, context| {
    // inspect value, read context, evolve state, return the next process
    ...
});
```

`bind` does these things, in order:

1. If the upstream `error` is `Some`, short-circuit. Return a process with the same error, the carried state and context, and the existing logs; the value becomes `EffectValue::None`. No fabricated default value is invented.
2. Otherwise call the continuation with the value, state, and context.
3. Merge the upstream logs into the next process's logs via `LogAppend::append`. The audit trail grows; entries do not vanish across binds.
4. Keep the state and context of the process the continuation returned. This is what makes the chain Markovian when it needs to be: a step can read the running state, update it, and the update survives into the next step.

The earlier design split this into two binds: a value-only effect-system bind that could not thread state, and a separate state-threading one. The value-only form froze the Markovian state, so it was removed. The trait now is the contract, and there is one `bind` that threads state correctly for both the stateless and the stateful carrier.

## `fmap`

When a step only transforms the value and has no reason to touch state, context, or error, `fmap` is the lighter operation:

```rust
let doubled = effect.fmap(|x| x * 2);
```

`fmap` maps the value and passes state, context, and logs through unchanged. It short-circuits on error like `bind`, so it never panics on an errored or empty carrier. Reach for `bind` when a step needs the state or context; reach for `fmap` when it does not.

## A minimal example

```rust
use deep_causality::PropagatingEffect;

let final_process = PropagatingEffect::pure(10)
    .bind(|value, _state, _context| {
        let n = value.into_value().unwrap_or_default();
        let mut next = PropagatingEffect::pure(n + 1);
        next.logs.add_entry("step 1");
        next
    });

assert_eq!(final_process.value.into_value(), Some(11));
assert_eq!(final_process.logs.len(), 1);
```

Two binds and you have a chain. Five binds and you have a pipeline. Five hundred and you have a system.

## The monad laws

A monad earns the name by satisfying three identities. The carrier satisfies them.

**Left identity.** `pure(a).bind(f)` is equal to `f(a)`. Wrapping a value and immediately binding is the same as just calling the function.

**Right identity.** `m.bind(pure)` is equal to `m`. Binding `pure` at the end is a no-op.

**Associativity.** `m.bind(f).bind(g)` is equal to `m.bind(|x| f(x).bind(g))`. Grouping does not change the result.

The library's test suite covers these explicitly. The point of the laws in practice: you can refactor a chain freely, pull a step out, inline a step in, regroup, and the meaning does not change.

## Stateless and stateful, one algebra

`PropagatingEffect<T>` pins `State` and `Context` to `()`, so its bind threads the unit state trivially; the chain stays Markov-free. `PropagatingProcess<T, S, C>` keeps both generic, so its bind threads real state and context. They are the same algebra over the same struct. Lifting from the stateless to the stateful form is a single constructor call, [`with_state`](/docs/concepts/effect-propagation-process/).

```rust
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
```

A Causaloid returning a `PropagatingEffect` is returning a value that already implements this trait.

## Why this matters

Three concrete payoffs.

**Short-circuiting on error costs nothing.** The first failed step turns into an `error.is_some()` on the carried process, and every subsequent `bind` is a no-op that preserves the logs. You do not write `?` propagation by hand inside the chain.

**Logs accumulate without instrumentation.** `LogAppend::append` runs inside every bind. A consumer that wants to print or persist the trace gets the full ordered sequence with no side channel.

**Refactoring stays safe.** The laws guarantee that breaking a long chain into helper functions, or composing several chains into a larger one, does not change the result. You get the refactoring confidence that pure functional code usually offers.

## Where to look next

[Effect Propagation Process](/docs/concepts/effect-propagation-process/) is the carrier this trait operates over. [HKT](/docs/concepts/hkt/) explains how the signature stays generic across the five parameters without runtime cost. [Causaloid](/docs/concepts/causaloid/) is what produces the values that flow through.
