# uncertain-realfield-generic Specification

## Purpose
Precision is a **parameter** of `deep_causality_uncertain`, not a property baked into it. Every
type is generic in `R: RandScalar` — `RealField + FromPrimitive`, blanket-implemented — so a scalar
joins by satisfying the algebra and by nothing else: `f32`, `f64`, `Float106` and `BFloat16` all
work, and a scalar added to `deep_causality_num` tomorrow works with no line changed here. Nothing
in `src` names a concrete scalar outside two documented boundaries.

`Real` is not the bound and the weaker form is excluded rather than left open. A probability is a
ratio of two counts, and `Real` in `deep_causality_algebra` is a commutative ring with an order and
no `Div`; division arrives with `Field`. `RealField` is the weakest structure that can state a
frequency.

The scalar being a parameter forces a second thing: `Uncertain<bool>` cannot exist. A Bernoulli
leaf, a comparison and a logical combination all read a truth value off a tree whose leaves are
real, so `bool` is not something the tree can be parameterised by. The crate ships two carriers
over one `ConstTree<Node<R>>` — `Uncertain<R>` and `UncertainBool<R>` — and both keep `R`, because
the tree beneath a Boolean root holds `R`. What settles that is `core.verdict.closure`: `Verdict`
is instanced twice over two different algebras, Boolean on one carrier and MV on `[0, 1]` on the
other, and one type cannot hold both.

This is breaking by intent. The closed `SampledValue` dispatcher and the four traits that served it
are removed, dimensionless probabilities take the caller's scalar, and leaf draws are re-addressed,
so no seeded sequence recorded before the change reproduces after it.

## Requirements
### Requirement: Precision-generic uncertain types
`Uncertain<R>`, `UncertainBool<R>` and `MaybeUncertain<R>` SHALL be generic over a blanket-implemented scalar bound, and `deep_causality_uncertain/src` SHALL name no concrete scalar type in any implementation, constant or match arm.

The bound is `R: RandScalar` — `RealField + FromPrimitive`, blanket-implemented in
`deep_causality_rand` and re-exported by this crate so a downstream bound needs one dependency
rather than two. A scalar joins by satisfying the algebra and by nothing else, so a type added to
`deep_causality_num` works here with no line changed in this crate.

The bound is exactly the algebra and nothing is added to it. That is a property of the graph rather
than a convention: a trait object's default lifetime reaches the type it is parameterised by, so a
single `Arc<dyn …<R>>` in the node enum would put `R: 'static` on the whole public surface. There is
none. A mapped function is `fn(R) -> R`, a plain pointer, and the four higher-kinded arms that did
store one are removed — no public constructor built them, and a stored function belongs in the Arrow
layer over a composite whose type records what it holds.

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

### Requirement: Lossless precision on the certain and arithmetic paths
A certain value SHALL propagate `R` without narrowing, and deterministic arithmetic on
sampled values SHALL compose at full `R` precision. The crate SHALL document, at the
sampling boundary, that random draws are Monte-Carlo-bounded — higher precision does not
reduce sampling variance — so that the precision-genericity claim is not over-stated.

#### Scenario: Certain Float106 value round-trips without loss
- **WHEN** `MaybeUncertain::<Float106>::from_value(x)` is created for a `Float106` `x` whose low limb is non-zero and is read back through the present-value path
- **THEN** the recovered value equals `x` exactly, with no narrowing through f64

#### Scenario: Sampling boundary is documented as MC-bounded
- **WHEN** the crate documentation is inspected
- **THEN** it states that random draws are Monte-Carlo-bounded and that the value of precision-genericity is lossless certain/arithmetic propagation plus removal of the `R → f64` cast island, not reduced sampling variance

