## MODIFIED Requirements

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
