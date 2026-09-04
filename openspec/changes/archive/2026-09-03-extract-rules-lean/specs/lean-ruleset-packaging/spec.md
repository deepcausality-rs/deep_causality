## ADDED Requirements

### Requirement: The ruleset is MIT licensed and attributes its origin
The extracted repository SHALL carry an MIT LICENSE at its root, SPDX headers on its sources, and an attribution notice recording that it derives from `tomato-bazel/rules_lean`.

Unlicensed code cannot be vendored into an MIT project, which is why the surveyed alternative was
unusable here. The ruleset is a derivative work and says so.

#### Scenario: License is present and machine-readable
- **WHEN** the repository is inspected by a consumer or by the registry
- **THEN** an MIT LICENSE file is present at the root
- **AND** every source file carries an SPDX identifier

#### Scenario: Derivation is recorded
- **WHEN** a reader asks where the rules came from
- **THEN** the repository states its upstream origin and what was changed

### Requirement: The public rule surface is documented and exercised
The extracted repository SHALL document every public rule, repository rule and module-extension tag it exposes, and SHALL ship a runnable example for each.

A ruleset nobody but its author can configure is not publishable. Examples are also the material the
registry's presubmit runs against.

#### Scenario: Documented surface matches the implementation
- **WHEN** a public symbol is added, renamed or removed
- **THEN** its documentation and at least one example change with it

#### Scenario: Examples build
- **WHEN** the examples are built
- **THEN** each succeeds against the ruleset as published

### Requirement: This repository builds against the published module
The deep_causality repository SHALL depend on the published module rather than a vendored copy, and its Lean gate SHALL pass unchanged after the migration.

The migration is the acceptance test for the extraction: the ruleset is only proven by the consumer
that drove its requirements.

#### Scenario: Vendored copy is retired
- **WHEN** the published module builds this repository green
- **THEN** the dependency is declared as a registry module
- **AND** the vendored copy is moved aside rather than deleted, per repository convention

#### Scenario: The Lean gate is unaffected
- **WHEN** the Lean proofs are built after the migration
- **THEN** every namespace type-checks as before
- **AND** the build remains fully cacheable, locally and remotely
