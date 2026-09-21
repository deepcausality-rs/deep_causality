# context-associated-value-types Specification

## Purpose
TBD - created by archiving change consolidate-context-signature. Update Purpose after archive.
## Requirements
### Requirement: The coordinate and time traits carry associated types

The coordinate and time traits SHALL each declare their value type as an associated type rather
than as a trait parameter. This covers `Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` and
`Distance`.

`Coordinate` declares `type Coord`; `Temporal` declares `type TimeUnit`; `Spatial` reads
`Identifiable + Coordinate`; `SpaceTemporal` reads `Identifiable + Spatial + Temporal` and returns
`&Self::TimeUnit` from `t`.

This is sound because no type implements any of these traits at two different value types. Across
the crate, `Coordinate` has 13 implementors, `Spatial` 12, `Distance` 6 and `SpaceTemporal` 5, each
at a single value type; `Temporal` has 11 implementors across three value types, varying between
implementors and never within one.

#### Scenario: One implementor, one value type

- **WHEN** a context node type implements `Coordinate`, `Spatial`, `Temporal`, `SpaceTemporal` or
  `Distance`
- **THEN** it fixes the associated type once, and a second impl of the same trait at a different
  value type is rejected by the compiler as a conflicting implementation

#### Scenario: Time units still vary across implementors

- **WHEN** the temporal node types are inspected
- **THEN** their `TimeUnit` types differ from one another, covering a real field, an unsigned
  integer and a signed integer, and each type names exactly one

### Requirement: The context types drop their value parameters

The context types SHALL NOT declare `VS` or `VT` parameters. This covers `Context`, `Contextoid`,
`ContextoidType`, `Contextuable`, `ContextuableGraph`, `ExtendableContextuableGraph` and
`SpaceTemporal`.

`Context`, `Contextoid` and `ContextoidType` therefore go from six type parameters to four before
the frame consolidates them further.

#### Scenario: Four parameters after the value types come off

- **WHEN** `Context`, `Contextoid` or `ContextoidType` is named before the frame lands
- **THEN** it takes `D, S, T, ST`, and supplying a fifth or sixth argument fails to compile

#### Scenario: No value parameter survives in the crate

- **WHEN** the crate is searched for `VS` or `VT` as a generic parameter
- **THEN** neither appears

### Requirement: The distance trait is named for what it computes

The distance trait SHALL be named `Distance`, and the name `Metric` SHALL refer to the manifold
signature from `deep_causality_metric` rather than to anything the context crate declares.

The unused marker combining a coordinate with a distance SHALL be removed. It had no implementors
and was named as a bound nowhere; with associated types the combination is written inline.

#### Scenario: The distance trait resolves under its new name

- **WHEN** a consumer calls `distance` on a context node type
- **THEN** the method comes from `Distance`, and `deep_causality_context::Metric` does not exist

#### Scenario: The removed marker is gone

- **WHEN** the workspace is searched for the coordinate-plus-distance marker trait
- **THEN** it is neither declared nor referenced, and nothing that previously compiled against it
  was relying on it

