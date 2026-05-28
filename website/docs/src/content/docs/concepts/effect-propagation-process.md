---
title: Effect Propagation Process
description: The struct that carries a value, a state, a context, an error, and a log through a chain of Causaloids.
sidebar:
  order: 5
---

The Effect Propagation Process (EPP) is the load-bearing abstraction of DeepCausality. It is the carrier effect: the value that flows between every other piece of the library. The Causaloid, the Context, the Causal State Machine, and the Effect Ethos all exchange work through one type, and that type implements the [Causal Monad](/concepts/causal-monad/) trait so it composes:

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    pub value:   EffectValue<Value>,
    pub state:   State,
    pub context: Option<Context>,
    pub error:   Option<Error>,
    pub logs:    Log,
}
```

This is the runtime realization of the theory described in the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf). The paper reframes causality as a spacetime-agnostic functional dependency between an input and a propagated effect. The struct above is that dependency made concrete and runnable.

## What the EPP contributes

Most causal libraries split reasoning across several incompatible vocabularies. Structural causal models live in one type. Sequential probabilistic chains live in another. State is held in a host struct. Errors propagate through `Result`. Logs end up in a tracing subscriber. Context is encoded implicitly. The EPP collapses that fragmentation. One container carries everything a chain needs to know about itself:

1. **Unified carrier for heterogeneous reasoning.** Structural reasoning (a Causaloid Singleton, Collection, or Graph) and sequential reasoning (a Causal Monad bind-chain) both return an EPP. The two reasoning styles share a single boundary type, which is what makes them composable without bridge code. A graph emits an EPP, a `bind` consumes it, another graph evaluates against the result. Nothing translates between worlds.

2. **Non-Markovian and Markovian under one type.** The same struct represents both. `PropagatingEffect<T>` fixes `State = ()` and `Context = ()`; each step depends only on its input, and the chain stays Markov-free. `PropagatingProcess<T, S, C>` keeps the state and context generic; each step receives the threaded state and context, and the chain becomes Markovian. Lifting between the two is a single constructor call. The boundary is real, but it is movable.

3. **Audit and replay.** Because the EPP carries the log inline with the value, every step appends to the same record, and a chain can be replayed off disk with no missing context. There is no separate tracing infrastructure to align, no out-of-band state to reconstruct.

## The EffectValue Type 

**`value`**: the propagating effect's payload, wrapped in an `EffectValue<T>` enum:

```rust
pub enum EffectValue<T> {
    None,
    Value(T),
    ContextualLink(ContextoidId, ContextoidId),
    RelayTo(usize, Box<PropagatingEffect<T>>),
    #[cfg(feature = "std")]
    Map(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>),
}
```

`None` is an explicit *no effect*. `Value(T)` is the everyday case. `ContextualLink` says "the value is whatever the Context says it is at these two ids" and defers the fetch. `RelayTo` is a dispatch command: route this effect to the rule at index N. `Map` carries a labeled bundle of sub-effects.

A Causaloid's wrapped function returns a `PropagatingEffect<T>` whose `value` is one of those variants. The richer variants exist so that downstream rules can do work for the upstream rule without losing the audit trail in between.

**`state`**: caller-supplied state threaded through the chain. For the stateless case, `State = ()` and the field carries no information.

**`context`**: an optional `Context` value. When a contextual Causaloid runs it threads the Context through here; when a stateless rule runs it stays `None`.

**`error`**: `Option<Error>`. The chain short-circuits when this is `Some`. The presence of an error does not stop the log from accumulating; the failure point is recorded with everything else.

**`logs`**: an append-only `EffectLog`. Every Causaloid that runs adds an entry. The log is the audit trail.

## The aliases

You will rarely instantiate the five-parameter form by hand. The library ships two pinned aliases.

```rust
// Stateless, contextless. The everyday case.
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;
```

```rust
// Stateful, with a typed context. The dynamic case.
type CausalProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

`PropagatingEffect<T>` is what a `Causaloid::from_causal_fn` closure returns. `CausalProcess<T, S, C>` is the stateful form a chain operates over when it threads state and context through [`bind`](/concepts/causal-monad/).

## How the process moves

A single Causaloid call takes an input, produces a `PropagatingEffect`, and returns. The "process" emerges when Causaloids compose. Each rule in the chain consumes the upstream effect, performs its own computation, and produces a new effect. The chain accumulates:

- The latest `value`.
- The threaded `state` (updated in place when stateful).
- The shared `context` (mutable or readonly depending on the configuration).
- The first encountered `error`, after which propagation stops.
- The growing `logs`, regardless of error state.

The composition is provided by the [Causal Monad](/concepts/causal-monad/) and its `bind` operation. Conceptually:

```
m₁ >>= f   →   m₂
```

`m₁` is the upstream `CausalEffectPropagationProcess`. `f` is the next Causaloid's function. `m₂` is the new process: the new value sits in `m₂.value`, the threaded state in `m₂.state`, the merged logs in `m₂.logs`, and any error surfaces in `m₂.error`.

## Inspecting an effect

A consumer typically pattern-matches on `EffectValue`:

```rust
match effect.value {
    EffectValue::Value(v)                 => commit(v)?,
    EffectValue::None                     => skip(),
    EffectValue::ContextualLink(a, b)     => resolve_link(&ctx, a, b)?,
    EffectValue::RelayTo(idx, sub)        => dispatch(idx, *sub)?,
    EffectValue::Map(parts)               => fan_out(parts)?,
}
```

The `error` field is checked before this match. The `logs` field is appended to the persistent audit log on every emission regardless of outcome.

## Why a five-field record

The five fields are the irreducible set. Drop any one and a contribution from the list above collapses:

- Without **`value`** there is nothing to propagate.
- Without **`state`** the Markovian case cannot be expressed without a separate type, and the unification falls apart.
- Without **`context`** spatial, temporal, and symbolic conditioning leak out into ambient state.
- Without **`error`** the chain cannot short-circuit cleanly, and partial-failure replay becomes guesswork.
- Without **`logs`** audit and replay stop being intrinsic and become an external concern again.

DeepCausality keeps the five together to enable verifiable end-to-end reasoning. A test that replays an effect off disk has everything. A debugger that wants to step backward through a propagation has everything for fine-grained diagnostics.

## Where to look next

[Causal Monad](/concepts/causal-monad/) is the algebra that composes processes. [HKT](/concepts/hkt/) is how the algebra is encoded in Rust's type system. [Causaloid](/concepts/causaloid/) is what produces the processes in the first place. [Effect Ethos](/concepts/effect-ethos/) is what verifies the actions an EPP chain ultimately proposes. The [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) is the formal treatment of the model this page implements.
