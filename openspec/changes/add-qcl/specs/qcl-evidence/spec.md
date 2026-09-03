<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: `Evidence` and `ShotBudget` are compiled only under the `qpu` feature

`Evidence`, `ShotBudget` and the builder method that accepts them SHALL be compiled only when the
`qpu` feature is enabled, so that a configuration naming a shot budget in a default build is refused
by the compiler. The modality split is a compile-time guarantee, sampled read-outs belong to the
emergent path, and a runtime rejection would erase the separation the feature gate enforces.

The placement follows what ships. `src/types/qpu/mod.rs` compiles `circuit` in every build and gates
`sampler`, `sim` and `bridge` on `qpu`; the evidence types join the gated group. Three things leave
it for the same reason the circuit data types did: `ShotHistogram` and `CountHistogram` are plain
data with no dependency of their own, the Born sampler over the density-matrix carrier is a seeded
draw from a shipped probability, and the scalar-generic estimator is arithmetic over counts. All
three are compiled in every build, because the plant read-out needs them there. What selects the
emergent modality is naming a budget. `validate` stays reachable in both builds, since the Markov,
decomposability and code checks spend no device time.

#### Scenario: A shot budget in a default build fails to compile

- **WHEN** a program calls `.evidence(Evidence::shots(1024).seed(20260821))` on a config built with
  default features
- **THEN** compilation fails on the unresolved `Evidence` name, and no runtime path exists anywhere
  in the crate that accepts a shot budget and then rejects it

#### Scenario: The verifiable half of the same program still builds

- **WHEN** the program drops the `.evidence(...)` call and runs `QclBuilder::validate` under default
  features
- **THEN** it compiles and runs to a `Screened<R>`, because `check_markov`, `check_decomposable`,
  `check_ldpc_weights` and `check_class_invariance` take no shots

### Requirement: Shot statistics follow the scalar and never pin `f64`

Shot statistics SHALL be generic over the scalar and bounded at `R: RealField + FromPrimitive`,
covering the point estimate, its standard error and every separation quantity derived from a
histogram, and they SHALL NOT name `f64` outside a display or verdict boundary.

The design note's §6.4 had this row at `Real + FromPrimitive`, on the reasoning that `sqrt`, `log2`
and ratios touch no complex carrier so dual numbers should stay admissible. The premise fails on
the first line of the estimator: `p = k / n` is a ratio, and `Real` in `deep_causality_algebra` is
`CommutativeRing + PartialOrd + Neg + …` with no `Div`. Division arrives with `Field`, so the
weakest structure that carries a frequency is `RealField`, and the row is corrected rather than
worked around. Dual numbers are not admissible here, and the surface says so.

Two shipped functions pin the scalar and are the pattern this requirement excludes.
`shots_to_qubit_bernoulli` accumulates `ones as f64 / total as f64` and returns `Uncertain<bool>`;
`shots_to_observable` takes `F: Fn(usize) -> f64` and returns `Uncertain<f64>`. Both keep their
signatures, and the scalar-generic estimator is a sibling beside them.

#### Scenario: The same histogram is summarised at two precisions

- **WHEN** the read-out surface is instantiated at `f64` and again at `Float106` over the same
  `CountHistogram`
- **THEN** both compile, the standard error and the shot-noise threshold are computed at the
  instantiated scalar, and §10.4's precision sweep records a different tolerance for each

#### Scenario: The `Uncertain` boundary states its own restriction

- **WHEN** a scalar-generic estimate crosses into `Uncertain<R>`
- **THEN** the bound gains `ProbabilisticType`, which `deep_causality_uncertain` implements for
  `f64` and `Float106` and not for `f32`, and the signature carries that bound rather than the
  pipeline widening its `FloatType` alias to compensate

### Requirement: A read-out decision is a margin over shots

Every read-out compared against a spec SHALL produce a `CheckReport<R>` carrying the point estimate,
the threshold, the margin and the number of shots that produced it, and no stage SHALL select among
candidates by an exact-real comparison over point estimates. This closes friction F5, where decisions
were `min_by` over exact reals with shot noise absent from the comparison.

`Tolerance::shot_noise()` is the member of the §3.3 family that reads its width from the budget
rather than from `R::epsilon()`, so `Spec::at_least(ft(0.999)).within(Tolerance::shot_noise())`
carries both sources of uncertainty: sampling from the budget, and rounding from the scalar.

#### Scenario: An accepted read-out reports how it was accepted

- **WHEN** a 1024-shot read-out estimates `0.9991` against `Spec::at_least(ft(0.999))`
- **THEN** the report names the estimate, the threshold, the margin and the count 1024, and
  acceptance accounts for the standard error at 1024 shots

#### Scenario: Candidates that overlap within shot noise stay unseparated

- **WHEN** three forked hypothesis worlds report read-outs and two of them differ by less than the
  shot-noise width at the budget taken
- **THEN** `adjudicate` reports the separation achieved at those shots against the plan's
  `floor_bits` and returns the residual ambiguity, and it does not name the world with the largest
  point estimate as the survivor

### Requirement: The plant read-out samples the shipped Born probability

The shot sampler for the `DensityMatrix` carrier SHALL obtain each outcome probability from
`born_projective_probability` and SHALL NOT reimplement the Born rule, and it SHALL return outcome
counts through the shipped `ShotHistogram` trait so both sampling paths report one shape.

`born_projective_probability` ships in the default build, generic over
`R: RealField + FromPrimitive + Default + Debug`, taking `&DensityMatrix<R>` and `&Projection<R, D>`
and returning `Result<R, QuantumError>` clamped to `[0, 1]`. Its sibling `born_projective_prob`
collapses the value into `Prob`, whose payload is `f64`, which makes it the verdict boundary rather
than the estimator's input. `QpuSampler::sample` takes a `QuantumCircuit`, so the calibration plant
reaches no shipped sampler; the draw from a probability to a count is what this change adds.

#### Scenario: A projector read-out converges on `Tr(Pρ)`

- **WHEN** an `Observable`'s projector is read out on a plant state under a budget of shots
- **THEN** the Born value is computed once by `born_projective_probability` at the pipeline's scalar,
  the sampled frequency of the accepting outcome lies inside the reported shot-noise interval around
  it, and the counts are exposed through `total()`, `num_bits()`, `count()` and `entries()`

#### Scenario: A malformed read-out spends nothing

- **WHEN** the state dimension disagrees with the projection dimension `D`
- **THEN** the sampler propagates the shipped `QuantumError::DimensionMismatch` from the Born call,
  draws zero shots, and leaves the shot count unchanged

### Requirement: A count histogram refuses what it cannot hold

`CountHistogram::new` SHALL return `Result` and refuse a width above `usize::BITS`, the bits an
outcome carries, with `DimensionMismatch`. `record` and `record_n` SHALL return `Result`, SHALL
refuse with `DimensionMismatch` an outcome that does not fit the width, that is an outcome at or
above `2^num_bits` when `num_bits < usize::BITS`, and SHALL add to the outcome's count and to the
total in checked arithmetic, refusing an overflow with `CalculationError`. A refused record SHALL
leave the histogram unchanged.

#### Scenario: An outcome beyond the width is refused and nothing is recorded

- **WHEN** a two-qubit histogram holding one shot of outcome `3` is asked to record outcome `4`
- **THEN** `record` returns `DimensionMismatch`, the total stays at one, and the entries stay
  `[(3, 1)]`

#### Scenario: A count that would overflow is refused and nothing is recorded

- **WHEN** a histogram whose total is `u64::MAX` is asked to record one more shot of any outcome
- **THEN** `record` returns `CalculationError`, and the outcome's count and the total are unchanged

### Requirement: A shot budget is reproducible and its draw-down is checked ℕ arithmetic

`ShotBudget` SHALL carry a seed beside its count, SHALL bound the count on `NaturalNumber` with its
width named once by `NumberType`, and SHALL draw down through `checked_difference`, reporting the
shortfall when a request exceeds what remains. A run repeated at the same seed, budget and subject
SHALL produce the same histogram and the same verdict.

Both halves are grounded in shipped code. `SimQpu` seeds a splitmix64 and samples by inverse CDF, so
a fixed seed reproduces a histogram exactly, and it surfaces the seed through `SimCalibration`. ℕ is
a `CommutativeSemiring` with no additive inverse, so `NaturalNumber::checked_difference` returns
`None` on an overdraw and `monus` clamps, which is the draw-down semantics without a hand-written
guard.

#### Scenario: Two runs at one seed agree exactly

- **WHEN** the calibration pipeline runs twice at seed `20260821` with a 1024-shot budget over the
  same plant and probe family
- **THEN** both runs produce identical histograms and identical adjudication, and the seed appears in
  the provenance the run reports

#### Scenario: An overdrawn budget names the shortfall

- **WHEN** a stage requests 150 shots against 100 remaining
- **THEN** `checked_difference` yields `None`, the request is refused with a typed error naming the
  shortfall, and no shots, experiments or device time are recorded

### Requirement: A budget that buys no evidence is rejected rather than passed

`build()` SHALL reject a zero shot count, and any statistic taken from a histogram whose `total()`
is zero SHALL return a typed error rather than a probability. A read-out that examined nothing has
agreed with nothing, so the §3.2 count obligation applies to evidence exactly as it applies to the
structural checks.

#### Scenario: A zero budget is a construction error

- **WHEN** a config names `Evidence::shots(0)` and `build()` runs
- **THEN** `build()` returns a typed error naming the zero budget, and no `control` stage is
  constructed

#### Scenario: An empty histogram is not a probability

- **WHEN** the estimator is handed a histogram whose `total()` is zero
- **THEN** it returns `QuantumError::NormalizationError`, matching the shipped bridges that refuse an
  empty histogram with "cannot bridge an empty shot histogram"
