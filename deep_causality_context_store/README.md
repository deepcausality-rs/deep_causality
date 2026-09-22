# deep_causality_context_store

The persistence contract for the [DeepCausality](https://deepcausality.com/) context: the record
vocabulary a `Context` projects onto, the storage trait a backend implements, and the vocabulary
types both sides name.

## What this crate is for

`deep_causality_context` holds a `Context` in memory. A store that keeps one needs a description of
what a context is made of that does not change when the engine's structs change. This crate is
that description. It depends on nothing, so a backend that implements the storage trait links this
crate alone and never the context crate or its dependencies.

```toml
[dependencies]
deep_causality_context_store = "0.1"
```

## Contents

* `RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef` — the vocabulary the records
  and the context node types share. `deep_causality_context` re-exports all four.

## Licence

MIT. See [LICENSE](LICENSE).
