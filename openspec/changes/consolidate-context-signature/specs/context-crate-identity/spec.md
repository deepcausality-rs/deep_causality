## MODIFIED Requirements

### Requirement: The context layer lives in `deep_causality_context`

The crate `deep_causality_context` SHALL own the context layer in full: the `Context` hypergraph,
`Contextoid`, `ContextoidType`, `RelationKind` and `TimeScale`; the context node types for data,
space, spacetime, symbolic spacetime and time; the `Contextuable`, `Coordinate`, `Datable`,
`UncertainDatable`, `Distance`, `MetricTensor4D`, `SpaceTemporal`, `SpaceTemporalInterval`,
`Spatial` and `Temporal` traits; `ContextFrame`; `ContextuableGraph` and
`ExtendableContextuableGraph`; `Adjustable` and `UncertainAdjustable`; the data- and time-indexable
traits; `ScalarProjector` and `ScalarValue`; the `ContextIndexError`, `IndexError`,
`AdjustmentError` and `UpdateError` error types; and the `BaseContext`, `BaseContextoid`,
`UniformContext` and `UniformContextoid` aliases.

No moved item SHALL change its semantics, its signature or its public name as part of a move
between crates. Signature changes made deliberately, rather than as a side effect of relocation,
are stated by the capability that makes them: `context-data-node-payload` relaxes the `Data<T>`
payload bound, `context-symbolic-dimension-removed` withdraws the symbolic dimension,
`context-associated-value-types` replaces the value parameters and renames the distance trait,
`context-scalar-parameter` makes the node types generic in their scalar, and `context-frame`
consolidates the remaining parameters.

#### Scenario: The context hypergraph is constructed from the new crate

- **WHEN** a consumer depends on `deep_causality_context` and calls `Context::with_capacity`, adds a
  `Contextoid` holding a `Root`, and reads the result back through `ContextuableGraph`
- **THEN** the context is built and read exactly as the same calls behaved before the move

#### Scenario: Moved behaviour is pinned by the tests that moved with it

- **WHEN** the test suite of `deep_causality_context` runs
- **THEN** it contains the test files that moved out of `deep_causality/tests`, and every one of
  them passes without an assertion having been weakened or removed

### Requirement: The crate does not depend on `deep_causality`

`deep_causality_context` SHALL declare no dependency on `deep_causality`, so the dependency edge
between the two runs in one direction only. Its dependencies are `deep_causality_algebra`,
`deep_causality_core`, `deep_causality_data_structures`, `deep_causality_metric`,
`deep_causality_uncertain` and `ultragraph`.

`deep_causality_core` SHALL NOT depend on `deep_causality_context`, so that core keeps its `no-std`
feature and its single-dependency footprint.

#### Scenario: The dependency direction holds

- **WHEN** the `[dependencies]` table of `deep_causality_context/Cargo.toml` is read
- **THEN** `deep_causality` does not appear in it, and neither does any crate that depends on
  `deep_causality`

#### Scenario: Core still builds without std

- **WHEN** `deep_causality_core` is built with `--no-default-features --features no-std`
- **THEN** the build succeeds, because no edge to `deep_causality_context` was introduced

#### Scenario: The added dependencies introduce no cycle

- **WHEN** the workspace dependency tiers are re-derived from the manifests
- **THEN** `deep_causality_algebra` and `deep_causality_metric` sit below `deep_causality_context`,
  and the tier block records the crate's position
