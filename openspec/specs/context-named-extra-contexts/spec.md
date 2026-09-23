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
identifier the context holds, and 1 when it holds none, so that an identifier added through
`extra_ctx_add_new_with_id` or through `Context::restore` is never allocated again. In the
sequential case the numbering is unchanged: 1, 2, 3.

#### Scenario: Sequential numbering is unchanged

- **WHEN** `extra_ctx_add_new` is called three times on a fresh context
- **THEN** it returns 1, 2 and 3

#### Scenario: An explicit identifier is never reached

- **WHEN** `extra_ctx_add_new_with_id(7, ..)` is followed by `extra_ctx_add_new`
- **THEN** the latter returns 8 rather than panicking on a collision

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
