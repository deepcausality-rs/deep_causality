# core-shared-vocabulary Specification

## Purpose
Defines `deep_causality_core` as the single home for the vocabulary the context and causal layers
share: the `Identifiable` trait, beside the `IdentificationValue` it returns, and the primitive type
aliases, each declared exactly once.

## Requirements

### Requirement: `Identifiable` lives in `deep_causality_core`

`deep_causality_core` SHALL define and export the `Identifiable` trait, whose single method `id`
returns the `IdentificationValue` that core already owns.

The trait is implemented on both sides of the context/causal split — `Contextoid` in
`deep_causality_context`, and `Causaloid`, `Model`, `Inference`, `Assumption`, `Observation` and
`ProposedAction` in `deep_causality` — so core is the only crate both can reach. Defining it in
`deep_causality_context` would make the context crate the author of the causal types' identity and
would force `deep_causality` to depend on it for a trait unrelated to context.

#### Scenario: Both layers implement the same trait

- **WHEN** a `Contextoid` from `deep_causality_context` and a `Causaloid` from `deep_causality` are
  each asked for their `id`
- **THEN** both resolve to `deep_causality_core::Identifiable`, and generic code bounded on that one
  trait accepts either

#### Scenario: The trait is stated in terms of the alias it returns

- **WHEN** the trait definition is read
- **THEN** `id` is declared as returning `IdentificationValue` rather than a bare `u64`

#### Scenario: No implementation needed editing

- **WHEN** the existing `Identifiable` impls are compared before and after the move
- **THEN** each differs only in the path it imports the trait from, because
  `IdentificationValue = u64` makes the restated return type the same type

### Requirement: Each shared primitive alias is declared exactly once

`deep_causality_core` SHALL be the sole declaration site for the primitive type aliases shared
across the workspace, and `deep_causality` SHALL NOT redeclare any of them.

`deep_causality/src/alias/alias_primitives.rs` declares ten aliases byte-identically to
`deep_causality_core/src/alias/mod.rs`. The eight that have users — `IdentificationValue`,
`ContextId`, `ContextoidId`, `CausaloidId`, `DescriptionValue`, `NumericalValue`, `NumberType` and
`FloatType` — SHALL keep their names at `deep_causality`'s root by re-export from core, so no call
site changes.

`TeloidTag` and `TeloidID` SHALL be removed from `deep_causality` rather than re-exported. Neither
has a user in `deep_causality` or in `deep_causality_core`, because `deep_causality_ethos` declares
and uses its own pair.

#### Scenario: The duplicate declarations are gone

- **WHEN** the workspace is searched for declarations of the eight live aliases
- **THEN** each is declared in `deep_causality_core` and nowhere else, and
  `deep_causality/src/alias/alias_primitives.rs` no longer exists

#### Scenario: No call site was touched

- **WHEN** the deduplication is applied
- **THEN** every existing use of `FloatType`, `NumericalValue`, `IdentificationValue`,
  `DescriptionValue`, `NumberType`, `CausaloidId`, `ContextId` and `ContextoidId` still resolves,
  including through `use deep_causality::*`, with no import rewritten

#### Scenario: The dead Teloid aliases leave the surface

- **WHEN** a crate writes `use deep_causality::TeloidID;`
- **THEN** compilation fails, and `deep_causality_ethos::TeloidID` is the name that resolves

#### Scenario: The context crate takes its aliases from core

- **WHEN** the imports of `deep_causality_context` are read
- **THEN** `FloatType`, `ContextId`, `ContextoidId` and `NumericalValue` come from
  `deep_causality_core`, and the crate declares no alias of its own for them
