## ADDED Requirements

### Requirement: A spacetime node reports its own metric signature

Every spacetime type SHALL report the metric signature it is in, and no type above it SHALL declare
one on its behalf.

Every spacetime node is in some signature: the Newtonian coordinate spacetime is in (+,+,+,+) and
the relativistic ones are in (−,+,+,+). The question is never whether a signature exists, only who
says what it is. A context whose spacetime varies holds nodes in more than one, so a single value
on the context cannot answer for all of them, and one that some contents contradict is a claim
nothing enforces.

Asking the node matches what the rest of the stack already does. `deep_causality_physics` reads a
metric off a `CausalMultiVector` and compares it against what the caller expects, at ten sites
across its electromagnetism, relativity and dynamics kernels. This is that shape one layer up.

A signature does not vary with position, by continuity, so a node derives it from what it is rather
than from what it currently holds: a numerically evolved spacetime keeps its signature while every
component of its tensor changes.

No type in this crate SHALL carry a metric tensor except the one coordinate type that is
numerically evolved. A tensor is a field varying with position, which `deep_causality_physics`
already computes for the analytic forms.

#### Scenario: Two signatures in one context

- **WHEN** a context over the variant spacetime enum holds a Newtonian spacetime node and a
  relativistic one
- **THEN** each reports its own signature, and the two differ

#### Scenario: The signature outlives the tensor

- **WHEN** every component of a numerically evolved spacetime's metric tensor is replaced
- **THEN** the signature it reports is unchanged

#### Scenario: A node signature feeds the existing convention checks

- **WHEN** a node's signature is passed to the metric crate's convention checks
- **THEN** they answer without anything new in between, because a node supplies the same `Metric`
  those checks already take

### Requirement: A context can state that it has no spatial extent

A zero-sized type SHALL implement the spatial and spacetime traits trivially, so that a context
holding no spatial node names it explicitly rather than naming a type its graph never holds.

A `Context` names a spatial and a spacetime type whether or not its graph holds either, so a
context with a clock and no position still has two slots to fill. Measured usage makes this the
common case rather than an edge: across the workspace the contextoid variants are used Root 45,
Datoid 47, Tempoid 15, Spaceoid 5, SpaceTempoid 5, so most contexts hold a root, some data and a
clock, and nothing spatial at all.

#### Scenario: A context with time but no space

- **WHEN** a context is declared for a graph that holds only root, data and temporal nodes
- **THEN** it names the empty spatial and spacetime types, it compiles, and it accepts temporal and
  data contextoids

#### Scenario: The absence is stated, and costs nothing

- **WHEN** such a context is read
- **THEN** the absence of a spatial part is visible in its own signature, and the type naming it is
  zero-sized

### Requirement: The analytic metric families are named in one place

`deep_causality_metric` SHALL name the analytic forms a metric tensor can take, with the parameters
that fix each one.

`deep_causality_physics` computes four closed forms and takes their parameters as loose arguments,
so the choice of form exists in that crate only as a function name. Naming the forms and their
parameters as one type gives that choice a value, which is what a caller needs to carry a decision
it has made but not yet evaluated.

Each variant SHALL carry the parameters that fix the form across the manifold and none of the
coordinates it is evaluated at. The type SHALL NOT bound its scalar, so that the crate stays a
dependency-free leaf.

#### Scenario: A family carries its parameters and not its evaluation point

- **WHEN** a family fixed by a central mass is named
- **THEN** it carries that mass, and does not carry the radius at which a tensor would be computed

#### Scenario: The flat case is one variant

- **WHEN** a flat metric is named
- **THEN** one variant covers it, and the signature distinguishes the Minkowski case from the
  Euclidean one
