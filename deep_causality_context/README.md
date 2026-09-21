# deep_causality_context

The context layer of [DeepCausality](https://deepcausality.com/): the `Context` hypergraph, the
contextoids that populate it, the context node types for data, space, time, spacetime and symbols,
and the traits that describe them.

## What this crate is for

A causal model reasons about something. That something is the context: an explicit, typed
environment a causaloid or a causal-monad chain reads while it runs.

This crate is separate from `deep_causality` so that context is an **opt-in** dependency. Reasoning
without context is the default case and costs nothing; a model that needs context declares this
crate and imports from it. A dependency search therefore answers "which crates use context?"
correctly.

```toml
[dependencies]
deep_causality_context = "0.1"
```

## Using context with the causal monad

The context is reachable from a `deep_causality_core`-only dependency set, so a monad chain can
carry a typed context without depending on the causaloid stack:

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
* Context node types — `Data`, `Root`, the space, spacetime, symbol and time families.
* `Contextuable`, `Datable`, `Spatial`, `Temporal`, `SpaceTemporal`, `Symbolic`, `Coordinate`,
  `Metric` and the indexable traits.
* `Adjustable` / `UncertainAdjustable` for nodes that update from an `ArrayGrid`.

## Licence

MIT. See [LICENSE](LICENSE).
