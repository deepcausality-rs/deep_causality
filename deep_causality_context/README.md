# deep_causality_context

The context layer of [DeepCausality](https://deepcausality.com/): the `Context` hypergraph, the
contextoids that populate it, the context node types for data, space, time and spacetime, and the
traits that describe them.

## What this crate is for

Every causal model reasons about something. The context holds that something — a typed environment
a causaloid or a causal-monad chain reads as it runs.

Splitting context into its own crate lets a consumer on `deep_causality_core` alone pay nothing for
it: a model that needs context declares this dependency and imports from it. `deep_causality`
depends on this crate unconditionally, so a causaloid model gets context either way. Listing the
crates that depend on this one directly therefore answers which crates model a context.

```toml
[dependencies]
deep_causality_context = "0.1"
```

## Using context with the causal monad

A `deep_causality_core`-only dependency set reaches the context, so a monad chain carries a typed
context without pulling in the causaloid stack:

```rust,ignore
use deep_causality_context::BaseContext;
use deep_causality_core::{PropagatingEffect, PropagatingProcess};

fn start(ctx: BaseContext) -> PropagatingProcess<f64, (), BaseContext> {
    PropagatingProcess::with_state(PropagatingEffect::pure(0.0), (), Some(ctx))
}
```

## Contents

* `Context` — the context hypergraph, with extra contexts and data/time indices.
* `Contextoid` / `ContextoidType` — the nodes it holds.
* Context node types — `Data`, `Root`, the space, spacetime, symbolic-spacetime and time families.
* `Contextuable`, `Datable`, `Spatial`, `Temporal`, `SpaceTemporal`, `Coordinate`, `Distance`,
  `MetricSignature`, `MetricTensor4D` and the indexable traits.
* `Adjustable<T>` for nodes that update from an `ArrayGrid<T, ..>`, and `UncertainAdjustable` for
  nodes that update from their own associated `Self::Data`.

## The scalar is a parameter

Spatial, spacetime and real-valued temporal types carry the scalar they measure in, bounded once on
`deep_causality_algebra::RealField`. The crate names no concrete float outside its ready-made
aliases, so a type satisfying that bound works without an entry being added for it:

```rust,ignore
let narrow: EuclideanSpace<BFloat16> = EuclideanSpace::new(1, x, y, z);
let wide: EuclideanSpace<Float106> = EuclideanSpace::new(1, x, y, z);
```

`BaseContext`, `BaseContextoid`, `UniformContext` and `UniformContextoid` stay concrete, so reaching
the context through one of those needs no annotation.

Tick-based times keep their integers. A tick counts, and its failure mode is overflow rather than
rounding, so widening its significand buys nothing.

## Licence

MIT. See [LICENSE](LICENSE).
