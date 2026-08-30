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

### Requirement: The duplicated linear algebra in `deep_causality_multivector` is marked for replacement
Every operation in `deep_causality_multivector` that duplicates one this crate provides SHALL be recorded with its successor, and each SHALL either be routed through `deep_causality_linear` or carry a written reason for staying.

The impact research surveyed six crates and reached multivector only through the shape census, which
counts what a crate *constructs* rather than what it *defines*. That missed a norm, a matmul and a
magnitude. Recording them here keeps the successor attached to each, so that "norms are defined in
exactly one place" is a checkable claim rather than an aspiration that stopped at the crates anyone
looked in.

| duplicate | successor | disposition |
|---|---|---|
| `MultiVectorL2Norm::norm_l2` / `normalize_l2` (`traits/l2_norm.rs:21,30`, impl `multivector/api/mod.rs:117`) | the vector 2-norm from `linear-vector` | route through it |
| `CausalMultiField::squared_magnitude` (`multifield/algebra/mod.rs`) — a hand-written `Σ x·x` over the tensor's slice | the vector squared 2-norm | route through it |
| `BatchedMatMul` (`multifield/ops/batched_matmul.rs`, 62 lines) | none — it batches rank-3 slices and belongs to the tensor surface | decide explicitly, and record the decision |

`ScalarEval` (`traits/scalar_eval.rs`) is **not** a duplicate and is not marked. It has the same
three members as `deep_causality_algebra::Normed`, but `extensions/scalar_eval/mod.rs` is a single
blanket delegating to it, existing only to add the `Sum` bound. It is a facade over the tower, which
is the arrangement this requirement wants everywhere else.

`squared_magnitude` is the one whose duplication is not merely tidiness. It sums `*val * *val` over
the raw slice, which is the squared modulus only for a real scalar; the tower's `modulus_squared`
is the operation that stays correct when the scalar is complex. The bound on the impl is
`T: Field + RealField`, so it is not wrong today — it is a second definition that is correct only
because of a bound that a later widening would remove silently.

#### Scenario: Each duplicate names its successor
- **WHEN** the table above is checked against `deep_causality_multivector`
- **THEN** each entry is either routed through `deep_causality_linear` or carries a written reason for staying

#### Scenario: The norm is defined once
- **WHEN** the workspace is searched for a definition of the Euclidean norm after migration
- **THEN** the definitions that remain are `deep_causality_linear`'s and any that documents why it differs

#### Scenario: The multivector results are unchanged
- **WHEN** the `deep_causality_multivector` suite runs after the routing
- **THEN** every value it reported before is reported again

#### Scenario: The batched matmul decision is recorded
- **WHEN** `BatchedMatMul` is inspected after this change
- **THEN** it either calls into `deep_causality_linear` or documents why a batched rank-3 operation stays on the tensor surface

#### Scenario: The census is corrected
- **WHEN** the construction census in `openspec/notes/archive/linear/deep-causality-linear.md` is re-read
- **THEN** the `deep_causality_multivector` row reflects the 13 `CausalTensor` constructions in its `src`, not zero
