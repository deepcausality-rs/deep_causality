<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# entropy-source Specification

## MODIFIED Requirements

### Requirement: The generator surface takes the caller's scalar where it takes a scalar at all

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
