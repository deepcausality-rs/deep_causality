## ADDED Requirements

### Requirement: `deep_causality` does not re-export the context surface

`deep_causality` SHALL NOT re-export any item that moved to `deep_causality_context`. Reaching a
moved item requires a declared dependency on `deep_causality_context` and an import from it.

Non-contextual reasoning is the default case and carries no context dependency. A re-export would
keep every existing `use deep_causality::Context` compiling, which would preserve the coupling this
extraction exists to expose and would leave the dependency graph reporting that `deep_causality`
owns context. The breakage is the mechanism: it makes each consumer decide once, in its
`Cargo.toml`, where the decision is durable and searchable.

#### Scenario: A moved item is not reachable through the old path

- **WHEN** a crate depends only on `deep_causality` and writes `use deep_causality::Context;`
- **THEN** compilation fails with an unresolved import, and the fix is to add
  `deep_causality_context` as a dependency and import from it

#### Scenario: The root module names no moved item

- **WHEN** `deep_causality/src/lib.rs` is read
- **THEN** it contains no `pub use` of `deep_causality_context`, and none of the moved type, trait,
  error or alias names appears in its export list

#### Scenario: A dependency search answers the question correctly

- **WHEN** the workspace manifests are searched for crates declaring `deep_causality_context`
- **THEN** the result is exactly the set of crates that use context, with no crate that merely
  depends on `deep_causality` appearing in it

### Requirement: Every consumer of a moved item is migrated

Each crate and example package that names a moved item SHALL declare `deep_causality_context` in its
manifest and import the moved items from it, so that the workspace builds and every test and example
passes.

The `use deep_causality::*;` glob imports SHALL be migrated too. A glob does not name the moved
symbols, so it fails at the use site rather than at the import, and a package left unmigrated
reports errors that point away from their cause.

#### Scenario: The workspace builds and tests green

- **WHEN** `bazel test //...` runs after the migration
- **THEN** every target builds and passes, including `deep_causality_ethos`, the example packages
  and the benches

#### Scenario: `deep_causality_ethos` names the dependency it uses

- **WHEN** `deep_causality_ethos/Cargo.toml` is read
- **THEN** it declares `deep_causality_context`, because its public API names `Context` in
  signatures its own consumers must satisfy

#### Scenario: Example packages declare what they import

- **WHEN** the manifests of `classical_causality_examples`, `csm_examples`, `tokio_example` and
  `avionics_examples` are read
- **THEN** each declares `deep_causality_context`, and no example imports a moved item from
  `deep_causality`

### Requirement: The breaking change is announced with a migration path

The release SHALL carry a migration note stating what moved, that the context dependency is now
declared rather than inherited, and the manifest and import edit a consumer makes.

The note SHALL state that `deep_causality_ethos` is breaking for its own consumers, since its public
API names `Context` and callers therefore need the new crate too.

#### Scenario: A downstream user can migrate from the note alone

- **WHEN** a user of `deep_causality` whose build has broken reads the migration note
- **THEN** it names the moved items, gives the `Cargo.toml` line to add, and shows the import
  rewrite, without requiring them to read the diff

#### Scenario: Version bumps reflect the breakage

- **WHEN** the released versions are inspected
- **THEN** `deep_causality` has taken a breaking bump to 0.18.0 and `deep_causality_ethos` to 0.4.0,
  while `deep_causality_core` has taken an additive bump only
