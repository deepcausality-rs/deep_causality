<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## MODIFIED Requirements

### Requirement: Precision-generic uncertain types
`Uncertain<R>`, `UncertainBool<R>` and `MaybeUncertain<R>` SHALL be generic over a blanket-implemented scalar bound, and `deep_causality_uncertain/src` SHALL name no concrete scalar type in any implementation, constant or match arm.

The bound is `R: UncertainScalar` — `RandScalar + 'static`, itself blanket-implemented over
`RealField + FromPrimitive` in `deep_causality_rand`. A scalar joins by satisfying the algebra and
by nothing else, so a type added to `deep_causality_num` works here with no line changed in this
crate.

`RandScalar` is the whole of the algebraic requirement and the only part a numeric type can fail.
The `'static` is the graph's, not the number's: the four unreachable higher-kinded node arms store
an `Arc<dyn …<R>>`, and a trait object's default lifetime reaches the type it is parameterised by.
Removing those arms removes the bound. A *mapped* function contributes nothing to it — it is held
as `fn(R) -> R`, a plain pointer — which is why `Send + Sync` are not in the bound.

`Real` is not a candidate for the bound and the weaker form is excluded rather than left open. A
probability is a ratio of two counts, and `Real` in `deep_causality_algebra` is
`CommutativeRing + PartialOrd + Neg + …` with no `Div`; division arrives with `Field`. `RealField`
is therefore the weakest structure that carries a frequency, and with `FromPrimitive` it is exactly
`RandScalar`. The same correction is recorded for the shot estimator in `qcl-evidence`.

The computation graph carries the scalar: `ConstTree<Node<R>>` with `Sample<R> { Real(R), Bool(bool) }`
and one `Distribution(DistributionEnum<R>)` arm in place of the three the closed dispatcher needed.
The distribution carriers, both samplers and the SPRT evaluator carry `R`.

**Two carriers over one graph.** A Boolean-valued node — a Bernoulli leaf, a comparison, a logical
combination — reads `Sample::Bool` from a tree whose other nodes are real, so it has no scalar of
its own and cannot be `Uncertain<bool>` once the scalar is a parameter. The crate therefore ships
two carrier structs over the same `ConstTree<Node<R>>`: `Uncertain<R>`, whose root yields
`Sample::Real`, and `UncertainBool<R>`, whose root yields `Sample::Bool`. Both keep `R`, because the
tree beneath a Boolean root holds real leaves: a Bernoulli parameter is `R`, a comparison threshold
is `R`, and the SPRT log-likelihood runs in `R`.

Two carriers are what `core.verdict.closure` requires, not a convenience. `Verdict` is instanced
twice over different algebras — the Boolean class on the Boolean carrier (`meet = &`, `join = |`,
`complement = !`) and the MV class on `[0, 1]` on the real carrier (`min`, `max`, `1 − p`). One
carrier cannot hold both, and dropping either breaks `Aggregatable: Verdict` in `deep_causality`.
Under two blanket instances over distinct local types the pair is coherent, and `Float106` gains the
MV instance it does not have today.

The presence channel of `MaybeUncertain<R>` is `UncertainBool<R>`: a presence event is a Boolean and
not a real, and it shares the value channel's `R` because both are one tree drawn at one index.
`MaybeUncertain<R>` means a probabilistically-present **real**; the Boolean form
(`MaybeUncertain<bool>`, alias `MaybeUncertainBool`) is removed. It had no use outside this crate's
own tests, and carrying it forward would add a third near-identical carrier struct to the change
whose purpose is deleting per-type duplication.

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

#### Scenario: Both verdict algebras survive the scalar parameter
- **WHEN** a collection of causaloids carrying `UncertainBool<R>` and a collection carrying `Uncertain<R>` are each aggregated through `Aggregatable`
- **THEN** both compile at any `R`, the Boolean carrier aggregates under `&` / `|` / `!` and the real carrier under `min` / `max` / `1 − p`, and neither instance overlaps the other

#### Scenario: A Boolean node keeps the tree's scalar
- **WHEN** an `Uncertain<R>` is compared against a threshold of type `R`, or a Bernoulli leaf is built from a parameter of type `R`
- **THEN** the result is an `UncertainBool<R>` over the same `ConstTree<Node<R>>`, and no conversion to or from another scalar occurs at the Boolean boundary

## REMOVED Requirements

### Requirement: f64 behavior preserved bit-for-bit
**Reason**: This change is breaking by intent, and two of its parts make bit-for-bit preservation
impossible rather than merely inconvenient. Leaf draws are re-addressed by seed, index and leaf
ordinal, so no seeded sequence recorded before the change reproduces after it. Dimensionless
probabilities change type from `f64` to the caller's scalar, so the signatures the requirement
pinned no longer exist. Holding this requirement would forbid the renovation the rest of the
capability now describes.

**Migration**: The alias module is removed entirely — `UncertainF64`, `UncertainF106`,
`MaybeUncertainF64`, `MaybeUncertainF106`, `UncertainBool` and `MaybeUncertainBool` are all gone.
Once the scalar is a parameter an alias adds nothing the instantiation does not already say, and the
one name a downstream crate does want bare, `UncertainBool`, is taken by the struct itself. A crate
that wants it declares `pub type UncertainBool = deep_causality_uncertain::UncertainBool<f64>` in
its own alias module: an alias belongs to whoever picked the scalar, and `deep_causality` picks it
for its 45 call sites, which then need no edit. `MaybeUncertainBool` has no replacement at all — see
the Boolean form's removal above.

A call site spelling `Uncertain<bool>` is now a compile error rather than a silent change of
meaning: 3 such sites in `deep_causality/src`, 13 in `deep_causality/tests`, 5 in
`deep_causality_ethos/tests`, 2 in `deep_causality_cfd/tests` and 3 in `deep_causality_quantum/src`.
Call sites passing a probability
literal need no edit either, because an untyped float literal infers the caller's scalar. Code that
recorded a seed and asserted on the resulting values must re-record: replace `seed_sampler(s)` with
a `SampleSession::seeded(s)` threaded through the draw, run once, and pin the new values. Code
naming `SampledValue`, `ProbabilisticType`, `IntoSampledValue`, `FromSampledValue`, `UncertainReal`,
`GlobalSampleCache`, `with_global_cache` or `SamplerKind` must drop those names; none of them has a
replacement, because each existed only to serve the removed dispatcher and cache.
