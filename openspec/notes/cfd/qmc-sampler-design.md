# QMC Sampler for `deep_causality_uncertain` — Design Note

Status: **proposed, not implemented**. Date: 2026-06-16.

Add a Quasi-Monte Carlo (`QmcSampler`) as an **alternative** `Sampler<T>` for the
batch estimators, where a low-discrepancy sequence converges faster than plain
Monte Carlo (error ~ `(log N)^d / N`, near `1/N` for low effective dimension `d`).
`SequentialSampler` (plain MC) stays the default; nothing changes unless QMC is
explicitly opted into.

## 1. The decisive fact: Ziggurat is incompatible with QMC

The current Normal sampler is **Ziggurat rejection sampling**
(`deep_causality_rand/src/utils/ziggurat_sampler.rs`). It consumes a *variable,
data-dependent* number of `next_u64()` draws per sample. QMC's entire benefit
lives in a *fixed, smooth* map from the unit cube `[0,1)^d` to outputs. Feeding a
Sobol stream through Ziggurat destroys the low-discrepancy structure — you would
get QMC-shaped points with zero QMC benefit.

Therefore the naive "swap the RNG behind `SequentialSampler`" is a **trap**. A
correct QMC sampler is a *different* sampler built on two pillars:

1. **Inverse-CDF (inversion) transforms** for every distribution leaf — the only
   transform that preserves a low-discrepancy sequence's structure.
2. **Explicit, fixed dimension assignment** — each stochastic leaf gets a stable
   coordinate index in the Sobol sequence.

## 2. Locked decisions

| Decision | Choice |
|---|---|
| Soundness boundary | **Enforced** (see §3) |
| Sequence | **Sobol** (direction-number table) |
| Randomization | **Seeded digital shift** (RQMC) |
| Precision | **f64 + Float106** |
| Float106 Normal | **Path A — real double-double `erfc`** in `deep_causality_num` |

## 3. Correctness boundary (enforced, not advisory)

This is a mathematical boundary, not a quality tradeoff.

- **Sound — QMC permitted:** `expected_value(n)`, `standard_deviation(n)`
  (with RQMC, §6), `estimate_probability(n)`. These consume points `0..n` as a
  batch.
- **Unsound — QMC refused:** `to_bool` / `probability_exceeds` /
  `implicit_conditional`. These run **SPRT**
  (`src/algos/hypothesis/sprt_eval.rs`), whose log-likelihood-ratio statistic
  *assumes i.i.d. Bernoulli draws*. QMC points are deliberately negatively
  correlated; the LLR boundaries are invalid for them. SPRT stays MC-only.
- **Unsupported — QMC refused:** trees containing `BindOp`, or a `ConditionalOp`
  whose branches contain *different* distributions. QMC needs a **fixed**
  dimension→leaf mapping across all `N` points; data-dependent control flow has
  no stable layout. The dimension pre-pass detects this and returns
  `Err(UncertainError::SamplingError(...))` rather than wrong numbers.

The random-index path (`Uncertain::sample()` drawing a random `u64` index) has no
QMC meaning — QMC is an *ensemble*. Only the batch APIs map onto it.

## 4. Why these three engine properties drive the design

| Engine property | Source | Consequence |
|---|---|---|
| Ziggurat for Normal | `ziggurat_sampler.rs:33-67` | Must use inverse-CDF in the QMC path |
| `sample_index` is a cache key, not a stream position | `uncertain_sampling.rs:20-31` | For QMC the index *becomes* the Sobol point index `i` |
| `BindOp` / `ConditionalOp` create data-dependent structure | `sequential_sampler.rs:138-142,261-280` | QMC sound only for statically-structured trees |

## 5. Numeric prerequisite from the Float106 choice (Path A)

Float106 has `exp`, `ln`, `sqrt`, `exp_m1` but **no `erf`/`erfc`**. Normal
inverse-CDF at double-double precision needs a high-precision normal CDF
`Φ(x) = ½·erfc(−x/√2)` to Newton/Halley-polish against. The other leaves are clean:

| Leaf | f64 inversion | Float106 inversion |
|---|---|---|
| Uniform | `low + u·(high−low)` | trivial, exact double-double |
| Bernoulli | `u < p` | trivial |
| Normal | Acklam/Wichura PPND16 (≈1e-9) | f64 seed → **Halley step needs `erfc` + `exp`** |

**Path A (locked):** implement a real double-double `erfc` in
`deep_causality_num/src/float_106/erf.rs` (series for small `|x|`, continued
fraction / Cody-style rational for the tail), then Halley-refine the f64 quantile
seed. True end-to-end double-double, consistent with the existing "double-double
entropy end to end" promise on `DistributionEnum<Float106>::sample`. `erfc` is an
independently useful gap-fill and lands first with its own coverage.

## 6. Randomization: seeded digital shift (RQMC)

Plain Sobol is fully deterministic, so `standard_deviation(n)` over raw QMC points
would estimate a meaningless quantity, not sampling error. The fix is a random
**digital shift** (XOR a per-run random vector into the Sobol bits), seeded from
the existing `seed_sampler` value (`src/types/sampler/sampler_seed.rs`). This
keeps runs reproducible, gives genuine error bars, and is standard practice. Owen
scrambling is a possible later upgrade; digital shift is correct and cheap first.

## 7. Module layout (follows the rand/uncertain seam and crate conventions)

```
deep_causality_num/
  src/float_106/erf.rs              # double-double erf / erfc  [Path A; isolated, tested first]

deep_causality_rand/
  src/types/qmc/sobol.rs            # Sobol generator (direction numbers) + seeded digital shift
  src/types/qmc/mod.rs              # SobolSequence: point(i) -> [f64; d] in [0,1)^d
  src/utils/inverse_cdf.rs          # inverse_normal_cdf: f64 (Acklam/Wichura) + Float106 Halley refine

deep_causality_uncertain/
  src/types/sampler/qmc_sampler.rs  # QmcSampler: dimension pre-pass + inversion tree-walk
```

No new external crates, no `unsafe` — Sobol direction numbers are a static table;
everything else is arithmetic. Consistent with the repo lint policy.

## 8. `QmcSampler` contract

1. **Dimension pre-pass** over the immutable `ConstTree` (memoizable by root id):
   deterministic DFS assigns each non-`Point` distribution leaf a stable dimension
   `0..d`. On `BindOp` or a branch-divergent `ConditionalOp` →
   `Err(UncertainError::SamplingError("QMC requires a static stochastic structure"))`.
2. **Point `i`** = `sobol.shifted_point(i)`, with the digital shift derived from
   the `seed_sampler` value.
3. **Tree walk** = the exact op handling of `SequentialSampler::evaluate_node`,
   except each leaf consumes `u[dim]` via inverse-CDF instead of drawing from an
   `Rng`. All arithmetic / comparison / logical / fmap branches reused unchanged.

`d` is small for the target workloads (a presence/collapse gate is a handful of
distributions), so `(log N)^d / N` stays near `1/N` — the intended regime.

## 9. Cache interaction (must-fix)

The global cache is keyed `(usize, u64)` with no sampler identity
(`src/types/cache/global_cache.rs:11`). If MC and QMC runs share a process, QMC
entries would be served to MC callers and vice-versa — silent corruption. Extend
the key to `(usize, u64, SamplerKind)` so both can coexist.

## 10. Selection API

Smallest viable surface: per-call opt-in on the batch methods (QMC variants of
`expected_value` / `standard_deviation` / `estimate_probability`, or a
`SamplingMethod` argument), rather than global state. Keeps `seed_sampler`
semantics intact and avoids a hidden mode. SPRT methods are deliberately **not**
given a QMC variant.

## 11. Phasing (each phase independently testable, per AGENTS §5)

1. `erfc`/`erf` at Float106 in `deep_causality_num` + tests vs known values.
2. `inverse_normal_cdf` (f64 Acklam + Float106 Halley refine) and Sobol generator
   with seeded digital shift, in `deep_causality_rand` + tests
   (discrepancy / known-quantile checks).
3. `QmcSampler` + dimension pre-pass + static-structure rejection + cache-key
   discriminant, in `deep_causality_uncertain`.
4. Opt-in batch API + convergence test showing QMC error < MC at equal `N` on a
   low-dimension integrand.

## 12. Honest cost summary

Not a drop-in. Real work: double-double `erfc`; inverse-normal-CDF (f64 + Float106
refine); Sobol + digital shift; the dimension pre-pass with static-structure
validation; the cache-key discriminant. Everything downstream of the leaves (the
whole `evaluate_node` op set) is reused verbatim.
