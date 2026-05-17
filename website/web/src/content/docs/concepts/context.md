---
title: Context
description: The explicit hypergraph that a dynamic causal rule reasons against.
section: concepts
order: 3
---

A `Context` is the explicit environment a [Causaloid](/docs/concepts/causaloid/) reasons against. It is the half of dynamic causality that holds the world while the rules hold the structure.

Concretely, a Context is a typed weighted hypergraph whose nodes are `Contextoid`s. Most production work uses one Context per system, mutated in place across the lifetime of a run.

## The type

The Rust definition (parent crate `deep_causality`, file `src/types/context_types/context_graph/mod.rs`):

```rust
pub struct Context<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: ContextId,
    name: String,
    base_context: UltraGraphWeighted<Contextoid<D, S, T, ST, SYM, VS, VT>, u64>,
    id_to_index_map: HashMap<ContextoidId, usize>,
    extra_contexts: Option<ExtraContextMap<…>>,
    number_of_extra_contexts: u64,
    extra_context_id: u64,
    current_data_map: HashMap<usize, usize>,
    previous_data_map: HashMap<usize, usize>,
    current_index_map: HashMap<usize, usize>,
    previous_index_map: HashMap<usize, usize>,
}
```

Seven generic parameters look intimidating; they are how the library remains polymorphic across Euclidean, non-Euclidean, temporal, and abstract relational settings. The `BaseContext` alias pins all seven to sensible defaults, and most code reaches for the alias.

## Contextoids

A `Contextoid` is the atomic unit of context. It carries an id and a typed payload:

- **`Datoid`**: arbitrary data with a name and value. The everyday case for tunable thresholds, model parameters, current state snapshots.
- **`Spaceoid`**: a spatial position or region in the chosen space type.
- **`Tempoid`**: a temporal position or interval.
- **`SpaceTempoid`**: a combined spacetime point or extent.
- **`Symboid`**: a symbolic entity (a label, a category, an external reference).

Contextoids are *not* recursive. A Contextoid cannot contain another Contextoid. The monograph treats this as a deliberate guard against self-referential paradox; the engineering payoff is that walking the graph stays predictable.

## Adding nodes and edges

```rust
use deep_causality::{BaseContext, Context, Contextoid, ContextoidType, Data, Time};
use std::time::SystemTime;

let mut ctx: BaseContext = Context::with_capacity(1, "trading", 64);

let threshold = Contextoid::new(
    10,
    ContextoidType::Datoid(Data::new(10, "volume_threshold".into(), 1_500.0)),
);
let now = Contextoid::new(
    20,
    ContextoidType::Tempoid(Time::new(20, "now".into(), SystemTime::now())),
);

let i_thresh = ctx.add_node(threshold);
let i_now = ctx.add_node(now);
ctx.add_edge(i_thresh, i_now, /* weight = */ 1)?;
```

Nodes are addressed by id (`u64`) at the public surface; the library maintains a private id→index map so queries stay O(1) regardless of insertion order. Edges carry a weight; the `u64` default suits most use cases and can be lifted by reaching past the `BaseContext` alias.

## Mutating in place

This is what makes the model dynamic.

```rust
let updated = Contextoid::new(
    10,
    ContextoidType::Datoid(Data::new(10, "volume_threshold".into(), 3_000.0)),
);
ctx.update_node(10, updated)?;
```

The Causaloids that read from this Context do not need to be rebuilt. They evaluate against whatever the Context currently holds. The `previous_data_map` field on `Context` preserves a one-step history, so a rule can compare *now* against *just-before-now* when the change itself is the relevant signal.

## Counterfactuals via extra contexts

The `extra_contexts` field carries parallel hypothetical contexts. Build a counterfactual the same way you build the primary Context, register it under an `extra_context_id`, and evaluate the same Causaloid against it.

```rust
let alt_id = ctx.add_extra_context();
ctx.with_extra(alt_id, |alt| {
    alt.update_node(10, /* counterfactual threshold */)?;
    Ok(())
})?;
let alt_effect = signal.evaluate_with_extra(&tick, alt_id)?;
```

Nothing on the primary Context is disturbed. The library treats counterfactual reasoning as a configuration of the same machinery rather than as a separate engine.

## When to add to the Context

A value belongs in the Context when one of these is true:

- The value changes during a run, and the rules need to see the change.
- The value is set externally and tunable by an operator.
- The value is something a counterfactual run might want to replace.
- The value is a shared piece of state that more than one Causaloid reads from.

Values that fail every test stay in the closure. The Context is not a junk drawer; it is the structured shared state.

## Where to look next

[Causaloid](/docs/concepts/causaloid/) is the rule that reads the Context. [Effect Propagation Process](/docs/concepts/effect-propagation-process/) is what the rule produces. [Effect Ethos](/docs/concepts/effect-ethos/) is what verifies the rule's output before it commits.
