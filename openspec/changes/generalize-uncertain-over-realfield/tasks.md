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

- [ ] A1 `SampleUniform` + `UniformFloat` for the precision targets; the double-double
      `Float106` uniform `[0,1)` construction (hi 53-bit draw + lo scaled second draw),
      behind the existing `Distribution`/`SampleUniform` traits (no `RngCore` change).
- [ ] A2 `Distribution<R> for StandardUniform` and `for StandardNormal` (Box–Muller via
      `R: RealField` transcendentals); `Bernoulli` thresholds over `R`.
- [ ] A3 f64 + f32 bit-identical regression: the existing f64/f32 sampling paths produce
      identical output under a fixed seed before and after (the impls must not be
      perturbed by the new generic ones).
- [ ] A4 Statistical acceptance on the `Float106` draws: sample mean/variance and a KS
      check against the analytic uniform and normal; independence of the low limb.
- [ ] A5 Group gate: format, clippy, full `deep_causality_rand` tests both feature
      configs; prepare the Group A commit message and ask the user to commit.

## B. Parameterize the uncertain engine over R (uncertain-realfield-generic)

- [ ] B1 `SampledValue<R> = { Float(R), Bool(bool) }`; thread `R` through
      `UncertainNodeContent<R>`, the `SampledFmapFn<R>`/`SampledBindFn<R>` closure traits,
      and the `ConstTree` node content.
- [ ] B2 `DistributionEnum<R>` + `NormalDistributionParams<R>` /
      `UniformDistributionParams<R>` / `BernoulliParams<R>`; sampling delegates to the
      Group A `Distribution<R>` impls.
- [ ] B3 `SequentialSampler` + `sprt_eval` + `GlobalSampleCache` over `R` (cache is generic
      over `R`, per-`R` storage — resolved decision 2); `Sampler<T>` returns `SampledValue<R>`.
- [ ] B4 `Uncertain<R>` / `MaybeUncertain<R>` generic, including arithmetic/comparison/logic
      ops and `lift_to_uncertain` (thresholds over `R`); `ProbabilisticType` impls for
      f64 + f32 + Float106 with the trait kept as the open extension point for future value
      types (resolved decision 1); `is_present` stays `Uncertain<bool>`; `Display` of a
      Float106 sample is a single-decimal pretty-print, two limbs under `Debug` (decision 3).
- [ ] B5 f64 preservation (D7): `UncertainF64`/`MaybeUncertainF64`/`UncertainBool` aliases
      and every existing public method keep their signatures; a bit-identical regression
      battery (distributions, ops, SPRT, `lift`) under fixed seed passes unchanged.
- [ ] B6 Precision-propagation tests: a certain `from_value(Float106)` round-trips with no
      narrowing; deterministic arithmetic on sampled `Float106` values composes at full
      precision; the mandatory honesty doc (D6) is present at the sampling boundary.
- [ ] B7 Downstream smoke: `deep_causality` (`data_uncertain_*` nodes) and the three
      `causal_uncertain_examples` compile and run numerically unchanged (no edits expected).
- [ ] B8 Group gate: format, clippy, full `deep_causality_uncertain` (+ touched
      `deep_causality`) tests both feature configs; prepare the Group B commit message and
      ask the user to commit. Change exit: `MaybeUncertain<R>` is available for Stage 4
      Group C.
