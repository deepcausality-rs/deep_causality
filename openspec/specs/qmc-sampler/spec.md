# qmc-sampler Specification

## Purpose
`Uncertain<R>`'s batch estimators converge at the Monte-Carlo rate `~1/√N`. For the
low-dimension presence/collapse gates evaluated at high sample counts, a
low-discrepancy sequence converges far faster (`~(log N)^d / N`). This capability
adds `QmcSampler` — an alternative `Sampler<T>` that draws each leaf by inverse-CDF
on a digitally shifted Sobol point — with an enforced soundness boundary (static
trees only; never SPRT) and an opt-in batch-estimator surface. The default
`SequentialSampler` and all existing f64 behaviour are unchanged.

## Requirements

### Requirement: QmcSampler as an alternative Sampler
`deep_causality_uncertain` SHALL provide a `QmcSampler` implementing `Sampler<T>`
for every supported `ProbabilisticType`, alongside the existing
`SequentialSampler`. `QmcSampler` SHALL evaluate an `Uncertain<R>` computation
graph by drawing each stochastic leaf through the inverse-CDF transforms (never
through rejection sampling), using one coordinate of a Sobol point per leaf. All
deterministic node kinds (arithmetic, comparison, logical, fmap/apply, function,
negation) SHALL be evaluated identically to `SequentialSampler`. `SequentialSampler`
SHALL remain the default; existing default behavior and seeded MC bit-output SHALL
be unchanged.

#### Scenario: QMC reduces error at equal sample count
- **WHEN** the mean of a low-dimension static `Uncertain<f64>` integrand is estimated with `QmcSampler` and with `SequentialSampler` at the same sample count `N`
- **THEN** the QMC estimate's error against the analytic value is no larger than the MC estimate's, and converges at a faster rate as `N` grows

#### Scenario: Deterministic node handling matches the MC sampler
- **WHEN** a tree mixing distributions with arithmetic, comparison, and logical operators is evaluated under QMC
- **THEN** the deterministic operators produce the same composition of leaf values as `SequentialSampler` would for those same leaf values

### Requirement: Fixed dimension assignment over static trees
`QmcSampler` SHALL perform a deterministic pre-pass over the immutable computation
graph that assigns each non-`Point` distribution leaf a stable dimension index in
`0..d`, where `d` is the number of stochastic leaves. The assignment SHALL be a
function of graph structure only, independent of sampled values, so that the same
leaf maps to the same Sobol coordinate across all sample indices.

#### Scenario: Each leaf keeps a stable coordinate across points
- **WHEN** a static tree with `d` distribution leaves is sampled at indices `0..N`
- **THEN** a given leaf draws from the same dimension of the Sobol sequence at every index, and exactly `d` dimensions are used

### Requirement: Enforced soundness boundary
`QmcSampler` SHALL refuse cases where Quasi-Monte Carlo is statistically invalid,
returning an error rather than a wrong result. The pre-pass SHALL reject any tree
containing a `BindOp`, or a `ConditionalOp` whose branches do not draw the same
set of distributions, with `UncertainError::SamplingError`. QMC SHALL NOT back the
SPRT decision methods (`to_bool`, `probability_exceeds`, `implicit_conditional`),
which retain Monte-Carlo sampling; no QMC variant of these SHALL be exposed.

#### Scenario: Dynamic structure is rejected
- **WHEN** `QmcSampler` is asked to sample a tree containing a `BindOp` or a branch-divergent `ConditionalOp`
- **THEN** it returns `Err(UncertainError::SamplingError(..))` and produces no sample

#### Scenario: SPRT remains Monte-Carlo
- **WHEN** the public surface for `to_bool` / `probability_exceeds` / `implicit_conditional` is inspected
- **THEN** there is no QMC-backed variant, and these decisions continue to draw i.i.d. Monte-Carlo samples

### Requirement: Reproducible, variance-estimable QMC via seeded shift
`QmcSampler` SHALL draw points from a digitally shifted Sobol sequence whose shift
is derived from an explicit `u64` seed supplied at construction (and threaded
through the opt-in batch estimators), so that a run with a given seed is
reproducible. Because the shift is randomized, `standard_deviation_qmc` over QMC
draws SHALL estimate genuine sampling error (not a degenerate zero). Construction
without a seed SHALL use the raw (unshifted) sequence.

#### Scenario: Seeded QMC run is reproducible
- **WHEN** a QMC batch estimate is computed twice with the same seed
- **THEN** both runs produce identical estimates

#### Scenario: Standard deviation is non-degenerate under QMC
- **WHEN** `standard_deviation_qmc` is computed over a QMC batch of a non-constant integrand
- **THEN** the result is a positive estimate of sampling error, not zero

### Requirement: Sampler-discriminated sample cache
The global sample-cache key SHALL be extended to include a sampler discriminant so
that Monte-Carlo and Quasi-Monte-Carlo samples for the same `(uncertain_id,
sample_index)` are stored separately and never cross-served within one process.

#### Scenario: MC and QMC samples do not collide
- **WHEN** the same `Uncertain` is sampled at the same index under both `SequentialSampler` and `QmcSampler` in one process
- **THEN** each sampler retrieves its own cached value, and neither observes the other's

### Requirement: Opt-in batch-estimator surface
QMC SHALL be selectable on the batch estimators `expected_value`,
`standard_deviation`, and `estimate_probability` through explicit `_qmc` methods
that take a digital-shift seed, never as implicit global state. The existing MC
signatures and their results SHALL be unchanged for callers that do not opt in.

#### Scenario: Default callers are unaffected
- **WHEN** an existing caller invokes `expected_value` / `standard_deviation` / `estimate_probability` without opting into QMC
- **THEN** the call uses `SequentialSampler` and returns the same result as before this change

#### Scenario: Opting into QMC routes through QmcSampler
- **WHEN** a caller invokes `expected_value_qmc` / `standard_deviation_qmc` / `estimate_probability_qmc` over a static low-dimension tree
- **THEN** the estimate is produced by `QmcSampler` using the Sobol/inversion path
