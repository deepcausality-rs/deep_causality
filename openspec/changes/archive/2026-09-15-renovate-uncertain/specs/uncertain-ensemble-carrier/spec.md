<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: One materialisation signature serves every witnessed carrier

`Uncertain<R>` SHALL expose a single `materialize` whose carrier is a type parameter, and SHALL NOT declare a container type of its own to hold an ensemble.

```rust
pub fn materialize<W>(&self, session: &mut SampleSession, n: usize)
    -> Result<W::Type<R>, UncertainError>
where
    W: Collectable<W> + HKT;
```

The caller chooses `DenseVectorWitness` or `CausalTensorWitness` at the call site and receives a
container that already carries every categorical structure that witness provides. No new container
type is introduced, which is what `sampling-hkt-composition` requires.

#### Scenario: The same call produces either carrier

- **WHEN** `materialize` is instantiated at `DenseVectorWitness` and again at `CausalTensorWitness` over the same value, session seed and count
- **THEN** both compile from one signature, and the two results hold the same `n` draws in the same order

#### Scenario: The crate declares no ensemble container

- **WHEN** `deep_causality_uncertain/src` is read
- **THEN** it declares no container type for holding draws

#### Scenario: The ensemble carries its witness's structure

- **WHEN** an ensemble is materialised into a witnessed carrier
- **THEN** the existing `Functor`, `Foldable` and `Semigroupal` implementations of that witness apply to it with no further work in this crate

### Requirement: Building a rank-1 container from a sequence is a haft capability

`deep_causality_haft` SHALL provide a capability that builds `F::Type<T>` from a sequence of `T`, and `deep_causality_haft`, `deep_causality_linear` and `deep_causality_tensor` SHALL implement it for their rank-1 witnesses.

No existing haft trait can do this. `Foldable` consumes a structure, `Pure` builds a one-element
one, and `Semigroupal::zip_with` pairs two without extending either. All of haft's capability traits
were checked before adding one.

```rust
pub trait Collectable<F: HKT> {
    fn collect<T, I: IntoIterator<Item = T>>(items: I) -> F::Type<T>;
}
```

Placing it in `haft` beside `Foldable` — rather than in the crate that noticed it was missing —
keeps `deep_causality_uncertain` dependent on `haft` alone, so the crate gains no dependency on
`linear` or `tensor` and its dependency tier does not change. This follows the placement already
settled for `DiagonalTraversable`.

The requirement is about the dependency graph a **consumer links**, so it binds the normal
dependencies and not the dev ones. The two container crates are dev-dependencies of
`deep_causality_uncertain`, and have to be: the claim under test is that one signature serves more
than one witness, and a test instantiating a single witness cannot see it. A dev-dependency reaches
the test harness and not the library, so nothing downstream links either crate.

#### Scenario: Every rank-1 witness implements it

- **WHEN** `Collectable` is instantiated at `VecWitness`, at `DenseVectorWitness` and at `CausalTensorWitness`
- **THEN** each returns a container of the given values in the given order, and the tensor result has rank 1 with extent equal to the number of values

#### Scenario: The empty sequence is accepted

- **WHEN** `collect` is given no values
- **THEN** it returns the carrier's empty container rather than failing

#### Scenario: The uncertain crate does not depend on the container crates

- **WHEN** `deep_causality_uncertain`'s manifest is read after this change
- **THEN** its `[dependencies]` name `deep_causality_haft` and name neither `deep_causality_linear` nor `deep_causality_tensor`, and no file under `src/` names either
- **AND** the two container crates appear only under `[dev-dependencies]`, where the two-witness test reaches them and a consumer does not

### Requirement: Ensembles compose by index, and the composition surface says so

The crate's composition documentation SHALL direct a caller to the zip witnesses and the diagonal traversal for combining ensembles, and SHALL record that the cartesian traversal forms the product.

Two ensembles drawn at the same indices from one session are correlated by index: draw *i* of one
belongs with draw *i* of the other. That is `Semigroupal::zip_with` on `ZipDenseVectorWitness` or
`ZipTensorWitness`, and `DiagonalTraversable::sequence_zip` for turning a structure of ensembles
inside out.

Neither zip witness has `Pure`, because the unit of a positional zip is the infinite repeat, so
`sequence_zip` takes its accumulator as a parameter. For sampling that means the ensemble size is
declared rather than inferred, which is correct.

#### Scenario: Correlated quantities pair by index

- **WHEN** two quantities are materialised from one session at the same indices and combined for downstream use
- **THEN** the i-th draw of one is paired with the i-th draw of the other, and the result holds `n` values

#### Scenario: The documentation names the cartesian hazard

- **WHEN** the crate's composition documentation is read
- **THEN** it states that `Traversable::sequence` forms the cartesian product across quantities and is not the traversal to use for sampling

#### Scenario: Materialising a field is documented as infeasible

- **WHEN** the same documentation is read for guidance on uncertainty over a simulation field
- **THEN** it states that materialising an ensemble per cell does not fit in memory at field size, and directs the reader to materialise the uncertain inputs, traverse them diagonally, and evaluate per draw
