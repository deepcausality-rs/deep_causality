## Why

`deep_causality_uncertain` is the one numeric crate that is *not* precision-parametric.
`Uncertain<T: ProbabilisticType>` carries a `PhantomData<T>` front-end over a computation
graph whose leaf value type is hard-coded f64: `SampledValue = { Float(f64), Bool(bool) }`,
`UncertainNodeContent::DistributionF64(DistributionEnum<f64>)`, fmap/bind closures of
`Fn(SampledValue) -> SampledValue`, and `DistributionEnum<f64>::sample -> f64`. The
`impl MaybeUncertain<f64>` / `impl Uncertain<f64>` blocks are the only instantiations.

This is an f64 island in a platform whose thesis is "precision is a type parameter at the
call site" (`causal_cfd.md` §2.6; the `generalize-physics-over-realfield` and
`generalize-topology-over-realfield` changes already did this for their crates). The
island forces a lossy `R → f64` cast at every boundary where uncertainty meets the rest
of the stack — and casts are where precision silently disappears.

Two paths through the crate genuinely propagate precision and are degraded by the f64
island today:

- **Certain values.** `MaybeUncertain::from_value(x)` / `Uncertain::point(x)` involve no
  sampling at all; forcing a `Float106` `x` through f64 is pure, needless precision loss.
- **Deterministic arithmetic on sampled values.** Once drawn, `a * b + c` feeding an
  `R`-precise consumer (a `Float106` physics kernel, a high-precision Poisson solve) is
  `R`-precise iff the value type is `R`; the f64 island narrows it.

The honest boundary — stated up front and preserved in the design — is that the
**random draw itself is Monte-Carlo-bounded**: sampling error is O(1/√N), which swamps
f64 truncation, so `Float106` *sampling* does not reduce variance versus f64. This change
does not claim otherwise. It removes the cast island, makes the certain/arithmetic paths
lossless, and gives sampling an `R`-native (not f64-rounded) draw so the type is honest
end to end.

The trigger is the CFD Stage-4 uncertain-inflow zone (`add-cut-cells-and-immersed-boundaries`,
Group C), which wants `MaybeUncertain<R>` to compose with the `R: RealField` solver
without a boundary cast. That change is a *consumer*; this one is the prerequisite, so
the genericity is designed once, here, rather than worked around there.

## What Changes

- **`deep_causality_rand` — generalize the distribution surface from `Float` to `Real`
  (`rand-realfield-sampling`).** The crate predates the num crate's `Real` / `RealField`
  split and is over-coupled to `Float` (`Normal<F: Float>`, `UniformFloat<F: Float>`), even
  though the distribution math needs only `Real` (`is_finite`, `one`, `epsilon`, arithmetic,
  transcendentals — all on `Real` via `impl<T: Float> Real for T`). The change:
  - Re-bound the consumer-facing wrappers `Float → Real`: `Normal<F: Real>`,
    `UniformFloat<F: Real + RandFloat>`, so `RealField` consumers can use them without the
    forward-compat-breaking `Float` coupling.
  - Keep the per-type entropy seam (`RandFloat::rand_float_gen`, the mantissa-bit step) and
    `StandardNormal: Distribution<F>` as per-type impls — the legitimate low home for
    specific-type detail — and add `Float106` impls: the **double-double uniform `[0,1)`
    construction** (high part from a 52-bit draw, low part from a second scaled draw) and a
    **Box–Muller normal** via `Real` transcendentals.
  - Add a `RealRng` convenience bound (`Real + SampleUniform`, `StandardNormal:
    Distribution<Self>`, blanket-impl'd) so downstream threads one bound.
  - `Xoshiro256` / `RngCore::next_u64` is unchanged and type-agnostic; the **f64 and f32
    paths stay bit-identical** (regression-gated). `Bernoulli` stays f64-probability (its
    output is `bool`; a probability is dimensionless).
- **`deep_causality_uncertain` — parameterize the engine over `R: RealField`
  (`uncertain-realfield-generic`).**
  - `SampledValue<R> = { Float(R), Bool(bool) }`; `UncertainNodeContent<R>`,
    `DistributionEnum<R>`, `NormalDistributionParams<R>` / `UniformDistributionParams<R>` /
    `BernoulliParams<R>`, the fmap/bind closure traits, the `SequentialSampler`, the SPRT
    evaluator, and `GlobalSampleCache` all carry `R`.
  - `Uncertain<R>` / `MaybeUncertain<R>` and their arithmetic/comparison/logic ops become
    generic; `ProbabilisticType` (already the boundary trait) is implemented for the
    precision targets.
  - **f64 preserved bit-for-bit:** `UncertainF64 = Uncertain<f64>`,
    `MaybeUncertainF64 = MaybeUncertain<f64>`, `UncertainBool`, and every existing public
    method keep their signatures and numerical behavior. The genericity is purely
    additive; the existing test suite must pass unchanged.

## Impact

- **Affected specs (new capabilities):** `rand-realfield-sampling`,
  `uncertain-realfield-generic`.
- **Affected code:** `deep_causality_rand` (additive distribution impls),
  `deep_causality_uncertain` (engine parameterization). Both already carry the workspace
  `unsafe_code = "forbid"` lint; no `unsafe` is introduced.
- **Downstream (must remain green, unchanged):** `deep_causality` consumes uncertainty
  only through `data_uncertain_f64` / `data_uncertain_bool` context nodes and the f64
  aliases; the `causal_uncertain_examples` (clinical_trial, gps_navigation,
  sensor_processing) are all f64. The f64-preservation decision (above) keeps all of them
  compiling and numerically identical — no edits expected outside the two crates.
- **Honest non-goal:** this does not reduce Monte-Carlo sampling error at higher precision
  (it cannot — that is set by `N`, not by the float type). The win is the removed cast
  island plus lossless certain/arithmetic propagation. Documented at the sampling
  boundary, mirroring the accepted `R → f64` SURD boundary in `3DCausalFluidDynamics.md`.
- **Unblocks:** `add-cut-cells-and-immersed-boundaries` Group C (`MaybeUncertain<R>`
  inflow zone) — that change's Group C is sequenced after this one.
