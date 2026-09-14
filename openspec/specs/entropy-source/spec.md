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

`Rng::random_bool(p: f64)` and `SobolSequence::coordinate`/`point` pin `f64` today. A program
written against a `FloatType` alias meets a raw `f64` at each, which is the leak the alias
discipline exists to prevent.

`SobolSequence` is the documented exception: its direction numbers resolve 32 bits per coordinate
by construction, a limit no wider scalar lifts. It SHALL accept and return the caller's scalar and
SHALL state the resolution limit at the item, so that a `Float106` caller is not misled into
expecting 106 bits of stratification.

#### Scenario: A program at a non-f64 alias reads a Sobol coordinate without a cast
- **WHEN** a program whose working type is `f32` or `Float106` reads a Sobol coordinate
- **THEN** it receives the working type with no cast at the call site

#### Scenario: The resolution limit is stated rather than implied
- **WHEN** `SobolSequence`'s documentation is read
- **THEN** it states that a coordinate carries at most 32 bits of resolution regardless of the scalar

#### Scenario: Two indistinguishable coordinates are documented as such
- **WHEN** two `Float106` Sobol coordinates differing by less than the 32-bit resolution are compared
- **THEN** they are equal, and the documentation predicts this

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
