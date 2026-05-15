---
title: Causaloid
description: A self-contained unit of causality that composes into larger units of itself.
section: concepts
order: 2
---

A `Causaloid` is the fundamental unit of causality in DeepCausality. Three properties define it.

1. It wraps a function that takes input, optionally consults a context, and returns a `PropagatingEffect`.
2. It carries enough metadata (id, name, description) to remain identifiable when it shows up in a log.
3. It composes recursively. A collection of Causaloids is a Causaloid. A graph of Causaloids is a Causaloid. The composition has the same shape as its parts.

The third property is the load-bearing one. It is borrowed from physicist Lucian Hardy's work on quantum gravity, where a *causaloid* folds cause and effect into a single object so that causal structure can be discussed without assuming a fixed temporal order.

## The type

The Rust definition (parent crate `deep_causality`, file `src/types/causal_types/causaloid/mod.rs`):

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

A Causaloid is one of three shapes, recorded in `CausaloidType`:

- **Singleton** — a single causal function. The atomic case.
- **Collection** — a `Vec<Causaloid>` evaluated together under an `AggregateLogic` (conjunction, disjunction, threshold). The "many rules, one decision" case.
- **Graph** — a `CausaloidGraph<Self>`. The "rules with structure" case, where the order and reachability of evaluation matters.

The three shapes are *isomorphic*. A Singleton is a Collection of one element; a Collection is a Graph with no edges. The library exploits that to give you one type to think about rather than three.

## Construction

```rust
use deep_causality::prelude::*;

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

## Why this shape

The Causaloid earns its keep in two places.

It earns it at composition time. Three rules into one looks the same in the code as three thousand into one. The graph constructor accepts whatever shape you build; the evaluation engine does not care how deep the recursion goes.

It earns it at debugging time. A misbehaving production system needs you to ask, *which rule fired, against which inputs, in which order?* The `EffectLog` answers that without instrumentation. Every Causaloid that runs leaves a structured entry with its id, name, and the effect it produced.

## Common patterns

**Stage gates.** Compose a chain of Causaloids that progressively narrow a decision. Each stage is its own Causaloid; the pipeline is the graph.

**Voting.** A Collection under `AggregateLogic::Threshold(n)` returns `Deterministic(true)` when at least `n` of its members fire. Useful for fault-tolerant signals.

**Counterfactuals.** Clone the Context, mutate the relevant Contextoids, evaluate the same Causaloid against the new Context. The result is the counterfactual effect.

**Hot-swap.** A Causaloid's `causal_fn` field is a function pointer, not a trait object. Replacing the rule is a struct-update operation, not a vtable swap.

## What it is not

A Causaloid is not a state machine. It does not advance through phases on its own. Each `evaluate` call is independent.

A Causaloid is not a Bayesian network node. It does not carry conditional probability tables. If you want probabilistic effects, return `PropagatingEffect::Probabilistic(p)` from the wrapped function; the type accepts it but does not infer it.

A Causaloid is not a constraint solver. It does not search; it evaluates. If you need search, build the search loop around the Causaloid.

## Where to look next

[Context](/docs/concepts/context/) is the structure a contextual Causaloid reads from. [Effect Propagation Process](/docs/concepts/effect-propagation-process/) is the type that flows through a Causaloid chain. [Causal Monad](/docs/concepts/causal-monad/) is how the algebra is encoded so that chains compose without losing their properties.
