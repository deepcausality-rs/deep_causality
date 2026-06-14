# Milestone structure

Two sub-groups, each ending green (full tests on the touched crate in both feature
configurations, clippy/fmt clean) with a prepared commit message. Group A genericizes the
sampling primitives in `deep_causality_rand` (bottom of the stack); Group B threads
`R: RealField` through `deep_causality_uncertain` on top of them. The f64 bit-identical
regression gate (D7) runs in both groups and is the load-bearing acceptance check.

Per AGENTS.md golden rules: agents never `git commit` and never delete files — each group
gate prepares a commit message and asks the user to commit. `make` targets are run by the
user on review.

## A. R-generic sampling primitives (rand-realfield-sampling)

- [x] A1 `SampleUniform` + `RandFloat` for `Float106` (`extensions/uniform/uniform_f106`),
      reusing the existing double-double `[0,1)` construction in
      `extensions/distribution/dist_float_106` (53-bit hi + scaled-53-bit lo); no `RngCore`
      change.
- [x] A2 `Float→Real`/`RealField` re-bound of `Normal`/`UniformFloat` so `RealField`
      consumers reach them without `Float`; `Distribution<Float106> for StandardNormal`
      (Box–Muller via `Real` transcendentals); `RealRng` convenience bound bundling the
      capabilities. (`Bernoulli` left f64-probability — output is `bool`, a dimensionless
      probability; noted in design D1/proposal.)
- [x] A3 f64 + f32 unchanged: the existing 14 rand tests pass; the per-type entropy seam and
      `StandardNormal: Distribution<{f32,f64}>` impls are untouched.
- [x] A4 Statistical acceptance on the `Float106` draws: mean/σ on the standard normal and
      the ranged uniform, plus a double-double-entropy (nonzero low limb) check. (Used
      mean/variance rather than a full KS test — sufficient for the rung.)
- [x] A5 Group gate: `make format` clean; full `deep_causality_rand` tests green via Bazel.
      Refactor: `extensions/{distribution,uniform}` house the type-specific impls.

## B. Parameterize the uncertain engine over R (uncertain-realfield-generic)

Design revised mid-implementation (see design D3): a **closed `SampledValue` enum
dispatcher** replaces the `SampledValue<R>` / generic-static plan — it keeps the cache,
graph, and sampler non-generic while still propagating `Float106`.

- [x] B1 `SampledValue` is the closed dispatcher `{ Float(f64), DoubleFloat(Float106),
      Bool(bool) }`; `UncertainNodeContent` gains a `DistributionF106` leaf; the engine stays
      non-generic.
- [x] B2 `NormalDistributionParams<R>` / `UniformDistributionParams<R>` (unbounded structs);
      `DistributionEnum<T>` uses `params<T>`; `DistributionEnum<Float106>::sample` draws via
      the Group A `Real`-bounded surface. (`BernoulliParams` stays f64-probability.)
- [x] B3 Sampler dispatches per `SampledValue` variant (arithmetic/comparison/negation/
      function leaves); `ArithmeticOperator::apply`/`ComparisonOperator::apply` generic over
      `RealField`/`Real`. The global cache stays a plain `static` (it only provides top-level
      `sample_with_index` reproducibility — the design fork the user resolved with the enum).
- [x] B4 `Uncertain<R>` / `MaybeUncertain<R>`: generic source-compatible constructors
      (`point`/`normal`/`uniform` via the `UncertainReal` node-builder trait, single impl so
      `Uncertain::normal(0.0, 1.0)` still infers f64); `Float106` sampling + arithmetic ops
      (generic over `ProbabilisticType + RealField`, excludes bool); `MaybeUncertain<Float106>`
      with `lift_to_uncertain`; `is_present` stays `Uncertain<bool>`.
- [x] B5 f64 preserved: all 23 existing uncertain tests pass unchanged; aliases retained,
      `UncertainF106`/`MaybeUncertainF106` added. The comparison-threshold and `.map`-closure
      f64 boundaries are documented in code + design D3.
- [x] B6 Precision-propagation tests (`integration_tests/float106_precision_tests`): certain
      `point(Float106)` lossless, arithmetic composes at full precision, normal/uniform draws
      carry double-double entropy, MaybeUncertain present/dropout.
- [x] B7 Downstream unchanged: full workspace builds + 759 Bazel tests pass with **no source
      edits** to `deep_causality` (`data_uncertain_*`) or the three `causal_uncertain_examples`.
- [x] B8 Group gate: `make format` clean; full workspace green via Bazel. Change exit:
      `MaybeUncertain<Float106>` available for Stage 4 Group C. (User runs the final global
      `make` format/test + commit.)
