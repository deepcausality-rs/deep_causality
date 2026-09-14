<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# stats-sampling Specification

## MODIFIED Requirements

### Requirement: One body serves every scalar

Each distribution SHALL be written once, generically over one blanket-implemented scalar bound, and neither crate SHALL state any per-type fact about a scalar it supports.

`deep_causality_fft` is the reference: one blanket-implemented `FftScalar`, zero per-type files,
and a new scalar joins by satisfying the algebra. The sampling layer now matches it.

**The width constant is withdrawn.** An earlier version of this requirement permitted one per-type
fact — how many 53-bit words a significand absorbs, held as an associated constant. It was
permitted because it looked irreducible; it was not. `Real::epsilon()` is on the bound these
functions already carry, and every scalar already reports its own precision through it, so the draw
stops when the next word would land entirely below that resolution:

```rust
let mut scale = word_scale;
while scale > Self::epsilon() {
    acc += word(rng) * scale;
    scale *= word_scale;
}
```

Derived rather than declared, this gives one word for `f32`, `f64` and `BFloat16` and two for
`Float106` — the same counts the table stated. What it removes is a hand-maintained list that had
to be edited for every new type, written twice: once as `RandWidth` in this crate and once as
`RandFloat::WORDS` in the entropy crate, two names for one invented fact.

The accumulation itself SHALL exist once in the workspace. This crate SHALL NOT define a sampling
capability trait of its own; it re-exports the entropy crate's by name.

#### Scenario: No per-type fact is stated anywhere
- **WHEN** either crate is searched for an implementation or constant declared for one named scalar
- **THEN** none is found, and the draw width is derived from the scalar's own epsilon

#### Scenario: A new scalar joins by algebra alone
- **WHEN** a real-field scalar is used at a sampling call site
- **THEN** it samples with no implementation, constant or list entry written for it

#### Scenario: The wide scalar still receives its full entropy
- **WHEN** many draws are taken at `Float106`
- **THEN** they carry bits below the `f64` rounding of themselves, so the second limb received an independent word

#### Scenario: The accumulation is not duplicated across the crate boundary
- **WHEN** the unit draw is read in both crates
- **THEN** one body exists and the other delegates to it

#### Scenario: The normal does not narrow
- **WHEN** a standard-normal value is drawn at `Float106`
- **THEN** the computation uses `Float106` transcendentals throughout rather than widening an `f64` result

### Requirement: A wide scalar receives its full entropy

A draw at a scalar whose significand exceeds 53 bits SHALL consume as many generator words as that significand absorbs, and SHALL NOT be a single narrow draw widened into the wider type.

This fails silently without a test. A single 53-bit draw returned as a `Float106` is an `f64`
wearing a wider type: it satisfies every bounds check and passes every moment test, and it defeats
the precision claim the alias discipline exists to make. Measured on the prototype: a naive
single-draw generic carried a non-zero low limb in **0 of 200** draws; consuming the full number of
words carried one in **400 of 400**.

How many words that is SHALL be derived from the scalar rather than declared for it. The previous
version of this requirement had the capability trait carry an associated constant naming the count;
that constant is withdrawn, and the loop instead stops when the next word would land entirely below
`Real::epsilon()`. The counts are unchanged — one word for `f32`, `f64` and `BFloat16`, two for
`Float106` — but nothing has to be edited when a scalar is added.

#### Scenario: A double-double draw differs from its own narrow round trip
- **WHEN** 200 `Float106` uniform draws are taken from a deterministic generator
- **THEN** more than half satisfy `Float106::from(f64::from(v)) != v`

#### Scenario: The width is derived rather than declared
- **WHEN** the sampling capability trait is read
- **THEN** it carries no constant naming a word count, and the accumulation terminates against the scalar's own epsilon

### Requirement: Every uniform draw lies in the half-open unit interval

A unit draw SHALL be strictly below one at every scalar, and the function promising that interval SHALL be the one that enforces it.

A value drawn from `[0, 1)` can round onto exactly `1.0` in a narrow significand, leaving the
interval every inverse-CDF transform above it assumes. Measured at `BFloat16`, whose significand is
8 bits: 6 draws in 2 000. The rejection belongs in the shared accumulation rather than in each
caller — an earlier arrangement guarded it in this crate's `StandardUniform` while the entropy
crate's range sampler, calling the same accumulation, did not.

Rejecting costs a redraw at that rate and nothing at `f32` and wider, where the rate is `2^-25` or
below. Clamping instead would pile an atom of probability mass on one value, and pre-scaling would
bias every draw to correct a rare one.

Several of the distributions above take `ln(u)` and therefore need `u > 0` as well. That stricter
interval SHALL be obtained by rejection at the point of use, not by clamping a zero draw to a small
positive value, which would place mass at that value.

#### Scenario: No draw reaches the upper bound
- **WHEN** at least 1 000 uniform draws are taken for each supported scalar
- **THEN** every value satisfies `0 <= v < 1`

#### Scenario: A log-transform never sees a zero argument
- **WHEN** a distribution whose inverse CDF takes `ln(u)` is drawn 10 000 times
- **THEN** no result is infinite or `NaN`

#### Scenario: The unit draw honours its own interval
- **WHEN** many unit draws are taken at a scalar narrow enough to round onto one
- **THEN** none equals one

#### Scenario: The guard is not duplicated
- **WHEN** the unit draw is read
- **THEN** the rejection appears once, in the function that states the interval
