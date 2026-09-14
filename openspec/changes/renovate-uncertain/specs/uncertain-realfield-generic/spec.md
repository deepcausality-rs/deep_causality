<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: Precision-generic uncertain types
`Uncertain<R>` and `MaybeUncertain<R>` SHALL be generic over a blanket-implemented scalar bound, and `deep_causality_uncertain/src` SHALL name no concrete scalar type in any implementation, constant or match arm.

The bound is `R: RandScalar` — `RealField + FromPrimitive`, blanket-implemented in
`deep_causality_rand` and re-exported by `deep_causality_stats`. A scalar joins by satisfying the
algebra and by nothing else, so a type added to `deep_causality_num` works here with no line changed
in this crate.

The computation graph carries the scalar: `ConstTree<Node<R>>` with `Sample<R> { Real(R), Bool(bool) }`
and one `Distribution(DistributionEnum<R>)` arm in place of the three the closed dispatcher needed.
The distribution carriers, both samplers and the SPRT evaluator carry `R`. The presence channel of
`MaybeUncertain<R>` remains `Uncertain<bool>`, because a presence event is a Boolean and not a real.

The closed `SampledValue` dispatcher and the `ProbabilisticType`, `IntoSampledValue`,
`FromSampledValue` and `UncertainReal` traits that served it are removed. The previous version of
this requirement already described a generic graph — "`SampledValue<R>`, `UncertainNodeContent<R>`"
— while the capability's own Purpose described the closed three-variant enum that was actually in
the tree. The requirement was never met; this states what it asked for and removes the dispatcher
that prevented it.

Dimensionless probabilities take the caller's scalar: the Bernoulli parameter, the comparison
thresholds, the SPRT threshold and confidence, and the returned probability estimate. `f64` appears
only at a display boundary. One bound does not move with the scalar and is documented where it
lives: `Bernoulli::new` holds its parameter as 64-bit fixed point, which is what makes probability
zero and probability one exact, so a wider scalar states a finer probability than the draw honours.

#### Scenario: A scalar the crate never names is drawn
- **WHEN** an `Uncertain<R>` is built from a normal distribution and sampled at a scalar `deep_causality_uncertain` mentions nowhere in its sources, such as `f32` or `BFloat16`
- **THEN** it compiles and the sample is of type `R`, drawn through the `R`-native distribution path with no round trip through another scalar

#### Scenario: No concrete scalar is named
- **WHEN** `deep_causality_uncertain/src` is searched for implementations, constants or match arms naming `f32`, `f64`, `Float106` or `BFloat16`
- **THEN** none is found outside a display boundary

#### Scenario: Probabilities are stated in the caller's scalar
- **WHEN** a Bernoulli parameter, a comparison threshold, an SPRT threshold or a probability estimate is read from the public API
- **THEN** its type is `R`, and the crate documents at the Bernoulli constructor that the parameter is honoured to a multiple of `2^-64` whatever scalar states it

#### Scenario: A generic downstream crate is no longer pinned
- **WHEN** `deep_causality_cfd`'s uncertain march is instantiated at `f32`
- **THEN** it compiles, and no bound in that crate names a trait from `deep_causality_uncertain` in order to restrict the scalar

## REMOVED Requirements

### Requirement: f64 behavior preserved bit-for-bit
**Reason**: This change is breaking by intent, and two of its parts make bit-for-bit preservation
impossible rather than merely inconvenient. Leaf draws are re-addressed by seed, index and leaf
ordinal, so no seeded sequence recorded before the change reproduces after it. Dimensionless
probabilities change type from `f64` to the caller's scalar, so the signatures the requirement
pinned no longer exist. Holding this requirement would forbid the renovation the rest of the
capability now describes.

**Migration**: The four aliases `UncertainF64`, `UncertainBool`, `MaybeUncertainF64` and
`UncertainF106` are kept, so call sites that spell the type through an alias — 45 in
`deep_causality`, 12 in `deep_causality_cfd` — need no edit. Call sites passing a probability
literal need no edit either, because an untyped float literal infers the caller's scalar. Code that
recorded a seed and asserted on the resulting values must re-record: replace `seed_sampler(s)` with
a `SampleSession::seeded(s)` threaded through the draw, run once, and pin the new values. Code
naming `SampledValue`, `ProbabilisticType`, `IntoSampledValue`, `FromSampledValue`, `UncertainReal`,
`GlobalSampleCache`, `with_global_cache` or `SamplerKind` must drop those names; none of them has a
replacement, because each existed only to serve the removed dispatcher and cache.
