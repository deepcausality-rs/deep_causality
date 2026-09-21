# context-scalar-parameter Specification

## Purpose
TBD - created by archiving change consolidate-context-signature. Update Purpose after archive.
## Requirements
### Requirement: Every context node type is generic in its scalar

The context node types SHALL declare a scalar type parameter and SHALL NOT name a concrete floating
point type in a field, a signature or a bound.

The spatial types, the spacetime types and the real-valued temporal types each take one scalar
parameter bounded once on the real-field bound from `deep_causality_algebra`. Integer-valued
temporal types keep their integer representations, which are a choice of encoding rather than of
precision.

#### Scenario: No context type spells a concrete float

- **WHEN** `deep_causality_context/src` is searched for `f64` or `f32` in a struct field, a type
  alias or a trait bound
- **THEN** none is found outside the ready-made aliases that deliberately fix a working scalar

#### Scenario: A node type is constructed at more than one precision

- **WHEN** the same spatial node type is instantiated at two different real fields
- **THEN** both compile and each carries its own scalar, with no impl duplicated to serve them

### Requirement: A context runs at any shipped real field

A context SHALL be constructible and readable at each real field the workspace ships, with no line
added to `deep_causality_context` to admit one.

This is the acceptance criterion for treating precision as a parameter: the crate is generic in the
scalar, not enumerating the scalars it supports.

#### Scenario: A context at a reduced-precision scalar

- **WHEN** a context is built whose spatial and spacetime nodes carry the crate's smallest shipped
  real field, and its coordinates are read back
- **THEN** it compiles and round-trips, and the crate contains no code naming that type

#### Scenario: A context at an extended-precision scalar

- **WHEN** the same context is built at the crate's largest shipped real field
- **THEN** it compiles and round-trips, and the only edit between the two is the scalar named at the
  call site

### Requirement: The ready-made aliases keep the common path inference-free

The shipped `Base*` and `Uniform*` aliases SHALL fix a concrete working scalar, so that code using
them needs no type annotation it did not need before.

Making the types generic must not make the common case harder to write. The aliases are where the
scalar is chosen for callers who do not want to choose.

#### Scenario: Existing alias-based code needs no annotation

- **WHEN** code that names `BaseContext` or `BaseContextoid` is compiled
- **THEN** it requires no scalar annotation, because the alias supplies one

