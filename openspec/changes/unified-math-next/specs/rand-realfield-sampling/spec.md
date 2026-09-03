<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

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
wrappers let precision-generic code sample without naming `Float`, which is what the two physics call
sites that sample at `f64` and lift should be using. Their module comments claim the crate implements
`Distribution` only for `f32` and `f64`; that claim is false today, and correcting those comments and
call sites needs no change to this crate at all.

#### Scenario: Precision-generic sampling works through the existing bound
- **WHEN** code bounded on `R: RealField` with the sampling capability draws a uniform and a normal variate
- **THEN** it compiles and samples `R` values without naming `Float`

#### Scenario: The wide type keeps its entropy
- **WHEN** a uniform draw is taken at `Float106`
- **THEN** it carries double-double mantissa entropy, not a widened single draw

#### Scenario: The stale claims downstream are corrected
- **WHEN** the physics sampling sites are read after this change
- **THEN** they sample at their working precision through the capability bound, and no comment claims the crate supports only `f32` and `f64`

#### Scenario: The existing streams are unchanged
- **WHEN** the `f64` and `f32` sampling paths run under a fixed seed after this change
- **THEN** every produced sample is bit-identical to before
