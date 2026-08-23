## ADDED Requirements

### Requirement: The retired crate stays published and keeps working
`deep_causality_sparse` SHALL remain a published crate and a workspace member for a deprecation window of several months after `deep_causality_linear` is released, and SHALL NOT be yanked.

Already-published dependents resolve `deep_causality_sparse` from crates.io. The window exists so
those releases keep building; yanking would break exactly the consumers it protects. The repository
also never deletes files, so the crate directory stays regardless.

#### Scenario: A prior release still resolves
- **WHEN** a previously published dependent is built from crates.io during the window
- **THEN** it resolves `deep_causality_sparse` and compiles

#### Scenario: The retired crate stays under test
- **WHEN** `bazel test //...` runs during the window
- **THEN** the retired crate is built and its tests execute

#### Scenario: No version is yanked
- **WHEN** the crate's version history is inspected at any point in the migration
- **THEN** no version is marked yanked

### Requirement: The retirement is stated in the crate's own documentation
`deep_causality_sparse` SHALL carry a retirement notice in its README naming `deep_causality_linear` as the successor and stating that the crate receives no further development.

A reader arriving from docs.rs or crates.io sees the README first. A retirement recorded only in a
changelog or a release note does not reach them.

#### Scenario: The notice is the first thing in the README
- **WHEN** the retired crate's README is rendered
- **THEN** the retirement notice appears before any usage documentation
- **AND** it names the successor crate

#### Scenario: The notice ships to the registry
- **WHEN** the final version is published
- **THEN** the notice is present in the published README

### Requirement: The retired crate re-exports rather than duplicates
`deep_causality_sparse` SHALL re-export its public items from `deep_causality_linear` rather than retaining its own copy of the implementation.

Freezing the implementation would make `deep_causality_sparse::CsrMatrix` and
`deep_causality_linear::CsrMatrix` distinct types. A crate depending on both — which is what a
partially migrated dependent looks like — would fail to typecheck. Re-exporting keeps one type, so a
dependent can migrate one module at a time.

#### Scenario: The two paths name one type
- **WHEN** a value produced through the old path is passed to a function expecting the new path
- **THEN** it typechecks

#### Scenario: A partially migrated dependent builds
- **WHEN** a crate imports from both paths in different modules
- **THEN** it compiles and its tests pass

#### Scenario: The old surface is complete
- **WHEN** the retired crate's public surface is compared against its last independent release
- **THEN** every item is still present

### Requirement: The tensor conversion becomes an ordinary part of the tensor crate
The `CausalTensor` ↔ sparse conversion SHALL move into `deep_causality_tensor` unconditionally, and the `tensor-iso` feature SHALL be removed rather than relocated.

The conversion lives today in `deep_causality_sparse/src/extensions/ext_iso.rs` behind
`tensor-iso`, which exists so that sparse users do not pay for an optional dependency on tensor.
Once tensor depends on `deep_causality_linear` outright, that dependency is already paid and the gate
has nothing left to gate. No library crate enables the feature; the only enablements are
`examples/mathematics_examples/Cargo.toml:23` for one example and the sparse crate's own Bazel
targets for its own tests.

#### Scenario: No feature gate remains
- **WHEN** the features of `deep_causality_linear` and `deep_causality_tensor` are enumerated
- **THEN** neither declares `tensor-iso`
- **AND** no `#[cfg(feature = "tensor-iso")]` remains in either crate

#### Scenario: The conversion is unconditionally available
- **WHEN** a caller converts between a tensor and a sparse matrix with default features
- **THEN** the conversion is available from `deep_causality_tensor`
- **AND** its behaviour, including its error type, is unchanged

#### Scenario: The example builds without naming a feature
- **WHEN** `examples/mathematics_examples` is built after its `features = ["tensor-iso"]` line is removed
- **THEN** the `tensor_sparse_memory_budget` example compiles and runs unchanged

### Requirement: Both build systems describe the same dependency graph
Every crate's `Cargo.toml` and its `BUILD.bazel` SHALL declare the same dependency on the linear crate.

`deep_causality_cfd/BUILD.bazel:30` declares a `deep_causality_sparse` dependency that
`deep_causality_cfd/Cargo.toml` does not. Migrating one build system without the other would carry
the discrepancy forward silently.

#### Scenario: The existing discrepancy is resolved
- **WHEN** the cfd crate's two manifests are compared after the migration
- **THEN** they agree on whether it depends on the linear crate

#### Scenario: Both gates pass
- **WHEN** `cargo test --workspace` and `bazel test //...` both run
- **THEN** both succeed

### Requirement: Documentation names the crate that exists
Every document that names `deep_causality_sparse` outside the archive SHALL be updated to name `deep_causality_linear`, and archived change proposals SHALL be left unchanged.

The literal appears in 203 files. 34 of those are under `openspec/changes/archive/`, which records
what was proposed at the time; rewriting them would falsify the record.

#### Scenario: Live documentation is current
- **WHEN** `AGENTS.md`, `README.md`, the project website and the docs site are searched
- **THEN** they describe the crate that exists

#### Scenario: The dependency graph in AGENTS.md is updated
- **WHEN** `AGENTS.md` §Project Dependencies is read
- **THEN** it shows the linear crate's tier and the tensor → linear edge

#### Scenario: Archived proposals are untouched
- **WHEN** `openspec/changes/archive/` is inspected after the migration
- **THEN** its references to the retired crate are unchanged
