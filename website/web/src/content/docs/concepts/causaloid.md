---
title: Causaloid
description: A self-contained unit of causality that composes into larger units of itself.
section: concepts
order: 2
---

A `Causaloid` is the fundamental unit of causality in DeepCausality. Three properties define it.

1. It wraps a function that takes input, optionally consults a context, and returns a `PropagatingEffect`.
2. It carries enough metadata (id, name, description) to remain identifiable when it shows up in a log.
3. It composes isomorphic-recursively. A Causaloid, a collection of Causaloids, and a graph of Causaloids all implement the same `Causable` + `MonadicCausable` trait surface, so each one stands in for any other and they nest into each other without limit.

The third property is the load-bearing one. It is borrowed from physicist Lucian Hardy's work on quantum gravity, where a *causaloid* folds cause and effect into a single object so that causal structure can be discussed without assuming a fixed temporal order.

## The type

The Rust definition lives in [`deep_causality/src/types/causal_types/causaloid/mod.rs`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/causal_types/causaloid/mod.rs):

```rust
pub struct Causaloid<I, O, STATE, CTX>
where
    I: Default,
    O: Default + Debug,
    STATE: Default + Clone,
    CTX: Clone,
{
    id: IdentificationValue,
    causal_type: CausaloidType,
    causal_fn: Option<CausalFn<I, O>>,
    coll_aggregate_logic: Option<AggregateLogic>,
    coll_threshold_value: Option<NumericalValue>,
    context_causal_fn: Option<ContextualCausalFn<I, O, STATE, CTX>>,
    context: Option<CTX>,
    causal_coll: Option<Arc<Vec<Self>>>,
    causal_graph: Option<Arc<CausaloidGraph<Self>>>,
    description: String,
    _phantom: PhantomData<(I, O, STATE, CTX)>,
}
```

Four generic parameters do real work. `I` is the input type; `O` is the output effect's value type. `STATE` carries any per-evaluation state the rule wants to thread through. `CTX` is the context type. For the common case where you do not need state or context, the `BaseCausaloid<I>` alias pins them to `()` and `BaseContext`.

## The structure

A Causaloid is one of three shapes, recorded in `CausaloidType`:

- **Singleton**: a single causal function. The atomic case.
- **Collection**: a native Rust collection of Causaloids evaluated together under an `AggregateLogic` (conjunction, disjunction, threshold). The "many rules, one decision" case. Slices, `VecDeque`, `HashMap`, and `BTreeMap` all pick up the [`MonadicCausableCollection`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/causable/mod.rs) blanket impl, so any of them works.
- **Graph**: a proper `CausaloidGraph<Self>` hypergraph (backed by [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph)). The "rules with structure" case. Order and reachability of evaluation matter, and a single hyperedge can connect more than two Causaloids at once.

The three shapes are *isomorphic-recursive*. A Singleton, a Collection, and a Graph are distinct concrete structures, yet each one implements the same [`Causable`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/causal_types/causaloid/causable.rs) and `MonadicCausable` trait surface. As far as the rest of the library is concerned, each one *is* a Causaloid. That uniformity is what makes them composable into each other. A Causaloid wrapping a Graph can be a node in another Graph, an entry in a Collection, or the operand of a `bind` step. The structure nests to arbitrary depth without the calling code changing shape.

This is the central representational move. Classical causality frameworks force you to pick a structure up front. A Pearl SCM is a graph. A Granger model is a set. A Bayesian network is a graph with a specific edge semantics. Changing the structure means rewriting the model. DeepCausality lets you choose your structure for any specific problem, combine different structures for complex cases, and encapsulate sub-modules into single Causaloids to make larger models manageable and composable.

## Construction

```rust
use deep_causality::{
    AggregateLogic, BaseCausaloid, BaseContext, Causaloid, CausaloidGraph, PropagatingEffect,
};
use std::sync::Arc;

// 1. Stateless, no context. The default for simple rules.
let above_zero: BaseCausaloid<f64> = Causaloid::from_causal_fn(
    1,
    "above_zero",
    "value is greater than zero",
    |x: &f64| Ok(PropagatingEffect::Deterministic(*x > 0.0)),
);

// 2. Contextual. The Causaloid captures an Arc<Context>; the closure
//    receives both the input and the captured context on every call.
let with_ctx: BaseCausaloid<Tick> = Causaloid::from_contextual_causal_fn(
    2, "name", "description", ctx,
    |t: &Tick, c: &Arc<BaseContext>| Ok(PropagatingEffect::Deterministic(true)),
);

// 3. Collection. Aggregates a set of Causaloids under one rule.
let any_of: BaseCausaloid<f64> = Causaloid::from_causal_collection(
    3, "any_of", "any predicate fires",
    vec![above_zero.clone(), other_rule.clone()],
    AggregateLogic::Or,
    None,
);

// 4. Graph. Edges encode dependencies between rules.
let mut g = CausaloidGraph::new();
let root = g.add_root_causaloid(stage_one);
let next = g.add_causaloid(stage_two);
g.add_edge(root, next)?;
let pipeline: BaseCausaloid<Tick> = Causaloid::from_causal_graph(
    4, "pipeline", "two-stage signal", g,
);
```

Each constructor returns the same `Causaloid` type. They differ only in which of the four optional fields are populated. The discriminant is `causal_type`.

## Evaluation

`evaluate` runs the Causaloid against an input and returns a `PropagatingEffect`:

```rust
let effect = pipeline.evaluate(&tick)?;
```

For a Singleton the result is the function's return value. For a Collection the per-element effects are reduced under `AggregateLogic`. For a Graph the children are evaluated in topological order against the parent's effect, and the final node's effect is returned.

Errors short-circuit the chain. The `EffectLog` accumulates regardless, so a failed run still produces an audit trail of where it failed.

## Where to look next

[Context](/docs/concepts/context/) is the structure a contextual Causaloid reads from. [Effect Propagation Process](/docs/concepts/effect-propagation-process/) is the carrier effect that flows through a Causaloid chain. [Causal Monad](/docs/concepts/causal-monad/) is the `pure`/`bind` algebra that carrier implements, so chains compose without losing their properties.
