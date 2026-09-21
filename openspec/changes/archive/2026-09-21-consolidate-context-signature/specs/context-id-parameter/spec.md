## ADDED Requirements

### Requirement: The context crate declares its own identifier aliases

`deep_causality_context` SHALL declare the aliases for its own identifiers and SHALL NOT take them
from another crate.

`ContextId` and `ContextoidId` move here from `deep_causality_core`, which declared them and never
used them. They are not duplicated: after the move core has no context identifier alias at all.

A context's identifier width is a property of contexts, so the crate that owns `Context` and
`Contextoid` owns the alias. An alias declared elsewhere is that crate's decision: were it widened
there, every context type would change width without this crate having said anything.

#### Scenario: The width changes at one declaration

- **WHEN** the crate's identifier alias is changed to a wider unsigned integer
- **THEN** every identifier field, constructor parameter and accessor in the crate follows, and no
  other declaration is edited

#### Scenario: The crate is independent of another crate's identifier decision

- **WHEN** a crate the context crate depends on changes its own identifier alias
- **THEN** the context types are unaffected, because they do not name it

#### Scenario: The identifiers have exactly one declaration site

- **WHEN** the workspace is searched for declarations of the context identifier aliases
- **THEN** each appears only in `deep_causality_context`, and neither `deep_causality_core` nor
  `deep_causality` declares or re-exports one

### Requirement: No context type spells a raw integer for an identifier

The context types SHALL name the crate's identifier aliases in every field, parameter and return
position that carries an identifier, rather than naming a primitive integer. This covers the node
types, the context graph and the graph traits.

This is the range axis of the same problem the scalar parameter solves for precision. A real scalar
selects the significand, where the failure mode is rounding; an identifier width selects the range,
where the failure mode is overflow — not an approximation of the right answer but a wrong one. A
crate that parameterises one and hardcodes the other has unwelded half of itself.

#### Scenario: Identifiers are searchable and uniform

- **WHEN** `deep_causality_context/src` is searched for a raw unsigned integer type in an
  identifier position
- **THEN** none is found

#### Scenario: A context addresses more contextoids than the current width allows

- **WHEN** the identifier alias is widened and a context is built with identifiers beyond the
  previous range
- **THEN** it compiles and the identifiers round-trip through the graph's lookup

### Requirement: The context graph stores the relation, not its discriminant

The context graph SHALL carry `RelationKind` as its edge weight, and the crate SHALL NOT cast a
relation to an integer to store it.

The public API already takes a `RelationKind`. Storing the cast discriminant discards the type at
the graph boundary, so the edge kind cannot be read back, and it widens a `#[repr(u8)]` enum of
four variants into eight bytes. The backing graph bounds its weight on clone and default only, and
the context graph calls no weighted algorithm — only node and edge structure — so the enum
qualifies once it has a default.

That default exists to satisfy the bound. Edge construction always supplies a real relation, so the
default is never produced by the API and is documented as the bound's filler rather than as a
meaningful relation.

#### Scenario: An edge's relation survives storage

- **WHEN** an edge is added with a given relation kind and the graph is queried for that edge
- **THEN** the relation is recoverable, rather than having been flattened to a number

#### Scenario: No relation is cast to an integer

- **WHEN** the crate is searched for a cast of a relation to an integer type
- **THEN** none is found
