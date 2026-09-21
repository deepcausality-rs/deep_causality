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

Core declares an alias only for vocabulary core itself uses. `ContextId` and `ContextoidId` name
concepts core does not own and never referenced, so they move to `deep_causality_context`, which
owns `Context` and `Contextoid`. That leaves six aliases here — `IdentificationValue`,
`CausaloidId`, `DescriptionValue`, `NumericalValue`, `NumberType` and `FloatType` — all of which
keep their names at `deep_causality`'s root by re-export, so no call site changes.

`TeloidTag` and `TeloidID` remain declared in core with no user, and are the same defect: naming a
concept `deep_causality_ethos` owns. They are not moved here only because nothing in this change
touches that crate's vocabulary.

#### Scenario: The duplicate declarations are gone

- **WHEN** the workspace is searched for declarations of the six live aliases
- **THEN** each is declared in `deep_causality_core` and nowhere else, and
  `deep_causality/src/alias/alias_primitives.rs` no longer exists

#### Scenario: No call site changed for the six that stayed

- **WHEN** the deduplication is applied
- **THEN** every existing use of `FloatType`, `NumericalValue`, `IdentificationValue`,
  `DescriptionValue`, `NumberType` and `CausaloidId` still resolves, including through
  `use deep_causality::*`, with no import rewritten

#### Scenario: The dead Teloid aliases leave the surface

- **WHEN** a crate writes `use deep_causality::TeloidID;`
- **THEN** compilation fails, and `deep_causality_ethos::TeloidID` is the name that resolves

#### Scenario: The context identifiers are no longer core's

- **WHEN** a crate writes `use deep_causality_core::ContextId;` or
  `use deep_causality::ContextoidId;`
- **THEN** compilation fails, and `deep_causality_context` is where both resolve

