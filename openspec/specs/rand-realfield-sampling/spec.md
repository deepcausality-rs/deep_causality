# rand-realfield-sampling Specification

## Purpose
`deep_causality_rand`'s distribution surface must be consumable by precision-generic code
bounded on `Real`/`RealField`, not `Float`. Because `RealField` is blanket-implemented as
`impl<T: Float> RealField for T` (Float ⇒ RealField, never the reverse), a `RealField`
consumer cannot reach a `Float`-bounded API without re-coupling to the bit-level trait —
which would break the platform's "a new float type needs only a num-crate `impl Float`"
mechanism. This capability keeps the distribution math on `Real`, confines the genuinely
bit-level seam (entropy generation) to per-type impls, and adds honest `Float106` sampling,
so downstream crates (e.g. `deep_causality_uncertain`) abstract over `RealField` alone.
## Requirements
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

### Requirement: The generator trait provides an in-place shuffle

`Rng` SHALL provide a shuffle that permutes a mutable slice uniformly, so that no consumer writes its own.

Two verbatim Fisher–Yates implementations exist downstream, both six lines, both drawing through
`random_range(0..(i + 1))`. One is generic over the generator and one is fixed to a concrete one.
They agree, which makes this the cheapest kind of absorption: one implementation, two call sites
retired, no semantic question to settle.

The draw is the standard backward Fisher–Yates, so the permutation for a given seed is determined by
the sequence of range draws and is reproducible.

#### Scenario: A permutation is produced
- **WHEN** a slice is shuffled
- **THEN** the result is a permutation of the input — same multiset, possibly different order

#### Scenario: The permutation is uniform
- **WHEN** a slice of three elements is shuffled many times from a seeded generator
- **THEN** each of the six permutations appears with a frequency consistent with uniformity at the sample size used

#### Scenario: Degenerate slices are handled
- **WHEN** an empty slice or a single-element slice is shuffled
- **THEN** the call succeeds and leaves the slice unchanged

#### Scenario: The result is reproducible under a fixed seed
- **WHEN** the same slice is shuffled twice from generators seeded identically
- **THEN** both produce the same permutation

#### Scenario: The two downstream copies are retired
- **WHEN** the workspace is searched for a Fisher–Yates implementation after this change
- **THEN** only the trait's implementation is found

### Requirement: A generic blanket Distribution over the real field is not attempted

The crate SHALL NOT add a blanket `Distribution<F> for StandardUniform` over `F: RealField + FromPrimitive`, and the generic-sampling need it was proposed for SHALL be met through the existing capability bound.

The assessment asked for this and it cannot be written. A blanket implementation over `RealField`
collides with the concrete `Distribution<u64>`, `Distribution<u32>` and `Distribution<bool>`
implementations on the same type, because `RealField` is upstream of this crate and the compiler
cannot rule out a future implementation of it for those types. The result is a coherence error, not a
design trade-off.

It would also be the wrong shape if it compiled. A sampler generic over `FromPrimitive` constructs
its value by converting a primitive, which for `Float106` means a draw carrying 53 bits of entropy
widened into a 106-bit type. The crate deliberately does the opposite: it assembles a double-double
draw from a high part and an independent scaled low part, precisely so the wide type receives wide
entropy.

The need behind the request is already met. The `RealRng` bound and the `Real`-bounded distribution
wrappers let precision-generic code sample without naming `Float`.

Two physics call sites still sample at `f64` and lift, and their module comments say the crate
implements `Distribution` only for `f32` and `f64`. That claim is stale. But those comments give a
second reason — that for a wider `R` the sampling noise sits at the `f64` floor anyway, so the lift
loses no meaningful entropy — and that is a claim about the physics, not about this crate. Whether
those sites should change is therefore not a question this capability can answer, and acting on it
would change a seeded random stream. It is tracked separately.

#### Scenario: Precision-generic sampling works through the existing bound
- **WHEN** code bounded on `R: RealField` with the sampling capability draws a uniform and a normal variate
- **THEN** it compiles and samples `R` values without naming `Float`

#### Scenario: The wide type keeps its entropy
- **WHEN** a uniform draw is taken at `Float106`
- **THEN** it carries double-double mantissa entropy, not a widened single draw

#### Scenario: The crate's own capability is not misstated downstream
- **WHEN** a consumer's documentation describes what this crate supports
- **THEN** it does not claim `Distribution` is implemented only for `f32` and `f64`

#### Scenario: The existing streams are unchanged
- **WHEN** the `f64` and `f32` sampling paths run under a fixed seed after this change
- **THEN** every produced sample is bit-identical to before

