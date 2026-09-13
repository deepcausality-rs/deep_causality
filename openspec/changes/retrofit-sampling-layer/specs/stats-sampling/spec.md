<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# stats-sampling Specification

## Purpose

Give `deep_causality_stats` the half of statistics it is missing. It models distributions as
densities today and cannot draw from them; this capability puts the samplers beside the densities,
under the scalar bound the crate already uses, and states what each distribution guarantees —
including the ones whose usual guarantees do not exist.

## ADDED Requirements

### Requirement: Distributions live beside the densities they belong to

`deep_causality_stats` SHALL own the real-valued distributions and their samplers, and SHALL depend on `deep_causality_rand` for entropy alone.

The crate already carries `gaussian_log_density`, `entropy`, `log_sum_exp`,
`bernoulli_proportion` and `covariance_matrix`, all bounded `T: RealField + FromPrimitive`, with no
dependency on `rand`. It knows what a Gaussian is and cannot draw from one. The sampler for that
same Gaussian sits two tiers below, in a crate that supplies bits.

Putting them together is what makes the rest possible. Importance sampling needs a density ratio;
sequential Monte Carlo needs `log_sum_exp` for weight normalisation; MCMC diagnostics need the
moment machinery. Each of those is a few lines once the sampler and the density are in one crate,
and none can be written in `rand` without dragging the analytic surface down two tiers.

The dependency runs `stats -> rand`, tier 4 to tier 2, so it is downhill and no cycle arises.

#### Scenario: The distributions are exported from stats
- **WHEN** `deep_causality_stats`'s public API is read after the move
- **THEN** `Normal<T>`, `Uniform<T>`, `Bernoulli`, `StandardUniform`, `StandardNormal`, the `Distribution` trait, the inverse-CDF functions and the distribution error types are present

#### Scenario: The dependency direction is downhill
- **WHEN** the workspace dependency graph is computed
- **THEN** `deep_causality_stats` depends on `deep_causality_rand` and not the reverse

#### Scenario: A density and its sampler are reachable together
- **WHEN** a caller draws from a normal distribution and evaluates its log-density
- **THEN** both come from `deep_causality_stats` with one import

### Requirement: A generic caller states one scalar bound

A function generic in its scalar SHALL sample by naming only a bound on that scalar, and SHALL NOT carry a `where` clause naming a concrete distribution type.

This is the point of the retrofit. The project README's own Monte Carlo example reads
`fn monte_carlo<S: Scalar>(..) where StandardUniform: Distribution<S>`. No other math crate asks
for anything of that shape. The clause propagates: `Normal<F>` carries one in its **struct
definition**, and `topology`'s `metropolis.rs` carries two, five tiers above the crate that
defines them.

Inside `stats` the blanket compiles, because `stats` has no reason to sample a machine word and so
never meets the `u64` implementation that made it `error[E0119]` in `rand`.

#### Scenario: The Monte Carlo example loses its distribution clause
- **WHEN** a function bounded only on the scalar trait draws uniform and normal values
- **THEN** it compiles with no `StandardUniform: Distribution<S>` clause

#### Scenario: The tier-7 leak is repaired
- **WHEN** `topology`'s `CubicalReggeGeometry` Metropolis impl is built
- **THEN** it carries no `where` clause naming a distribution type

#### Scenario: No struct definition names a distribution bound
- **WHEN** the public types of `deep_causality_stats` are read
- **THEN** no `struct` or `enum` carries a `Distribution<..>` clause in its own generics

### Requirement: One body serves every scalar

Each distribution SHALL be written once, generically over the scalar bound, and neither crate SHALL carry a per-type source file for any float it supports.

`deep_causality_fft` is the reference: one blanket-implemented `FftScalar`, zero per-type files.
`rand` today carries `dist_float_32.rs`, `dist_float_64.rs`, `dist_float_106.rs` and
`dist_float_common.rs`, and its standard normal is an `f64` ziggurat that `f32` narrows from and
`Float106` cannot use — which is why `Float106` needed its own hand-written Box–Muller.

The normal SHALL be computed in the caller's scalar through `Real`'s `ln`, `sqrt`, `cos` and `pi`.
The only per-type fact a numerical draw may consult is how many 53-bit words its significand
absorbs, held as an associated constant on a capability trait.

#### Scenario: The per-type distribution files are gone
- **WHEN** the source trees of both crates are listed
- **THEN** no file implements a distribution for one named float type only

#### Scenario: A new scalar joins by algebra
- **WHEN** a real-field scalar carrying the width constant is used at a sampling call site
- **THEN** it samples with no new distribution implementation written for it

#### Scenario: The normal does not narrow
- **WHEN** a standard-normal value is drawn at `Float106`
- **THEN** the computation uses `Float106` transcendentals throughout rather than widening an `f64` result

### Requirement: A wide scalar receives its full entropy

A draw at a scalar whose significand exceeds 53 bits SHALL consume as many generator words as that significand absorbs, and SHALL NOT be a single narrow draw widened into the wider type.

This fails silently without a test. A single 53-bit draw returned as a `Float106` is an `f64`
wearing a wider type: it satisfies every bounds check and passes every moment test, and it defeats
the precision claim the alias discipline exists to make. Measured on the prototype: a naive
single-draw generic carried a non-zero low limb in **0 of 200** draws; consuming the declared
number of words carried one in **400 of 400**.

#### Scenario: A double-double draw differs from its own narrow round trip
- **WHEN** 200 `Float106` uniform draws are taken from a deterministic generator
- **THEN** more than half satisfy `Float106::from(f64::from(v)) != v`

#### Scenario: The width is declared per scalar rather than assumed
- **WHEN** the capability trait is read
- **THEN** it carries an associated constant naming the number of 53-bit words the scalar absorbs
- **AND** the default value serves every scalar of 53 bits or fewer

### Requirement: Every uniform draw lies in the half-open unit interval

A uniform draw SHALL satisfy `0 <= v < 1` for every supported scalar and every generator state, including the states whose rounding would carry the value onto the upper bound.

Not theoretical: a narrow significand can round a value drawn from `[0, 1)` onto exactly `1.0`,
leaving the interval every inverse-CDF transform assumes. Measured on `BFloat16`, whose 8-bit
significand hit `1.0` within 1 000 draws. The implementation SHALL reject and redraw rather than
clamping, which puts an atom of probability mass on one value, or pre-scaling, which biases every
draw to correct a rare one.

Several of the distributions below take `ln(u)` and therefore need `u > 0` as well. That stricter
interval SHALL be obtained by rejection at the point of use, not by clamping a zero draw to a small
positive value, which would place mass at that value.

#### Scenario: No draw reaches the upper bound
- **WHEN** at least 1 000 uniform draws are taken for each supported scalar
- **THEN** every value satisfies `0 <= v < 1`

#### Scenario: A rounding-to-one draw is redrawn, not clamped
- **WHEN** a generator state would round a narrow-significand draw to exactly `1.0`
- **THEN** the implementation draws again, and no single value below 1 receives the rejected mass

#### Scenario: A log-transform never sees a zero argument
- **WHEN** a distribution whose inverse CDF takes `ln(u)` is drawn 10 000 times
- **THEN** no result is infinite or `NaN`

### Requirement: The statistical contract holds at every precision

Uniform and normal draws SHALL satisfy their defining moments at every supported scalar, so that changing the working type changes precision and not distribution.

Precision as a parameter is only true if the distribution survives the switch. Measured on the
prototype at 50 000 draws: mean 0.4998 (`f32`, `f64`) and 0.4995 (`Float106`) against 0.5;
`E[x²]` 0.3332 against 1/3; normal variance 0.998 against 1.

Where a narrow scalar cannot hold an accumulated result the failure is the scalar's, not the
sampler's: `BFloat16` saturates a running sum at 256 by the mechanism the project README records as
"BFloat16 stops adding at k = 23". The requirement is on the draws, not on a caller's accumulation
at a scalar too narrow to hold it.

#### Scenario: Moments hold across the supported scalars
- **WHEN** 50 000 uniform and normal draws are taken at `f32`, `f64` and `Float106`
- **THEN** the uniform mean is within 0.02 of 0.5, `E[x²]` within 0.02 of 1/3, and the normal variance within 0.05 of 1

#### Scenario: A narrow scalar's draws are correct even where its sums are not
- **WHEN** `BFloat16` uniform draws are averaged over a count small enough to avoid saturation
- **THEN** the mean is within 0.05 of 0.5
- **AND** the saturation of a larger sum is documented as a property of the scalar

### Requirement: Six further distributions, each verified against its closed form

`deep_causality_stats` SHALL gain Exponential, LogNormal, Cauchy, Weibull, Categorical and Poisson, each generic in the scalar and each checked against an exact analytic result.

Each is one to three lines over `Real` once the foundation lands, and none can be written that way
today. All six were implemented and measured before being proposed: Exponential(2) mean 0.5002
against 0.5; LogNormal(0, 0.5) mean 1.1330 against `e^0.125 = 1.1331`; Weibull(2, 1) mean 0.8861
against `sqrt(pi)/2 = 0.8862`; Categorical over weights 1:3:6 gave 0.100/0.300/0.600; Poisson(3)
mean 3.0004. Exponential at `Float106` gave 0.4995 with no additional code.

The verification SHALL be against the closed form, not against a second implementation. A sampler
tested against a reimplementation of itself tests neither.

#### Scenario: Each distribution matches its analytic mean
- **WHEN** 200 000 draws are taken of Exponential, LogNormal, Weibull and Poisson at `f64`
- **THEN** each sample mean is within 1% of the closed-form mean for its parameters

#### Scenario: The categorical respects its weights
- **WHEN** 200 000 categorical draws are taken over weights `1:3:6`
- **THEN** the observed frequencies are within 0.01 of `0.1`, `0.3` and `0.6`

#### Scenario: Each distribution runs at every supported scalar
- **WHEN** each of the six is drawn at `f32`, `f64` and `Float106`
- **THEN** each returns values of that scalar with no per-type implementation

### Requirement: A discrete uniform covers unweighted choice over a range

`deep_causality_stats` SHALL provide a discrete uniform distribution over an integer range, so that choosing one item out of `n` with equal probability is a distribution rather than a raw word draw.

Categorical covers weighted choice; nothing covers the unweighted case, which is the more common
one. Picking a lattice edge, a graph vertex or an array index uniformly is the same operation each
time, and today every caller writes `(rng.next_u64() as usize) % n` by hand.

That hand-written form is also wrong in a way worth naming. Modulo of a uniform 64-bit word is
biased whenever `n` does not divide `2^64`: the low residues occur once more often than the high
ones. The bias is negligible for small `n` and real for large ones, and it is invisible in any test
that only checks the result is in range. The distribution SHALL be free of modulo bias, by
rejection or by an equivalent method, and SHALL say which it uses.

This is what lets `deep_causality_topology` reach `stats` alone. Its Regge Metropolis step draws a
Gaussian length change and picks an edge; the first is already a distribution and the second becomes
one here.

#### Scenario: Every index in range is drawn about equally often
- **WHEN** 200 000 draws are taken over a range of 7, a size that does not divide any power of two
- **THEN** each of the 7 indices occurs within 1% of `1/7` of the draws

#### Scenario: The draw never leaves its range
- **WHEN** draws are taken over ranges of 1, 2 and 1000
- **THEN** every result lies inside the range, and a range of 1 always returns its single value

#### Scenario: Modulo bias is absent by construction
- **WHEN** the implementation is read
- **THEN** it rejects out-of-band words or uses an equivalent unbiased method, and its documentation names the method

#### Scenario: An empty range is refused
- **WHEN** a discrete uniform is constructed over an empty or inverted range
- **THEN** the constructor returns an error

### Requirement: A caller reaching stats for distributions needs no second sampling dependency

`deep_causality_stats` SHALL re-export, by name, exactly the generator traits a caller must name in order to pass a generator to a sampler, and SHALL NOT re-export the entropy crate's surface wholesale.

`Distribution::sample` takes `&mut R` where `R: Rng`. A crate that draws from a `stats`
distribution therefore has to name `Rng`, and without a re-export it would declare a dependency on
`deep_causality_rand` solely to spell a bound — a dependency on an entropy crate held by a crate
that wants only mathematics.

Re-exporting is the established pattern here: `deep_causality_calculus` re-exports
`deep_causality_haft::EndoArrow` and `deep_causality_algebra::Scalar` for the same reason.

**The re-export is named, never blanket.** A `pub use path::Trait` is an alias to the same trait
item, so a bound written against either path is satisfied by the same implementations — verified:
a function bounded on the re-exported path accepts a generator, and one bounded on the original
path accepts the same value, interchangeably. What must not happen is `stats` declaring its own
`Rng` trait of the same shape. That is a distinct item, and a value satisfying one would not satisfy
the other — verified as `error[E0277]: the trait bound R: deep_causality_rand::Rng is not
satisfied`. The two look identical at the call site and differ completely to the type checker.

A wholesale re-export is refused for a second reason: it would put every entropy type behind a
`stats` path, so two names would exist for each and a reader could not tell from an import which
crate owns the item. The list is short and SHALL be enumerated — the generator traits and nothing
else.

The re-export covers the generator *traits*, not the generators. A caller that wants to construct
an `Xoshiro256`, read an OS-entropy source or walk a Sobol sequence still depends on
`deep_causality_rand` directly, and should: those are entropy, and naming the entropy crate to
reach them is truthful rather than a wart.

#### Scenario: A distribution consumer names one crate
- **WHEN** a crate draws from a `stats` distribution and does nothing else with randomness
- **THEN** it imports the distribution and the generator bound from `deep_causality_stats`
- **AND** its manifest declares no dependency on `deep_causality_rand`

#### Scenario: An entropy consumer still names the entropy crate
- **WHEN** a crate constructs a generator, reads OS entropy or uses a Sobol sequence
- **THEN** it depends on `deep_causality_rand` directly

#### Scenario: The re-exported bound is the same trait, not a copy
- **WHEN** a generator is passed to a function bounded on the re-exported path and to one bounded on the original path
- **THEN** both compile for the same value, because the re-export is an alias rather than a new trait

#### Scenario: The re-export is enumerated
- **WHEN** `deep_causality_stats`'s re-exports are read
- **THEN** each is a named item, and no glob re-export of `deep_causality_rand` is present

### Requirement: Cauchy is tested by quantile, never by moment

The Cauchy distribution SHALL be verified through its median and quartiles, and SHALL NOT carry a test asserting a sample mean or variance.

The Cauchy distribution has **no** mean and no variance: the defining integrals diverge. A sample
mean of Cauchy draws does not converge as `n` grows — it wanders, and its own distribution is
Cauchy again. A test asserting one is not a weak test but a meaningless one, and it will fail
intermittently on a seed change while appearing to pass under review.

The location and scale are recoverable from quantiles: the median is the location, and the
interquartile range is twice the scale. Measured on the prototype, the median of 20 000 standard
Cauchy draws was −0.0149.

This requirement exists because the obvious test is the wrong one, and the obvious test is what a
later contributor will add if the reason is not recorded at the item.

#### Scenario: The median identifies the location
- **WHEN** 20 000 standard Cauchy draws are taken and sorted
- **THEN** the median is within 0.05 of 0

#### Scenario: The interquartile range identifies the scale
- **WHEN** the same sample is used
- **THEN** the difference between the 75th and 25th percentiles is within 0.1 of 2

#### Scenario: No moment test exists for Cauchy
- **WHEN** the Cauchy test module is read
- **THEN** it contains no assertion on a sample mean or variance
- **AND** the documentation states that neither exists

### Requirement: Poisson states the parameter range its algorithm serves

The Poisson sampler SHALL document the range of `lambda` over which its algorithm is correct and efficient, and SHALL behave predictably outside that range rather than silently degrading.

The Knuth product algorithm draws uniforms until their product falls below `exp(-lambda)`. Its
expected iteration count is `lambda + 1`, so it is fine for the small rates a simulation usually
wants and unacceptable at large ones. Worse, `exp(-lambda)` underflows to zero for `lambda` beyond
roughly 700 at `f64` and far sooner at `f32`, at which point the loop cannot terminate by its own
condition.

The sampler SHALL either refuse a `lambda` beyond its documented range with an error, or switch to
a method valid there. It SHALL NOT loop unboundedly. Which of the two is chosen is an
implementation decision; that one of them is chosen is not.

#### Scenario: The documented range is honoured
- **WHEN** Poisson is drawn at a `lambda` inside its documented range
- **THEN** the sample mean is within 1% of `lambda` and the sample variance within 5% of `lambda`

#### Scenario: An out-of-range rate is refused or handled, never hung
- **WHEN** Poisson is drawn at a `lambda` beyond the documented range, including one large enough that `exp(-lambda)` underflows at the working scalar
- **THEN** the call returns an error or a correct value, and does not loop indefinitely

#### Scenario: A zero rate returns zero
- **WHEN** Poisson is drawn at `lambda = 0`
- **THEN** every draw is `0`

### Requirement: Every distribution refuses parameters outside its support

A distribution constructor SHALL reject a parameter for which the distribution is undefined, and SHALL NOT return a value computed from one.

The crate's existing convention: `gaussian_log_density` refuses a non-positive variance rather than
returning "a plausible number where there is no answer". The samplers follow it. A negative scale,
a non-positive rate, a probability outside `[0, 1]`, an empty or all-zero categorical weight vector,
and a non-finite parameter are each undefined, and each is a caller error worth surfacing.

#### Scenario: A non-positive scale or rate is refused
- **WHEN** Normal is constructed with a negative standard deviation, or Exponential, LogNormal or Weibull with a non-positive rate, scale or shape
- **THEN** the constructor returns an error

#### Scenario: A probability outside the unit interval is refused
- **WHEN** Bernoulli is constructed with a probability below 0 or above 1
- **THEN** the constructor returns an error

#### Scenario: A degenerate categorical weight vector is refused
- **WHEN** a categorical distribution is constructed with no weights, with a negative weight, or with weights summing to zero
- **THEN** the constructor returns an error

#### Scenario: A non-finite parameter is refused
- **WHEN** any distribution is constructed with a `NaN` or infinite parameter
- **THEN** the constructor returns an error

### Requirement: No hardcoded f64 remains in the distribution API

Public functions SHALL take and return the caller's scalar rather than `f64`, except at a documented boundary where the underlying representation is genuinely fixed-width.

`Bernoulli::new(p: f64)` and `Bernoulli::p()` pin `f64` today, as do the inverse-CDF functions. A
program written against a `FloatType` alias meets a raw `f64` at each.

`Bernoulli` is the documented exception: it stores its parameter as 64-bit fixed point, so no
scalar wider than `f64` changes the draw. It SHALL accept and return the caller's scalar and state
the representable limit at the item.

#### Scenario: The distribution surface is free of f64 parameters
- **WHEN** the distribution API is read
- **THEN** none takes or returns `f64` except at an item whose documentation states why the width is fixed

#### Scenario: A program at a non-f64 alias constructs every distribution without a cast
- **WHEN** a program whose working type is `f32` or `Float106` constructs each shipped distribution
- **THEN** each accepts the working type with no cast at the call site
