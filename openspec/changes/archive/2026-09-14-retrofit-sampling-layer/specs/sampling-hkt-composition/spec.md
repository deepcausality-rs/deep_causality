<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# sampling-hkt-composition Specification

## Purpose

Put sampling on the composition surface the rest of unified math uses, and record — from
measurement rather than analogy — which categorical structures an ensemble carries, which it must
withhold, and why the monoidal reading is the meaningful one for correlated draws.

## ADDED Requirements

### Requirement: The sampling carrier is an existing witnessed container

Sampling SHALL produce values into a container that already carries an HKT witness, and this change SHALL NOT introduce a new container type to hold an ensemble of draws.

A carrier holding `n` realised draws is a `Vec<T>` with a witness, and the stack already has
several. `CausalTensor` carries `Functor`, `Foldable`, `Pure`, `Applicative`, `Monad` and
`Traversable` today, and a rank-1 tensor is exactly an ensemble. A parallel container would
duplicate that surface and would name in a sampling crate a concept `tensor` already owns.

Drawing into such a container is an ordinary generic function, fully monomorphised, needing no
witness of its own.

#### Scenario: An ensemble is an ordinary tensor
- **WHEN** `n` draws of a distribution are sampled into a rank-1 `CausalTensor`
- **THEN** the result carries every categorical structure that witness already provides
- **AND** neither `deep_causality_rand` nor `deep_causality_stats` declares a container type for this purpose

#### Scenario: The draw function needs no witness
- **WHEN** a function samples a distribution into a container
- **THEN** it is bounded on the scalar and the distribution only, with no HKT bound

### Requirement: A lazy sampler carrier is not given a witness

A carrier that stores a sampling closure SHALL NOT be given an HKT witness, and the reason SHALL be recorded rather than left to be rediscovered.

The container traits in `deep_causality_haft` carry no `'static` bound — measured: zero occurrences
across `Functor`, `Pure`, `Applicative`, `Monad`, `Traversable` and `LaxMonoidal`. That absence is
deliberate. A functor receives a function, applies it and drops it, so the function never outlives
the call. `Profunctor`, which does store its functions, carries `'static` on every parameter.

A carrier holding a stored closure therefore needs bounds the container traits do not provide, and
an implementation cannot add them: `error[E0276]: impl has stricter requirements than trait`. This
is not a gap in `haft`. The container traits are for data; a lazy sampler is a program, and haft's
home for programs is the `Arrow` layer.

#### Scenario: The limitation is documented where a reader would look for it
- **WHEN** the sampling crate's composition documentation is read
- **THEN** it states that a lazy sampler is Arrow-shaped rather than witness-shaped, and that the container traits deliberately carry no `'static`

### Requirement: Correlated draws pair by index, not by cartesian product

An ensemble traversal used for sampling SHALL pair the i-th draw of one quantity with the i-th draw of another, and SHALL NOT form the cartesian product across quantities.

This is a correctness matter rather than a performance one. Turning a field of ensembles inside out
through the ordinary `Traversable::sequence` uses the cartesian applicative. Measured on a 2×2
field of 50 draws per cell, the result had **6 250 000** entries — 50⁴, every combination across
the four cells.

For sampling that is meaningless. Draw *i* of the viscosity belongs with draw *i* of the inflow,
not with all fifty of them; correlation is by index. The diagonal reading is what
`ZipTensorWitness` already implements as `Semigroupal`.

#### Scenario: The diagonal traversal returns one field per draw
- **WHEN** a 2×2 field whose cells each hold 50 draws is turned inside out by the diagonal traversal
- **THEN** the result holds 50 fields, each of shape `[2, 2]`
- **AND** the i-th field holds the i-th draw of every cell

#### Scenario: The cartesian traversal is not used for sampling
- **WHEN** the composition documentation describes turning a field of ensembles inside out
- **THEN** it names the diagonal traversal and records that the cartesian one produces the product

### Requirement: The diagonal traversal is available without a unit

A traversal SHALL exist that requires only `Semigroupal` of the inner witness, and it SHALL take its starting accumulator as a parameter rather than deriving one.

`Traversable::sequence` requires `M: Applicative`, which requires `Pure`. `ZipTensorWitness`
deliberately has no `Pure`: the unit of a positional zip is the infinite repeat, which a finite
container cannot represent, and haft's own `LaxMonoidal` documentation prescribes that such a
witness implements `Semigroupal` and stops. So the diagonal witness cannot drive the existing
traversal — measured: `error[E0277]: the trait bound ZipTensorWitness: Applicative is not
satisfied`.

With no `Pure` there is nothing to build the starting accumulator from, so the caller supplies it.
That is the honest signature rather than a wart: it is where the absence of a unit surfaces, and
for sampling it means the ensemble size is declared rather than inferred, which is correct.

This gap is not specific to sampling. `ZipTensorWitness` and `ZipDenseVectorWitness` are equally
stranded today, so the traversal belongs in `deep_causality_haft` beside `Traversable`.

#### Scenario: A zip witness drives a traversal
- **WHEN** the diagonal traversal is called with a witness implementing `Semigroupal` but not `Pure`
- **THEN** it compiles and returns the diagonal result

#### Scenario: The empty structure takes the caller's seed
- **WHEN** the diagonal traversal is given a structure with no columns
- **THEN** the result is the seed the caller supplied

#### Scenario: Ragged columns truncate the ensemble, not the structure
- **WHEN** the columns hold differing numbers of draws
- **THEN** the ensemble truncates to the shortest column
- **AND** every field in the result retains every cell of the original structure

### Requirement: The uncertain crate takes its distributions from stats alone

`deep_causality_uncertain`'s distribution layer SHALL name `deep_causality_stats`, and SHALL name `deep_causality_rand` only where it draws raw entropy.

This is what the retrofit is for. `uncertain` imports `Bernoulli`, `Normal`, `Uniform` and the
`Distribution` trait from `rand` in `types/distribution/mod.rs`; the three distribution error types
in `errors/uncertain_error.rs`; and all four inverse-CDF functions in
`types/sampler/qmc_sampler.rs`. Every one of those is distribution mathematics, and after the move
every one comes from `stats`.

What remains of `rand` in that crate is entropy and range sampling: `Xoshiro256` and `rng()` in
`sampler_seed.rs` and `sequential_sampler.rs`, `SobolSequence` with `MAX_SOBOL_DIM` in the QMC
sampler, and `Uniform<X>` with `UniformDistributionError` in the distribution enum. Those are
correct dependencies on a source of numbers.

`Uniform<X>` is the one that looks like a distribution and is not. It is built on `SampleUniform`,
which can only be implemented in the crate that owns it, so it cannot move — the `stats-sampling`
capability records the same ruling from the other side. Range sampling is entropy wearing an
interval, and naming the entropy crate to reach it is truthful rather than a leftover.

The crate is next in line for its own renovation, and this change does not enter it beyond the
import moves and the two word-draw call sites. `SampledValue`, the global sample cache and the lazy
graph belong to that renovation.

#### Scenario: The distribution imports come from stats
- **WHEN** `deep_causality_uncertain`'s sources are read after the migration
- **THEN** every distribution type, distribution error and inverse-CDF function is imported from `deep_causality_stats`

#### Scenario: What remains of rand is entropy and range sampling
- **WHEN** the same sources are searched for `deep_causality_rand`
- **THEN** every remaining reference is to a generator, the thread RNG, the Sobol sequence, or the range sampler `Uniform<X>` and its error
- **AND** no shaped distribution, distribution error or inverse-CDF function is among them

#### Scenario: The word draws say so
- **WHEN** `types/sampler/sampler_seed.rs` is read
- **THEN** its two raw draws use the machine-word method rather than the numerical one

#### Scenario: The crate still passes its suite
- **WHEN** `deep_causality_uncertain`'s test suite runs after the migration
- **THEN** it passes at or above its pre-change count

### Requirement: The topology crate takes its randomness from stats alone

`deep_causality_topology` SHALL depend on `deep_causality_stats` and SHALL NOT depend on `deep_causality_rand`.

Counted before proposing: thirteen `rand` references in that crate's sources, and after the move
they reduce to one operation. Six are `RngType: Rng` bounds on the gauge-field methods, satisfied by
the re-exported generator trait. Three are distribution types in the Regge Metropolis step, which
move to `stats` with everything else. Two are `rng.random::<f64>()` draws in the gauge code, which
become `stats` uniform draws and stop being pinned to `f64` in the process.

The thirteenth is `(rng.next_u64() as usize) % num_edges` in `cubical_regge_geometry/metropolis.rs`,
picking which edge to perturb. That is the operation the discrete uniform exists for, and it is also
the one place topology's randomness is currently biased: modulo of a 64-bit word is not uniform when
the range does not divide `2^64`.

`RandomField` in `types/gauge/link_variable/random.rs` is topology's own hand-rolled uniform,
hardcoded to `f64` and documented as bridging "the gap between `deep_causality_rand` and algebraic
types". That gap is what this change closes, so the trait's `f64` implementation is replaced by a
`stats` draw at the caller's scalar, and the gauge field gains precision as a parameter it does not
have today.

`LawRng` in `utils_tests` is a hand-rolled deterministic generator with no dependency on either
crate, and stays as it is: a property test that cannot reproduce its own counterexample is worth
less than one that can.

#### Scenario: The manifest names stats and not rand
- **WHEN** `deep_causality_topology`'s manifest is read after the migration
- **THEN** it declares a dependency on `deep_causality_stats` and none on `deep_causality_rand`

#### Scenario: The edge pick is a distribution
- **WHEN** the Regge Metropolis step selects an edge
- **THEN** it draws from the discrete uniform rather than taking a modulo of a raw word

#### Scenario: The gauge field draws at the caller's scalar
- **WHEN** `RandomField::generate_uniform` is called for a field whose scalar is `f32` or `Float106`
- **THEN** it returns that scalar, drawn at that precision, with no `f64` intermediate

#### Scenario: The crate still passes its suite
- **WHEN** `deep_causality_topology`'s test suite runs after the migration
- **THEN** it passes at or above its pre-change count
