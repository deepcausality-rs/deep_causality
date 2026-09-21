## ADDED Requirements

### Requirement: A frame names the space, time and spacetime a context uses

`ContextFrame` SHALL bundle the spatial, temporal and spacetime types of a context as associated
types, so that `Contextoid` takes the data type and the frame rather than naming all three.

A frame of reference is what licenses a split into space and time, so the three members belong
together: naming them separately lets a context pair a space with a spacetime that cannot contain
it.

#### Scenario: A contextoid takes two parameters

- **WHEN** a `Contextoid` is named after the frame lands
- **THEN** it takes the data type and the frame, and the space, time and spacetime come from the
  frame's associated types

#### Scenario: The frame's members are consistent by construction

- **WHEN** a frame is declared
- **THEN** its spatial, temporal and spacetime members are fixed together in one place, and a
  context built on that frame cannot mix a member from another

### Requirement: A frame over the variant kinds is the primary case

`ContextFrame` SHALL accept the variant `Kind` enums as its spatial, temporal and spacetime members,
and the crate SHALL treat that as the ordinary case rather than as a fallback. A frame naming
concrete types is the specialisation, for a context whose world is fixed and known at compile time.

Two things put it that way round. A causal model can change regime within one run — reasoning one
way far from a mass and another way close to it — and that is one context whose spacetime varies,
not two contexts. And a context assembled from stored records carries its frame per record, so a
read spanning several can return more than one; a fixed frame cannot hold the result at all.

The `Kind` enums satisfy the frame's member bounds already, and the existing uniform aliases are
built on them, so the caller chooses fixed or variant by choosing what the frame names, exactly as
the base and uniform aliases differ today.

#### Scenario: A frame over the variant enums

- **WHEN** a frame names the variant spatial, temporal and spacetime enums as its members
- **THEN** it satisfies the frame's bounds, and a context on that frame accepts contextoids holding
  any of the concrete variants

#### Scenario: A regime change inside one context

- **WHEN** a context on a variant frame holds one spacetime contextoid in one regime and a second
  in another
- **THEN** both are accepted by the same context, and the frame is named once

#### Scenario: A fixed frame is the narrower choice, not the default

- **WHEN** the crate's own frames and aliases are read
- **THEN** a variant frame is available for every position a fixed frame occupies, and the
  documentation says which to reach for when the world is not known at compile time

### Requirement: A frame can state that its context has no spatial extent

A zero-sized type SHALL implement the spatial and spacetime traits trivially, so that a frame whose
context holds no spatial node names it explicitly rather than inventing an unused type.

Associated-type defaults are not available, so the member cannot be omitted. Measured usage makes
this the common case rather than an edge: across the workspace the contextoid variants are used
Root 45, Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5, so most contexts hold a root, some data
and a clock, and nothing spatial at all.

#### Scenario: A context with time but no space

- **WHEN** a frame is declared for a context that holds only root, data and temporal nodes
- **THEN** it names the empty spatial and spacetime types, the frame compiles, and the resulting
  context accepts temporal and data contextoids

#### Scenario: The absence is stated, not implied

- **WHEN** such a frame is read
- **THEN** the absence of a spatial part is visible in the frame's own declaration

### Requirement: A spacetime node reports its own metric signature

Every spacetime type SHALL report the metric signature it is in, and the frame SHALL NOT declare
one.

Every spacetime node is in some signature: the Newtonian coordinate spacetime is in (+,+,+,+) and
the relativistic ones are in (−,+,+,+). The question is never whether a signature exists, only who
says what it is. A context whose spacetime varies holds nodes in more than one, so no constant on
the context can answer for all of them, and a constant that some contents contradict is a claim
nothing enforces.

Asking the node matches what the rest of the stack already does. `deep_causality_physics` reads a
metric off a `CausalMultiVector` and compares it against what the caller expects, at ten sites
across its electromagnetism, relativity and dynamics kernels. This is that shape one layer up.

A signature does not vary with position, by continuity, so a node derives it from what it is rather
than from what it currently holds: a numerically evolved spacetime keeps its signature while every
component of its tensor changes.

The frame SHALL NOT carry a metric tensor. A tensor is a field varying with position, which
`deep_causality_physics` already computes for the analytic forms.

#### Scenario: Two signatures in one context

- **WHEN** a context on a variant frame holds a Newtonian spacetime node and a relativistic one
- **THEN** each reports its own signature, and the two differ

#### Scenario: The signature outlives the tensor

- **WHEN** every component of a numerically evolved spacetime's metric tensor is replaced
- **THEN** the signature it reports is unchanged

#### Scenario: A node signature feeds the existing convention checks

- **WHEN** a node's signature is passed to the metric crate's convention checks
- **THEN** they answer without anything new in between, because a node supplies the same `Metric`
  those checks already take

### Requirement: A frame exposes the analytic metric family where one exists

`ContextFrame` SHALL expose the analytic metric family where one exists, as a function returning
nothing when the metric is numerically evolved and has no closed form.

A family stays on the frame where a signature does not, because a family is not derivable from a
node type. A node can report that it is Lorentzian; it cannot report that it is Schwarzschild at a
given central mass. That is a choice about the model, which is what a frame is for. It is a
function rather than a constant because those parameters differ between one model and the next.

#### Scenario: A numerically evolved metric has no family

- **WHEN** the metric family is requested for a frame whose metric has no closed form
- **THEN** the answer is that none exists, and the per-point tensor is carried by the coordinate
  type that needs it
