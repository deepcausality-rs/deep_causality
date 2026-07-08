---
title: Hello, Context
description: Build a Context hypergraph and let a Causaloid read from it.
sidebar:
  order: 5
---

The Causaloid in the [previous page](/getting-started/hello-causaloid/) took an input and returned an effect. That covers a surprising amount of practical work. However, it stops the moment a rule needs to know something about the world beyond its input. The [`Context`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) is the explicit place that world lives.

## What a Context is

A Context is a typed weighted hypergraph whose nodes are `Contextoid`s, each one carrying a typed payload: data, space, time, spacetime, or symbolic. The graph can be queried by id, walked along its edges, and mutated in place. Mutating it is the *dynamic* in dynamic causality: the same Causaloid evaluated against a new Context yields a new propagating effect. The [Context concept page](/concepts/context/) explains this further.

The `BaseContext` alias pins the seven generic parameters of `Context` to a sensible Euclidean default, which is what every example below uses. The base context lives in [`deep_causality`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) and is built on top of the [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph) hypergraph backend.

## Build a Context

We will give a rule a tunable volume threshold by putting it in the Context as a Datoid.

```rust
use deep_causality::{
    BaseContext, ContextoidType, Contextuable, ContextuableGraph, Context, Contextoid, Data,
};

fn build_context() -> BaseContext {
    let mut ctx: BaseContext = Context::with_capacity(1, "trading", 8);

    let threshold = Contextoid::new(
        10,
        ContextoidType::Datoid(Data::new(10, 1_500.0)),
    );
    let _idx = ctx.add_node(threshold);

    ctx
}
```

Three things to notice.

`Context::with_capacity(id, name, capacity)` takes an integer id, a name, and a pre-allocated capacity for the underlying hypergraph. The name shows up in logs; the capacity is a hint, not a hard cap.

`Data::new(id, value)` carries a numerical payload, and `Contextoid::new(id, ContextoidType::Datoid(data))` wraps it as a node. The other `ContextoidType` variants — `Spaceoid`, `Tempoid`, `SpaceTempoid`, `Symboid` — wrap the other four payload kinds.

`add_node` returns the node's index in the graph. The library also maintains an id-to-index map internally so queries by id stay O(1) regardless of insertion order.

## A context-aware Causaloid

A context-aware Causaloid uses [`Causaloid::new_with_context`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) and a slightly richer function signature than the stateless case. The context is shared, so the library wraps it in `Arc<RwLock<BaseContext>>`.

```rust
use deep_causality::{
    BaseCausaloid, BaseContext, Causaloid, ContextoidType, Contextuable, MonadicCausable,
    PropagatingEffect, PropagatingProcess,
};
use deep_causality_core::CausalEffect;
use std::sync::{Arc, RwLock};

fn volume_above_threshold(
    obs: CausalEffect<f64>,
    _state: (),
    ctx: Option<Arc<RwLock<BaseContext>>>,
) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
    let Some(volume) = obs.into_value() else {
        return PropagatingProcess::from_error(
            deep_causality::CausalityError::new(
                deep_causality_core::CausalityErrorEnum::Custom("missing input".into()),
            ),
        );
    };
    let Some(ctx_arc) = ctx else {
        return PropagatingProcess::from_error(
            deep_causality::CausalityError::new(
                deep_causality_core::CausalityErrorEnum::Custom("missing context".into()),
            ),
        );
    };

    let ctx_lock = ctx_arc.read().unwrap();
    let node = ctx_lock.get_node(0).expect("threshold contextoid missing");

    let threshold = match node.vertex_type() {
        ContextoidType::Datoid(d) => d.data(),
        _ => return PropagatingProcess::from_error(
            deep_causality::CausalityError::new(
                deep_causality_core::CausalityErrorEnum::Custom("wrong contextoid kind".into()),
            ),
        ),
    };

    PropagatingProcess::pure(volume > threshold)
}

fn main() {
    let ctx = Arc::new(RwLock::new(build_context()));

    let causaloid: BaseCausaloid<f64, bool> = Causaloid::new_with_context(
        100,
        volume_above_threshold,
        ctx.clone(),
        "volume exceeds the threshold from context",
    );

    let effect = PropagatingEffect::pure(2_400.0_f64);
    let result = causaloid.evaluate(&effect);
    println!("{:?}", result.value());
}
```

Three things changed compared to the stateless Causaloid:

- The function signature is `fn(CausalEffect<I>, S, Option<C>) -> PropagatingProcess<O, S, C>`. The first argument is a `CausalEffect`, not a bare `I`, because the upstream `PropagatingEffect` can carry a `None` or a `RelayTo` command as well as a value. The second is the threaded state, ignored here because `BaseCausaloid` pins it to `()`. The third is the optional context.
- The return type is a `PropagatingProcess`, the stateful sibling of `PropagatingEffect`. For a stateless context-aware rule like this one the state stays `()`, but the type is shared with truly stateful reasoning so the trait machinery composes uniformly.
- The Context is wrapped in `Arc<RwLock<...>>` so it can be shared across Causaloids and mutated in place. Read access uses `ctx_lock.get_node(...)`; mutation uses `ctx_lock.update_node(...)`.

## Mutating the Context

This is what makes the model dynamic. Replace the threshold and the same Causaloid produces a different effect on the same input:

```rust
use deep_causality::{ContextoidType, ContextuableGraph, Contextoid, Data};

let new_threshold = Contextoid::new(
    10,
    ContextoidType::Datoid(Data::new(10, 3_000.0)),
);

ctx.write().unwrap().update_node(10, new_threshold).unwrap();
```

The Causaloid never moved. The rule never moved. The world moved. The library treats that as a first-class case.

The [Context concept page](/concepts/context/) covers the related `Adjustable` trait, which exposes `update` and `adjust` methods on node payloads for the case where the structure stays fixed but incoming sensor data needs to be replaced or corrected for drift.

## When to add to the Context

A rule of thumb: if the rule needs to see a value that changes during a run, a value that an operator might tune without rebuilding, or a value that a counterfactual scenario might want to replace, that value belongs in the Context. Also, all external data feeds usually go through the context first.

## What's next

You now have the core moving parts: a Causal Monad, a Causaloid, a Context, and the propagating effect that flows between them. The [next page](/getting-started/hello-effect-propagation/) shows how these compose into one chain, and how a non-Markovian `PropagatingEffect` lifts into a Markovian `PropagatingProcess` when a downstream step needs state. The concept pages cover each piece in more detail, and the [examples](https://www.deepcausality.com/examples/) show what the primitives let you build.
