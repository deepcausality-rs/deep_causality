## ADDED Requirements

### Requirement: Distribution wrappers bound on Real, not Float
The consumer-facing distribution wrappers SHALL be bounded on `Real` (the analytic trait),
not `Float` (the bit-level trait), so that `RealField` downstream code can construct and
sample them without coupling to `Float`. `Normal<F>` SHALL require `F: Real` (with the
standard-normal capability), and the uniform float sampler SHALL require `F: Real` plus the
per-type entropy seam. The only `Float`-level code SHALL be the per-type entropy generation
(mantissa-bit assembly) and the per-type standard-distribution impls; `Xoshiro256` and the
`RngCore` entropy source SHALL be unchanged. A `RealRng` convenience bound SHALL bundle the
sampling capabilities so downstream can thread a single bound.

#### Scenario: A RealField consumer samples without naming Float
- **WHEN** generic code bounded only on `R: RealField` (plus the `RealRng` capability bound) constructs and samples a normal and a uniform distribution
- **THEN** it compiles and samples `R` values without any `R: Float` bound

### Requirement: Precision-generic distribution sampling
`deep_causality_rand` SHALL provide `Distribution<R>` sampling for the standard uniform
and standard normal distributions, and `R`-typed parameters for the Bernoulli, uniform,
and normal distributions, for every supported `R: RealField` precision target — through
the existing `Distribution` / `SampleUniform` / `UniformSampler` trait surface, without
changing the `RngCore` entropy source. The `Float106` uniform draw SHALL be constructed
with genuine double-double mantissa entropy (a high 53-bit draw plus an independent scaled
low draw), not by widening a single f64 draw. The existing `f64` and `f32` sampling paths
SHALL remain bit-identical under a fixed seed.

#### Scenario: Float106 normal sampling uses RealField transcendentals
- **WHEN** `StandardNormal` is sampled at `R = Float106` from a seeded RNG
- **THEN** the result is a `Float106` produced via Box–Muller using `Float106` `sqrt`/`ln`/`cos`, with the uniform inputs carrying double-double entropy

#### Scenario: f64/f32 paths are unchanged
- **WHEN** the existing `f64` and `f32` distribution sampling runs under a fixed seed before and after this change
- **THEN** every produced sample is bit-identical

#### Scenario: Float106 draws are statistically valid
- **WHEN** a large seeded batch of `Float106` standard-uniform and standard-normal draws is collected
- **THEN** sample mean and variance match the analytic moments within sampling tolerance and a KS test does not reject the target distribution, and the low mantissa limb is not constant
