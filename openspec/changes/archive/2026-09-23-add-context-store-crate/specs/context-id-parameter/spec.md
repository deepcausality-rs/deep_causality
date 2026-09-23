## MODIFIED Requirements

### Requirement: The context crate declares its own identifier aliases

`deep_causality_context` SHALL declare the aliases for its own identifiers and SHALL NOT take them
from another crate.

`ContextId` and `ContextoidId` move here from `deep_causality_core`, which declared them and never
used them. They are not duplicated in core: after the move core has no context identifier alias at
all.

`deep_causality_context_store` declares aliases of the same two names for the identifiers its
records carry, resolving to its own `IdentificationValue`. The two declarations are independent
and are pinned equal by the projection: `Context::snapshot` writes a context identifier into a record
field, so a widening on either side alone fails the context crate's build. A consumer that
glob-imports both crates and names `ContextId` resolves the ambiguity by importing one path.

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

#### Scenario: The identifiers have one declaration site per crate that owns them

- **WHEN** the workspace is searched for declarations of the context identifier aliases
- **THEN** each appears in `deep_causality_context` and in `deep_causality_context_store`, and
  neither `deep_causality_core` nor `deep_causality` declares or re-exports one

#### Scenario: The two widths are pinned equal

- **WHEN** a `Contextoid`'s identifier is written into a `ContextoidRecord` by `Context::snapshot`
- **THEN** it compiles without a cast, and a test passes a context crate `ContextoidId` to a
  function taking the store crate's `ContextoidId`
