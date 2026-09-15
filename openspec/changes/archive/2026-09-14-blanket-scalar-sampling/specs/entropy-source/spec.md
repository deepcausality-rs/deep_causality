<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# entropy-source Specification

## ADDED Requirements

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

## MODIFIED Requirements

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
