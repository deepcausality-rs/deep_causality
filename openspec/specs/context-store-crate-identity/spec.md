# context-store-crate-identity Specification

## Purpose
Defines the `deep_causality_context_store` crate: the Tier-0, dependency-free persistence contract for the context, what it owns, what it never depends on, and the crate infrastructure it carries.
## Requirements
### Requirement: The persistence contract lives in `deep_causality_context_store`

The crate `deep_causality_context_store` SHALL own the persistence contract for a `Context`: the
record types, `IdReserve`, `RECORD_VERSION`, `ProjectionError`, the identifier aliases, and the
`ContextStorage`, `Recordable`, `Substrate`, `ContextEvents` and `ContextStorageStream` traits. It
SHALL sit at the repository root beside `deep_causality_context`, and it SHALL be a `std` crate.

The crate declares no `unsafe`, no macro in `src/`, no prelude, and exports every public type,
trait and error from `src/lib.rs`.

#### Scenario: A backend names the contract through one crate

- **WHEN** a crate depends on `deep_causality_context_store` and nothing else from the workspace
- **THEN** it can implement `ContextStorage` and construct every record type, because every name
  the trait mentions is exported from that crate's root

#### Scenario: The crate is `std`

- **WHEN** the crate's `lib.rs` is read
- **THEN** it carries no `#![no_std]` attribute and no `no-std` feature

### Requirement: The crate has no dependencies

`deep_causality_context_store` SHALL declare no dependency of any kind: no workspace crate and no
external crate in `[dependencies]`, and no build dependency. Dev-dependencies are permitted only
where a test needs one, and none is planned.

This is what makes the crate the cheapest dependency a backend can take, and what lets it sit at
Tier 0.

#### Scenario: The manifest declares nothing

- **WHEN** the `[dependencies]` table of `deep_causality_context_store/Cargo.toml` is read
- **THEN** it is absent or empty

#### Scenario: The tier block records the position

- **WHEN** the workspace dependency tiers are re-derived from the manifests
- **THEN** `deep_causality_context_store` appears in Tier 0 of the `AGENTS.md` tier block, the
  root crate list and crate count in `AGENTS.md` include it, and `deep_causality_context`'s entry
  lists it as a direct dependency

### Requirement: The vocabulary types are declared in the store crate and re-exported by the context crate

`RelationKind`, `TimeScale`, `VerticalDatum` and `SubstrateRef` SHALL be declared in
`deep_causality_context_store`, and `deep_causality_context` SHALL re-export all four from its
root so that no consumer import changes.

The three tracked modules and their tests move by `git mv`, so history follows. The untracked
`SubstrateRef` draft moves by plain `mv` and is registered for the first time. `SubstrateRef`
keeps its two-field shape, `source` and `key`, with constructor, getters and `Display` as
`"{source}/{key}"`.

#### Scenario: Existing imports compile unchanged

- **WHEN** every `use deep_causality_context::{…, TimeScale, …}`, `RelationKind` and
  `VerticalDatum` import in the workspace is built after the move
- **THEN** all of them compile, and `cargo doc` for the context crate shows each as a re-export

#### Scenario: The store crate names them without the context crate

- **WHEN** a `RelationRecord` is constructed with `RelationKind::Spatial` in a crate depending on
  the store crate alone
- **THEN** it compiles

#### Scenario: Moved tests keep their assertions

- **WHEN** the test suites for `RelationKind`, `TimeScale` and `VerticalDatum` run in the store
  crate
- **THEN** every test that moved passes with no assertion weakened or removed

### Requirement: The store crate declares the identifier width the records carry

`deep_causality_context_store` SHALL declare `pub type IdentificationValue = u64;` and the
aliases `ContextId` and `ContextoidId` as `IdentificationValue`, and every record field carrying an
identifier SHALL name one of the two aliases rather than a primitive integer.

#### Scenario: The width changes at one declaration

- **WHEN** the store crate's `IdentificationValue` is changed to a wider unsigned integer
- **THEN** every record field, every trait signature and `IdReserve` follow, and the context
  crate's build fails at the projection until its own alias agrees

#### Scenario: No record spells a raw integer for an identifier

- **WHEN** `deep_causality_context_store/src` is searched for `u64` in an identifier position
- **THEN** none is found outside the `IdentificationValue` declaration itself

### Requirement: The crate carries the repository's standard crate infrastructure

`deep_causality_context_store` SHALL ship what every workspace member carries: a `BUILD.bazel`
declaring the library, its docs and one `rust_test_suite` per test folder; an SBOM pair generated
by `scripts/sbom.sh`; a README; `[lints] workspace = true`; a root `Cargo.toml` member entry; and
a `[workspace.dependencies]` entry at two-digit version precision. Its `CHANGELOG.md` SHALL NOT be
hand-written.

The crate carries an explicit starting version of 0.1.0. Later versions are release-plz's.

#### Scenario: Bazel runs every test file Cargo runs

- **WHEN** the crate's tests run under `cargo test -p deep_causality_context_store` and under
  `bazel test //deep_causality_context_store/...`
- **THEN** both pass, and every `*_tests.rs` file under `tests/` is matched by a suite glob

#### Scenario: The crate is visible to the derived crate list

- **WHEN** `scripts/crates.sh` is sourced
- **THEN** `deep_causality_context_store` appears in `DC_CRATES` with its directory in
  `DC_CRATE_DIRS`, with no workflow file edited by hand

#### Scenario: The crate publishes before the context crate

- **WHEN** release-plz computes the publish order
- **THEN** `deep_causality_context_store` precedes `deep_causality_context`, because the latter
  depends on the former
