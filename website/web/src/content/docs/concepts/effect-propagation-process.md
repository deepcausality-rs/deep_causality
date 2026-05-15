---
title: Effect Propagation Process
description: The struct that carries a value, a state, a context, an error, and a log through a chain of Causaloids.
section: concepts
order: 5
---

The Effect Propagation Process is not a metaphor. It is a Rust type:

```rust
pub struct CausalEffectPropagationProcess<Value, State, Context, Error, Log> {
    pub value:   EffectValue<Value>,
    pub state:   State,
    pub context: Option<Context>,
    pub error:   Option<Error>,
    pub logs:    Log,
}
```

This struct is the unit of work that flows between Causaloids. The five fields are everything a downstream rule needs to know about the upstream rule's output, and everything the surrounding machinery needs to reason about the chain as a whole.

## The five fields

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

The variants are not arbitrary. `None` is an explicit *no effect*. `Value(T)` is the everyday case. `ContextualLink` says "the value is whatever the Context says it is at these two ids" and defers the fetch. `RelayTo` is a dispatch command: route this effect to the rule at index N. `Map` carries a labelled bundle of sub-effects, useful for branching results.

A Causaloid's wrapped function returns a `PropagatingEffect<T>` whose `value` is one of those variants. The richer variants exist so that downstream rules can do work the upstream rule could not yet do, without losing the audit trail in between.

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

`PropagatingEffect<T>` is what a `Causaloid::from_causal_fn` closure returns. `CausalProcess<T, S, C>` is what a Causaloid running under a [Causal Monad](/docs/concepts/causal-monad/) operates over.

## How the process moves

A single Causaloid call takes an input, produces a `PropagatingEffect`, and returns. The "process" emerges when Causaloids compose. Each rule in the chain consumes the upstream effect, performs its own computation, and produces a new effect. The chain accumulates:

- The latest `value`.
- The threaded `state` (updated in place when stateful).
- The shared `context` (mutable or readonly depending on the configuration).
- The first encountered `error`, after which propagation stops.
- The growing `logs`, regardless of error state.

The composition is provided by the [Causal Monad](/docs/concepts/causal-monad/) and its `bind` operation. Conceptually:

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

Most libraries pick two or three of the five and put the rest somewhere else. State sits in a parent struct, errors propagate through `Result`, logs live in a tracing subscriber, contexts hide inside thread locals. That works until you need to reason about the chain as a whole, at which point the pieces have to be reassembled from scattered sources.

DeepCausality keeps the five together because the chain *is* the five together. A test that replays an effect off disk has everything. A debugger that wants to step backward through a propagation has everything. A counterfactual run that swaps the Context has everything except the Context, which it now controls.

## Common patterns

**Branching with `Map`.** A rule that needs to fan out to multiple downstream Causaloids returns `EffectValue::Map(parts)` where `parts` is a labelled bundle. Each label corresponds to a downstream rule.

**Deferred resolution with `ContextualLink`.** A rule that knows *which Context nodes to compare* but not *what the answer is yet* returns `EffectValue::ContextualLink(a, b)`. A later rule, perhaps after the Context has been refreshed, resolves the link.

**Explicit dispatch with `RelayTo`.** A graph that wants conditional dispatch (rule A goes to rule B if condition X, rule C otherwise) emits `RelayTo(idx, sub_effect)` to route the effect.

## What it is not

The Effect Propagation Process is not a stream type. It does not implement `Iterator` or `Stream`. A run is a value, not a sequence in the iterator sense.

The Effect Propagation Process is not async. The chain runs synchronously inside `evaluate`. You can wrap an entire chain in `tokio::task::spawn_blocking` to push it off an async runtime, but the propagation itself does not yield to a scheduler.

The Effect Propagation Process is not an event bus. It does not publish; it returns.

## Where to look next

[Causal Monad](/docs/concepts/causal-monad/) is the algebra that composes processes. [HKT](/docs/concepts/hkt/) is how the algebra is encoded in Rust's type system. [Causaloid](/docs/concepts/causaloid/) is what produces the processes in the first place.
