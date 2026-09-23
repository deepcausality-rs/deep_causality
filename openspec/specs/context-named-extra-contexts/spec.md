# context-named-extra-contexts Specification

## Purpose
Defines extra contexts as named containers keyed by the store's identifier: `extra_ctx_add_new(name, capacity, default)`, `extra_ctx_add_new_with_id`, `extra_ctx_get_name`, the allocator rule and the refusal of identifier 0.
## Requirements
### Requirement: An extra context has a name

`ExtendableContextuableGraph` SHALL take a name for every extra context: `extra_ctx_add_new(name:
&str, capacity: usize, default: bool) -> ContextId`, `extra_ctx_add_new_with_id(id: ContextId,
name: &str, capacity: usize, default: bool) -> Result<(), ContextIndexError>`, and
`extra_ctx_get_name(&self, id: ContextId) -> Option<&str>`.

A stored extra context is a container referenced by identifier and name, and an extra with no name
had nothing a reference could carry. This is a breaking change to a public trait; its callers are
four test files in the context crate.

#### Scenario: A name is kept with the extra

- **WHEN** an extra is added under the name `"weather"` and its identifier is queried with
  `extra_ctx_get_name`
- **THEN** the answer is `Some("weather")`, and an identifier no extra holds answers `None`

#### Scenario: Identifier 0 is refused

- **WHEN** `extra_ctx_add_new_with_id(0, ..)` is called
- **THEN** it returns `Err(ContextIndexError(..))`, because 0 means no extra context is current

### Requirement: The extra-context allocator never collides with a held identifier

`extra_ctx_add_new` SHALL derive the next identifier as one more than the highest extra-context
identifier the context has ever held, and 1 when it has held none, so that an identifier added
through `extra_ctx_add_new_with_id`, through `Context::restore` or through `Context::apply`, and
an identifier dropped since, is never allocated again. In the sequential case the numbering is
unchanged: 1, 2, 3.

#### Scenario: Sequential numbering is unchanged

- **WHEN** `extra_ctx_add_new` is called three times on a fresh context
- **THEN** it returns 1, 2 and 3

#### Scenario: An explicit identifier is never reached

- **WHEN** `extra_ctx_add_new_with_id(7, ..)` is followed by `extra_ctx_add_new`
- **THEN** the latter returns 8 rather than panicking on a collision

#### Scenario: A dropped identifier is not reallocated

- **WHEN** a context holding extras 40 and 41 drops 41 through `Context::apply` of
  `ContextRetracted(41)` and `extra_ctx_add_new` is then called
- **THEN** it returns 42

#### Scenario: A restored context allocates past its extras

- **WHEN** a context is restored from a snapshot whose extras carry identifiers 40 and 41 and
  `extra_ctx_add_new` is then called
- **THEN** it returns 42

### Requirement: A hydrated extra carries the store's identifier and name

`Context::restore` SHALL key each extra context by the identifier the snapshot's
`ExtraContextSnapshot` carries and name it as the snapshot names it, so that an extra materialised
from a stored container is addressed inside the engine by the container's own identifier, which is
unique across the store.

The identifiers an engine assigned to extras before a branch was stored do not survive the round
trip; the stored containers' identifiers replace them.

#### Scenario: Two hydrations agree

- **WHEN** one container referencing another is hydrated twice, into two `Context` values
- **THEN** both hold the extra under the same identifier and name, and a `NodeLinked` event naming
  that identifier applies to both without translation

### Requirement: A store container event never reaches a local extra

`Context` SHALL mark each extra as stored or local: an extra from `Context::restore` or from
`Context::apply` of `ContextAttached` is stored, and one from `extra_ctx_add_new` or
`extra_ctx_add_new_with_id` is local. The store assigns container identifiers, so a local extra is
not a container of the store even when its identifier equals one. `Context::apply` treats a local
extra as not held by any event that names a container: a membership event naming it is
`ProjectionError::Identity`, a `ContextDetached` or `ContextRetracted` naming it leaves it in
place, and a `ContextAttached` under its identifier is `ProjectionError::Identity`, so no graph is
merged into, kept under, or dropped for another container's identifier. Node identifiers are the
store's, so `NodeRetracted`, `EdgeCreated` and `EdgeRetracted` apply by node identity to every
graph holding the node, local extras included. A context that never applies a store event is
unaffected.

#### Scenario: An attachment under a local identifier is refused

- **WHEN** a subscribed context allocates a local extra, and the store then creates a container
  under the same identifier and attaches it to the subscribed container
- **THEN** applying the `ContextAttached` event is `Err(ProjectionError::Identity(..))`, the local
  extra keeps its name and nodes, and a fresh subscription holds the store's container under that
  identifier

#### Scenario: A local extra survives a store detach

- **WHEN** `ContextDetached` and `ContextRetracted` naming a local extra's identifier are applied
- **THEN** both return `Ok(())` and the local extra, and the current extra, are unchanged

#### Scenario: A node or edge event reaches a local extra holding the node

- **WHEN** a local extra holds nodes 3 and 4, and `EdgeCreated { 4, 3 }`, `EdgeRetracted { 4, 3 }`
  and `NodeRetracted(3)` are applied in turn
- **THEN** the local extra gains the edge, loses it, and then loses node 3, as every other graph
  holding those nodes does
