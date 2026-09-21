## ADDED Requirements

### Requirement: The context layer lives in `deep_causality_context`

The crate `deep_causality_context` SHALL own the context layer in full: the `Context` hypergraph,
`Contextoid`, `ContextoidType`, `RelationKind` and `TimeScale`; the context node types for data,
space, spacetime, symbol, symbolic spacetime and time; the `Contextuable`, `Coordinate`, `Datable`,
`UncertainDatable`, `Metric`, `MetricCoordinate`, `MetricTensor4D`, `SpaceTemporal`,
`SpaceTemporalInterval`, `Spatial`, `Symbolic` and `Temporal` traits; `ContextuableGraph` and
`ExtendableContextuableGraph`; `Adjustable` and `UncertainAdjustable`; the data- and time-indexable
traits; `ScalarProjector` and `ScalarValue`; the `ContextIndexError`, `IndexError`,
`AdjustmentError` and `UpdateError` error types; and the `BaseContext`, `BaseContextoid`,
`UniformContext` and `UniformContextoid` aliases.

No moved item SHALL change its semantics, its signature or its public name as part of the move, with
one stated exception: the `Data<T>` payload bound is relaxed as `context-data-node-payload`
requires.

#### Scenario: The context hypergraph is constructed from the new crate

- **WHEN** a consumer depends on `deep_causality_context` and calls `Context::with_capacity`, adds a
  `Contextoid` holding a `Root`, and reads the result back through `ContextuableGraph`
- **THEN** the context is built and read exactly as the same calls behaved before the move

#### Scenario: Moved behaviour is pinned by the tests that moved with it

- **WHEN** the test suite of `deep_causality_context` runs
- **THEN** it contains the 62 test files that moved out of `deep_causality/tests`, and every one of
  them passes without an assertion having been weakened or removed

### Requirement: The crate does not depend on `deep_causality`

`deep_causality_context` SHALL declare no dependency on `deep_causality`, so the dependency edge
between the two runs in one direction only. Its dependencies are `deep_causality_core`,
`deep_causality_data_structures`, `deep_causality_uncertain` and `ultragraph`.

`deep_causality_core` SHALL NOT depend on `deep_causality_context`, so that core keeps its `no-std`
feature and its single-dependency footprint.

#### Scenario: The dependency direction holds

- **WHEN** the `[dependencies]` table of `deep_causality_context/Cargo.toml` is read
- **THEN** `deep_causality` does not appear in it, and neither does any crate that depends on
  `deep_causality`

#### Scenario: Core still builds without std

- **WHEN** `deep_causality_core` is built with `--no-default-features --features no-std`
- **THEN** the build succeeds, because no edge to `deep_causality_context` was introduced

### Requirement: The typed context is reachable from a monad-only dependency set

A consumer SHALL be able to name the typed context and carry it through the causal monad's context
channel while depending on `deep_causality_core` and `deep_causality_context` only, and not on
`deep_causality`.

This is the reachability guarantee the extraction exists to provide. The monad's `Context` type
parameter remains unbounded; this requirement is about what a consumer can instantiate it with, not
about a new bound.

#### Scenario: A monad chain carries a typed context without the causaloid crate

- **WHEN** a crate depends on `deep_causality_core` and `deep_causality_context` only, and builds a
  `PropagatingProcess<f64, (), BaseContext>` seeded with a real `BaseContext`
- **THEN** it compiles, and `bind` delivers that context to each continuation

#### Scenario: The five migrated examples demonstrate it

- **WHEN** the `rcm_via_monad`, `cate_via_monad`, `scm_via_monad`, `dbn_via_monad` and
  `granger_via_monad` examples run
- **THEN** each carries a context built from `Data` contextoids rather than a locally declared
  struct, and each produces the same factual and counterfactual outcomes it produced before

### Requirement: The crate carries the repository's standard crate infrastructure

`deep_causality_context` SHALL ship the infrastructure every workspace member carries: a
`BUILD.bazel` and `tests/BUILD.bazel`, an SBOM pair, a README, `[lints] workspace = true` so the
repository-wide `unsafe_code = "forbid"` applies, a root `Cargo.toml` member entry, and a
`[workspace.dependencies]` entry at two-digit version precision.

Its `CHANGELOG.md` SHALL NOT be hand-written, because release-plz generates it.

#### Scenario: Bazel and Cargo agree on the test count

- **WHEN** the crate's tests are run under both `cargo test -p deep_causality_context` and
  `bazel test //deep_causality_context/...`
- **THEN** both report the same number of tests, and that number equals the count those tests had
  before the move

#### Scenario: The crate is visible to the derived CI crate list

- **WHEN** `scripts/crates.sh` is sourced
- **THEN** `deep_causality_context` appears in `DC_CRATES` with its directory in `DC_CRATE_DIRS`,
  without any workflow file having been edited by hand
