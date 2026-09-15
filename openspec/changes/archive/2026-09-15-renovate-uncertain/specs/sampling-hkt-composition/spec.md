<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: The sampling carrier is an existing witnessed container

Sampling SHALL produce values into a container that already carries an HKT witness, and this change SHALL NOT introduce a new container type to hold an ensemble.

A carrier holding `n` realised draws is a `Vec<T>` with a witness, and the stack already has
several. `CausalTensor` carries `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad` and
`Traversable` today, and a rank-1 tensor is exactly an ensemble. `DenseVector` carries the same set
plus `CoMonad`. A parallel container would duplicate that surface and would name in a sampling
crate a concept `tensor` and `linear` already own.

Drawing into such a container is an ordinary generic function, fully monomorphised, needing no
witness of its own. Where a function must *build* the container rather than receive one, it is
generic over the witness and uses haft's construction capability, so one signature serves every
carrier and the crate that draws depends on no container crate.

#### Scenario: An ensemble is an ordinary tensor
- **WHEN** `n` draws of a distribution are sampled into a rank-1 `CausalTensor`
- **THEN** the result carries every categorical structure that witness already provides
- **AND** neither `deep_causality_rand` nor `deep_causality_stats` nor `deep_causality_uncertain` declares a container type for this purpose

#### Scenario: The draw function needs no witness
- **WHEN** a function samples a distribution into a container the caller supplies
- **THEN** it is bounded on the scalar and the distribution only, with no HKT bound

#### Scenario: A building draw is generic over the carrier
- **WHEN** a function materialises `n` draws and must construct the container itself
- **THEN** its carrier is a type parameter bounded on haft's construction capability, and the same function returns a `DenseVector` or a rank-1 `CausalTensor` according to the witness the caller names

### Requirement: A lazy sampler carrier is not given a witness

A carrier that stores a sampling closure SHALL NOT be given an HKT container witness, SHALL be given haft's `Arrow` instead once its evaluation is pure, and the reason SHALL be recorded rather than left to be rediscovered.

The container traits in `deep_causality_haft` carry no `'static` bound — measured: zero occurrences
across `Functor`, `Pure`, `Applicative`, `Monad`, `Traversable` and `LaxMonoidal`. That absence is
deliberate. A functor receives a function, applies it and drops it, so the function never outlives
the call. `Profunctor`, which does store its functions, carries `'static` on every parameter.

A carrier holding a stored closure therefore needs bounds the container traits do not provide, and
an implementation cannot add them: `error[E0276]: impl has stricter requirements than trait`. This
is not a gap in `haft`. The container traits are for data; a lazy sampler is a program, and haft's
home for programs is the `Arrow` layer.

The `Arrow` layer is now occupied rather than merely named. `Arrow::run(&self, input) -> Out` admits
a lazy graph only when evaluating it is a pure function of its input, which holds once every leaf
draw is addressed by seed, index and leaf ordinal and nothing is read from ambient mutable state.
A graph that cannot meet that condition gets neither surface.

#### Scenario: The limitation is documented where a reader would look for it
- **WHEN** the sampling crate's composition documentation is read
- **THEN** it states that a lazy sampler is Arrow-shaped rather than witness-shaped, and that the container traits deliberately carry no `'static`

#### Scenario: The lazy graph composes through the Arrow layer
- **WHEN** a lazy computation graph whose evaluation at an index is pure is composed with a downstream operator
- **THEN** it composes through haft's `Arrow` combinators, the composite is a concrete generic type, and no trait object appears

#### Scenario: Node variants for the withheld surface are not retained
- **WHEN** a lazy graph's node representation is read
- **THEN** it holds no variant that exists only to serve an HKT container operation no constructor produces

### Requirement: The uncertain crate takes its distributions from stats alone

`deep_causality_uncertain`'s distribution layer SHALL name `deep_causality_stats`, and SHALL name `deep_causality_rand` only where it draws raw entropy or a range.

Every distribution type, distribution error and inverse-CDF function comes from `stats`. What
remains of `rand` in that crate is entropy and range sampling: the generator and thread RNG, the
Sobol sequence with `MAX_SOBOL_DIM`, and `Uniform<X>` with `UniformDistributionError`.

`Uniform<X>` is the one that looks like a distribution and is not. It is built on `SampleUniform`,
which can only be implemented in the crate that owns it, so it cannot move — the `stats-sampling`
capability records the same ruling from the other side. Range sampling is entropy wearing an
interval, and naming the entropy crate to reach it is truthful rather than a leftover.

The renovation this capability deferred — `SampledValue`, the global sample cache and the lazy
graph — is now specified by `uncertain-realfield-generic`, `uncertain-sample-session` and
`uncertain-arrow-graph`.

#### Scenario: The distribution imports come from stats
- **WHEN** `deep_causality_uncertain`'s sources are read
- **THEN** every distribution type, distribution error and inverse-CDF function is imported from `deep_causality_stats`

#### Scenario: What remains of rand is entropy and range sampling
- **WHEN** the same sources are searched for `deep_causality_rand`
- **THEN** every remaining reference is to a generator, the thread RNG, the Sobol sequence, or the range sampler `Uniform<X>` and its error
- **AND** no shaped distribution, distribution error or inverse-CDF function is among them

#### Scenario: The word draws say so
- **WHEN** the crate's raw index draws are read
- **THEN** they use the machine-word method rather than the numerical one

#### Scenario: The crate still passes its suite
- **WHEN** `deep_causality_uncertain`'s test suite runs
- **THEN** it passes at or above its pre-change count
