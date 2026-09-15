<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# entropy-source Specification

## Purpose

Fix what `deep_causality_rand` is: a source of machine entropy. It supplies bits, words, Booleans
and low-discrepancy point sets, and makes no claim about real-valued distributions. A crate that
needs a random index reaches it; a crate that needs a Gaussian does not.
## Requirements
### Requirement: The crate samples machine values and makes no claim about real distributions

`deep_causality_rand` SHALL expose generators, machine words, Booleans, byte fills, low-discrepancy point sets and uniform sampling over a range, and SHALL NOT define any distribution that needs the analytic surface.

The crate holds two unrelated things today. Entropy is genuinely low-level and belongs at tier 2:
`Xoshiro256`, `next_u64`, OS entropy. Distributions are mathematics over a real field, and they sit
two tiers below the analytic surface they need — Box–Muller in `rand` reaches for `Real::ln`,
`sqrt` and `cos` from a crate that cannot name them without inverting the stack.

Keeping both is also what blocks the retrofit. A blanket
`impl<T: RealField> Distribution<T> for StandardUniform` is `error[E0119]` against the `u64`,
`u32` and `bool` implementations, because coherence cannot prove those will never be real fields.
The conflict is a consequence of one crate sampling both kinds, and it disappears at the boundary.

`SobolSequence` stays. A Sobol point is a deterministic function of an index and a digital shift
drawn from `Xoshiro256`: a source of numbers in `[0, 1)`, not a statement about a distribution.

**The range machinery stays too**, and the boundary is drawn at the analytic surface rather than at
the word "distribution". A uniform over a range is arithmetic on its bounds; a normal needs `ln`,
`sqrt` and `cos`. Two measurements make this the only workable line: `impl SampleUniform for f64`
can only be written in the crate owning `SampleUniform`, and that trait backs `Rng::random_range`,
whose every external caller draws an **integer** range to pick an index. A graph library reaching a
statistics crate for an index is what this split exists to prevent.

The `Distribution` trait stays for the same reason: it is the bridge both crates speak.
`StandardWord` implements it here, and `stats` implements it for its own types — a local type
against a foreign trait, which the orphan rule permits.

#### Scenario: No analytic distribution remains
- **WHEN** the crate's exported types are listed after the split
- **THEN** `Normal`, `Bernoulli`, `StandardUniform`, `StandardNormal`, `Open01` and `OpenClosed01` are absent
- **AND** `Xoshiro256`, `SobolSequence`, the generator traits, and the range machinery including `Uniform<X>` are present

#### Scenario: Nothing in the crate reaches for a transcendental
- **WHEN** the crate's sources are searched for `ln`, `exp`, `sqrt`, `sin` or `cos`
- **THEN** none appears in a sampling path, because every draw it offers is bits or arithmetic on bounds

#### Scenario: A consumer that needs only an index depends on nothing more
- **WHEN** `ultragraph` or `deep_causality_data_structures` is built
- **THEN** it draws a random index through `deep_causality_rand` alone
- **AND** neither declares a dependency on `deep_causality_stats`

### Requirement: Machine words and Booleans are separate samplers

Machine words SHALL be sampled through `StandardWord` and Booleans through `StandardBool`, and one type SHALL NOT sample both.

These are different objects. A machine word is the generator's own output, carrying no algebra. A
Boolean is the two-element Boolean algebra. Conflating them under one name is the smaller instance
of the same error the crate boundary fixes, and separating them keeps the coherence conflict from
reappearing if a real-valued helper is ever added here.

The current Boolean draw is `next_u64() % 2 == 0`, a parity test that reads one bit's worth of
information from a 64-bit draw while consuming the whole word. The replacement takes a bit
directly. Measured on the prototype: true for 0.5013 of 10 000 draws.

#### Scenario: Each sampler answers for its own kind
- **WHEN** `StandardWord` is asked for a `u64` or a `u32`, and `StandardBool` for a `bool`
- **THEN** each returns a value of that type
- **AND** neither offers an implementation for the other's types

#### Scenario: The Boolean draw is unbiased
- **WHEN** 10 000 Boolean draws are taken from a deterministic generator
- **THEN** between 45% and 55% are true

#### Scenario: The call sites say which kind they mean
- **WHEN** a caller draws through `Rng`
- **THEN** `random_word` yields a machine word and `random_boolean` a Boolean
- **AND** neither name suggests a real-valued draw

### Requirement: The generator surface takes the caller's scalar where it takes a scalar at all

A public function of the entropy crate SHALL NOT take or return `f64` except at a documented boundary where the underlying representation is genuinely fixed-width.

`Rng::random_bool` and `SobolSequence::coordinate`/`point` were the two that pinned `f64`. Each
now takes the caller's scalar, so a program written against a `FloatType` alias meets no raw `f64`
at either. Where `f64` survives inside the crate it is a conversion pivot or a panic message, never
a parameter or a return type.

`SobolSequence` is the documented exception, and the exception is about resolution rather than
about the type: its direction numbers resolve 32 bits per coordinate by construction, a limit no
wider scalar lifts. It accepts and returns the caller's scalar and states the resolution limit at
the item, so that a `Float106` caller is not misled into expecting 106 bits of stratification.

`Bernoulli` in `deep_causality_stats` carries the same shape of exception from the other side: it
takes the caller's scalar and holds the probability as 64-bit fixed point, which is what makes
`p = 0` and `p = 1` exact, and it states that bound at its constructor.

#### Scenario: A program at a non-f64 alias reads a Sobol coordinate without a cast
- **WHEN** a program whose working type is `f32` or `Float106` reads a Sobol coordinate
- **THEN** it receives the working type with no cast at the call site

#### Scenario: The resolution limit is stated rather than implied
- **WHEN** `SobolSequence`'s documentation is read
- **THEN** it states that a coordinate carries at most 32 bits of resolution regardless of the scalar

#### Scenario: Two indistinguishable coordinates are documented as such
- **WHEN** two `Float106` Sobol coordinates differing by less than the 32-bit resolution are compared
- **THEN** they are equal, and the documentation predicts this

#### Scenario: No public function takes or returns a concrete float
- **WHEN** the crate's public function signatures are read
- **THEN** none names `f64` as a parameter or a return type, including the Bernoulli-style probability arguments

### Requirement: The superseded scalar bound is removed

`RealRng` SHALL be removed rather than kept.

It was an earlier attempt at this retrofit, and it has zero consumers in the workspace outside its
own test file. Its documentation promises that a new float type gains the bound "automatically ...
and downstream code is untouched". Two measurements contradict that: the seam it names is
`pub(crate)`, so no other crate can implement it, and made public the orphan rule still rejects a
foreign trait on a foreign type with `error[E0117]`. After the split it would also be a real-field
bound in a crate that no longer samples real fields.

#### Scenario: The bound is gone
- **WHEN** the crate's exported traits are listed
- **THEN** `RealRng` is absent

### Requirement: Every sampler implementation is blanket over an algebraic tower

Every sampler implementation in `deep_causality_rand` SHALL be blanket over an algebraic tower, and the crate SHALL name no concrete scalar type in any such implementation.

The range machinery stays in this crate, but its bindings do not. `SampleUniform` is implemented
once for the float tower and once for the unsigned tower, distinguished by a `Kind` type parameter
so the two blanket implementations do not overlap:

```rust
impl<T: RealField + FromPrimitive> SampleUniform<FloatKind>    for T
impl<T: RandUnsigned>              SampleUniform<UnsignedKind> for T
```

Without the parameter the two collide, and the crate's own comments once argued in a circle about
it: the float bindings cited the integers as the obstacle and the integer bindings cited the
floats. Neither was immovable. One trait cannot carry two blanket implementations over disjoint
towers; two parameterised implementations of one trait can.

`SampleRange` takes the same parameter and is implemented once for `Range<T>` across both towers.
`Rng::random_range` keeps one signature — the kind is inferred from the range, and no call site
names it.

The unsigned tower is served by one body over `Integer`'s `BITS` and `UnsignedInt`'s power-of-two
helpers, using bitmask rejection. The three hand-written copies it replaces had diverged: the
`usize` one drew from `next_u32`, so on a 64-bit target a range wider than `2^32` returned only its
bottom `2^32` — 200 000 draws over a range of `2^40` never exceeded 4 294 942 982.

#### Scenario: No sampler implementation names a type
- **WHEN** `deep_causality_rand`'s sources are searched for a sampler implementation on a concrete scalar
- **THEN** none is found, and every implementation is bounded on an algebraic tower

#### Scenario: A scalar the crate never names can be drawn
- **WHEN** a range is sampled at `BFloat16`, which appears nowhere in `deep_causality_rand`
- **THEN** it draws, because it satisfies the algebra

#### Scenario: Every unsigned width draws from a range
- **WHEN** a range is sampled at each of `u8`, `u16`, `u32`, `u64`, `u128` and `usize`
- **THEN** each returns a value inside the range, from one shared implementation

#### Scenario: A wide usize range reaches its top
- **WHEN** a `usize` range of width `2^40` is sampled many times
- **THEN** draws exceed `2^32`, which the superseded per-type sampler could not produce

#### Scenario: The range is half-open at every scalar
- **WHEN** a half-open range is sampled at any supported scalar, including one whose significand is narrow enough for the affine map to round onto the bound
- **THEN** no draw equals the upper bound, because a result that reaches it is rejected and redrawn

#### Scenario: The inclusive range still reaches its bound
- **WHEN** a range built by `new_inclusive` is sampled
- **THEN** the upper bound remains a legitimate result

