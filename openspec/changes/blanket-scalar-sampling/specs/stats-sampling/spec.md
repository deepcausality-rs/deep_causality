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

### Requirement: Every uniform draw lies in the half-open unit interval

A unit draw SHALL be strictly below one at every scalar, and the function promising that interval SHALL be the one that enforces it.

A value drawn from `[0, 1)` can round onto exactly `1.0` in a narrow significand, leaving the
interval every inverse-CDF transform above it assumes. Measured at `BFloat16`, whose significand is
8 bits: 6 draws in 2 000. The rejection belongs in the shared accumulation rather than in each
caller — an earlier arrangement guarded it in this crate's `StandardUniform` while the entropy
crate's range sampler, calling the same accumulation, did not.

Rejecting costs a redraw at that rate and nothing at `f32` and wider, where the rate is `2^-25` or
below. Clamping instead would pile an atom of probability mass on one value.

#### Scenario: The unit draw honours its own interval
- **WHEN** many unit draws are taken at a scalar narrow enough to round onto one
- **THEN** none equals one

#### Scenario: The guard is not duplicated
- **WHEN** the unit draw is read
- **THEN** the rejection appears once, in the function that states the interval
