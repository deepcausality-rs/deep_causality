## Why

`Uncertain<R>`'s batch estimators (`expected_value`, `standard_deviation`,
`estimate_probability`) converge at the Monte-Carlo rate `~1/√N`, so halving the
error costs 4× the samples. For the CFD presence/collapse gates — low-dimension
integrands evaluated thousands of times — a Quasi-Monte Carlo sampler converges
at `~(log N)^d / N` (near `1/N` for low effective dimension `d`), delivering the
same accuracy in far fewer samples. This directly serves the "minutes, not hours"
validation north-star. The full design rationale is in
`openspec/notes/cfd/qmc-sampler-design.md`.

## What Changes

- Add a `QmcSampler` as an **alternative** `Sampler<T>` implementation; the
  existing `SequentialSampler` (plain MC) stays the default. No behavior changes
  unless QMC is explicitly opted into.
- Build QMC on a **Sobol** low-discrepancy sequence with a **seeded digital
  shift** (randomized QMC), so `standard_deviation` stays meaningful and runs stay
  reproducible under `seed_sampler`.
- Replace the entropy-driven draw with **inverse-CDF (inversion) transforms** in
  the QMC path. Ziggurat rejection sampling consumes a variable number of draws
  and destroys the low-discrepancy structure, so it cannot be reused.
- Add a double-double **`erfc`/`erf`** to `deep_causality_num` so the `Float106`
  Normal inverse-CDF can be Halley-refined to full double-double precision.
- Enforce a **soundness boundary**: QMC backs only the batch estimators on
  statically-structured trees. It is refused (returns an error) for SPRT-based
  `to_bool` / `probability_exceeds` / `implicit_conditional`, and for trees
  containing `BindOp` or a branch-divergent `ConditionalOp`.
- Extend the global sample-cache key with a **sampler discriminant** so MC and
  QMC samples never cross-serve within one process.

## Capabilities

### New Capabilities
- `double-double-erf`: `erf` / `erfc` for `Float106` in `deep_causality_num`,
  accurate to double-double precision, enabling Normal-quantile refinement.
- `sobol-sequence`: a Sobol low-discrepancy sequence generator with a seeded
  digital shift (RQMC) in `deep_causality_rand`.
- `inverse-cdf-sampling`: inverse-CDF transforms for the Normal, Uniform, and
  Bernoulli distributions in `deep_causality_rand`, mapping `u ∈ [0,1)` to a draw
  at `f64` and `Float106` precision.
- `qmc-sampler`: the `QmcSampler` in `deep_causality_uncertain` — dimension
  pre-pass with static-structure validation, inversion-based tree walk,
  sampler-discriminated cache key, and the opt-in batch-estimator surface.

### Modified Capabilities
<!-- None. The QmcSampler is additive: it is a new Sampler impl alongside
     SequentialSampler. The cache-key extension is an internal implementation
     detail, not a spec-level requirement change to uncertain-realfield-generic
     or rand-realfield-sampling, whose existing requirements remain satisfied. -->

## Impact

- **`deep_causality_num`**: new `src/float_106/erf.rs` (`erf`/`erfc`). Additive.
- **`deep_causality_rand`**: new `src/types/qmc/` (Sobol + digital shift) and
  `src/utils/inverse_cdf.rs` (inversion transforms). Additive; existing RNG,
  distributions, and seeded `f64`/`f32` paths unchanged and bit-identical.
- **`deep_causality_uncertain`**: new `src/types/sampler/qmc_sampler.rs`; the
  global cache key gains a `SamplerKind` discriminant; new opt-in QMC variants of
  the batch estimators. SPRT methods are deliberately not given a QMC variant.
- **No new external crates, no `unsafe`** — Sobol direction numbers are a static
  table; everything else is arithmetic. Consistent with the repo lint policy.
- **Dependencies**: unchanged crate tiers (num → rand → uncertain).
