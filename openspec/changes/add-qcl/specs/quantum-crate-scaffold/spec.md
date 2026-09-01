<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: Verifiable and emergent modalities are kept type-distinct

The crate SHALL separate the verifiable modality (deterministic simulated Choi–Jamiołkowski
operators, the default build) from the emergent modality (a physical cloud-QPU call), exposing the
emergent path only as a `QpuSampler`-style seam behind an off-by-default `qpu` feature, and SHALL NOT
add any network or async dependency. The reified circuit data types `GateOp` and `QuantumCircuit`
SHALL be compiled in every build, because the always-on Haruna gate layer emits them; the seam that
executes a circuit against shots SHALL remain gated.

#### Scenario: Default build is verifiable-only

- **WHEN** `deep_causality_quantum` is built with default features
- **THEN** only the verifiable simulated-CJ path compiles, the `QpuSampler` seam and `SimQpu` are
  absent, and no network/async dependency is pulled in

#### Scenario: The circuit alphabet is data, not the seam

- **WHEN** `deep_causality_quantum` is built with default features
- **THEN** `GateOp` and `QuantumCircuit` are available, since `logical_z`, `logical_s`, `logical_t`,
  `logical_cz`, `logical_multi_cz` and `logical_hadamard` return gate programs over them and those
  builders are not feature-gated

#### Scenario: The emergent seam is a trait, not an adapter

- **WHEN** the `qpu` feature is enabled
- **THEN** a `QpuSampler` seam trait is available whose implementations return measurement shots as
  classical/`Uncertain` data at the Kleisli cut, and no concrete vendor adapter is shipped

#### Scenario: A named shot budget is a compile-time selection

- **WHEN** a QCL configuration names a shot budget in a build without the `qpu` feature
- **THEN** the program fails to compile, because the modality split is a compiler guarantee and a
  runtime rejection would erase it

## ADDED Requirements

### Requirement: Precision is a parameter on every numeric axis

Every numeric surface in the crate SHALL be written against an algebraic bound and SHALL name a
concrete width exactly once, through a type alias rather than a literal type. Three axes carry
their own parameter, and they are not interchangeable: `FloatType` for the real axis, `IntType` for
the integer axis, and exact rationals where a quantity is a ratio of integers by construction.

The real axis buys **accuracy**, and its failure mode is rounding bounded by `R::epsilon()`, which is
what every tolerance policy derives from. The integer axis buys **headroom**, and its failure mode is
overflow, which has no `epsilon()` to bound it, so it is served by checked arithmetic rather than by
a tolerance. A quantity that is exactly a ratio SHALL be carried as `Rational` rather than
approximated, so that the question asked of it is decided rather than thresholded.

#### Scenario: Swapping the float alias re-types the run

- **WHEN** `FloatType` is changed from `f64` to `Float106`
- **THEN** every operator, tolerance and threshold re-types with it, with no code change, and each
  tolerance tightens because it is a function of `R::epsilon()`

#### Scenario: Counts are ℕ and never a hardcoded width

- **WHEN** a shot count, an experiment count or a prediction count is stored
- **THEN** it is bounded on `NaturalNumber` and its width is named by `IntType`, and a draw-down uses
  `checked_difference` or `monus` rather than `Sub`, because ℕ is a `CommutativeSemiring` with no
  additive inverse

#### Scenario: An exact ratio is not given a tolerance

- **WHEN** a quantity is a ratio of integers by construction, such as a diagonal gate's phase
  `Q(n)/M` in turns
- **THEN** it is carried as `Rational` and compared exactly, and no tolerance is introduced for it,
  since a tolerance would answer a question the arithmetic never asks

#### Scenario: The float and integer axes are not conflated

- **WHEN** a design reaches for a tolerance on an integer quantity
- **THEN** it is rejected, because widening `IntType` changes headroom rather than accuracy and there
  is no graded error to bound
