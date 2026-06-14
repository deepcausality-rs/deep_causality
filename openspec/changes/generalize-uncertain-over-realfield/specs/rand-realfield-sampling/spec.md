## ADDED Requirements

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
