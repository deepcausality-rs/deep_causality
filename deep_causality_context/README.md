# deep_causality_context

The context layer of [DeepCausality](https://deepcausality.com/): the `Context` hypergraph, the
contextoids that populate it, the context node types for data, space, time and spacetime, and the
traits that describe them.

## What this crate is for

The context holds what a causal model reasons about: a typed environment that a causaloid or a
causal-monad chain reads as it runs.

A consumer of `deep_causality_core` alone pays nothing for context; a model that needs context
declares this dependency and imports from it. `deep_causality` depends on this crate
unconditionally, so a causaloid model gets context either way. The crates that depend on this one
directly are the crates that model a context.

```toml
[dependencies]
deep_causality_context = "0.1"
```

## Using context with the causal monad

With `deep_causality_core` and this crate, a monad chain carries a typed context without pulling
in the causaloid stack:

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
* `Storable`, `ContextStore`, `StoreError`, `Context::snapshot`, `Context::restore` and
  `Context::apply` for persistence, with the `SubstrateContext` and `SubstrateContextoid` aliases.
* `RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef`, re-exported from
  `deep_causality_context_store`, so a model names them from this crate alone.

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

## Persistence

A context lives in memory. `deep_causality_context_store` describes what a store keeps of one;
this crate projects in both directions.

`Context::snapshot` writes the base graph and every extra context into a `ContextSnapshot`, in
canonical order, and `Context::restore` builds a context from one. A `ContextStore` puts the
projection and a backend together:

```rust,ignore
use deep_causality_context::{ContextStore, UniformContext};

let store = ContextStore::new(backend);
let ctx: UniformContext = store.hydrate(&container).await?;
// explore a branch in memory, then keep it:
let branch_id = store.store_branch("branch", &branch).await?;
```

`store_branch` links every node the store already holds under the same record, creates every node
it does not hold, and creates under a fresh identifier every node the store holds under a different
record, all in one `commit`, so a refused store leaves nothing behind. A stored branch is a record
of a world, not a continuation of one; to keep exploring it, hydrate it. With a backend that
streams, `subscribe` returns a context and the stream of changes to it, and `Context::apply` applies
each event idempotently.

### Payloads

A data payload persists once its type implements `Storable`, which maps it onto a `DataRecord`
value tree. A struct maps to `Fields`, one entry per field:

```rust
use deep_causality_context::Storable;
use deep_causality_context_store::{ContextoidId, DataRecord, ProjectionError};

#[derive(Debug, Clone, Default, PartialEq)]
struct Reading {
    temperature: f64,
    samples: u64,
}

impl Storable for Reading {
    fn to_record(&self) -> DataRecord {
        DataRecord::Fields(vec![
            ("temperature".to_string(), self.temperature.to_record()),
            ("samples".to_string(), self.samples.to_record()),
        ])
    }

    fn from_record(id: ContextoidId, record: DataRecord) -> Result<Self, ProjectionError> {
        let DataRecord::Fields(entries) = record else {
            return Err(ProjectionError::WrongPayload(id, "Fields", record.kind_name()));
        };
        let field = |name: &'static str| {
            entries
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
                .ok_or(ProjectionError::MissingField(id, name))
        };
        Ok(Self {
            temperature: f64::from_record(id, field("temperature")?)?,
            samples: u64::from_record(id, field("samples")?)?,
        })
    }
}
```

`Data<Reading>` then snapshots, restores, stores and streams like `Data<f64>`. The crate implements
`Storable` for `f32`, `f64`, `u64`, `i64`, `bool`, `String`, `SubstrateRef`, `Vec<T>`,
`Option<T>`, `Float106` and `BFloat16`.

A store that holds structure and not values takes a data node as a `SubstrateRef`. `SubstrateContext`
names that shape, and `ContextStore::create_node_via` and `hydrate_via` move the values through a
`Substrate` on the way in and out, so a value-typed context reaches such a store unchanged.

### Extra contexts

An extra context has a name and an identifier. `extra_ctx_add_new(name, capacity, default)`
allocates the identifier as one past the highest ever held; `extra_ctx_add_new_with_id` takes one, and
refuses 0. In a store an extra is a separate container the base references, so a hydrated context
holds each referenced container as an extra under that container's identifier and name, and a
stored branch's extras become containers of their own.

### Precision in the store

Every scalar field of a record is `f64`. A node type over a wider or narrower scalar narrows to
`f64` on the way out and lifts on the way in, so `EuclideanSpace<Float106>` coordinates restore at
double precision and `EuclideanSpace<BFloat16>` coordinates restore rounded as that type rounds.
Ticks stay `u64`. The exception is a `Float106` data payload: `Storable` stores its two halves as
`Fields [("hi", Number), ("lo", Number)]` and restores every bit.
An `f32` payload widens exactly and reads back exactly; a `Number` it did not write rounds to
the nearest `f32`, a magnitude too small for the smallest subnormal becomes a zero of the same sign, and a
finite value beyond the `f32` range is `ProjectionError::Scalar`.

## Licence

MIT. See [LICENSE](LICENSE).
