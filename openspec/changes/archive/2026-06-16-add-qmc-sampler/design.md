## Context

`deep_causality_uncertain` evaluates an `Uncertain<R>` by walking an immutable
`ConstTree<UncertainNodeContent>` computation graph. Stochastic leaves draw from a
distribution; the `SequentialSampler` is the only `Sampler<T>` impl today
(`src/types/sampler/sequential_sampler.rs`). Each draw pulls entropy from a
thread-local RNG — a seeded `Xoshiro256` when `seed_sampler` is in effect, else
the OS-entropy thread RNG (`src/types/sampler/sampler_seed.rs`). Reproducibility
of a given sample is provided by a process-global cache keyed `(uncertain_id,
sample_index)` (`src/types/cache/global_cache.rs`); the `sample_index` is a cache
key only — the RNG is a continuously advancing stream, not seeded per index.

Batch estimators reduce many draws into one scalar at the Monte-Carlo rate
`~1/√N`. The CFD presence/collapse gates are low-dimension integrands evaluated
at high sample counts, where a low-discrepancy sequence (`~(log N)^d / N`) is
materially faster. The Normal path currently samples via **Ziggurat rejection**
(`deep_causality_rand/src/utils/ziggurat_sampler.rs`).

## Goals / Non-Goals

**Goals:**
- A `QmcSampler` selectable for the batch estimators, converging faster than MC on
  low-dimension static trees.
- Reproducible, variance-estimable QMC via seeded randomized QMC (digital shift).
- Full `f64` and `Float106` precision parity on the inversion transforms.
- Zero change to default behavior, the public f64 API, or seeded MC bit-output.
- No new external crates; no `unsafe`.

**Non-Goals:**
- QMC for SPRT-based decisions (`to_bool`, `probability_exceeds`,
  `implicit_conditional`) — statistically invalid for correlated points.
- QMC for data-dependent trees (`BindOp`, branch-divergent `ConditionalOp`).
- Replacing or removing `SequentialSampler`.
- Owen scrambling, higher-order digital nets, or adaptive dimension allocation
  (possible later work).
- A single-draw QMC `sample()` — QMC is an ensemble; only batch APIs map onto it.

## Decisions

### D1: Inversion, not an RNG swap
The QMC path transforms each leaf via inverse-CDF on its assigned coordinate
`u ∈ [0,1)`. **Alternative considered:** make Sobol implement `RngCore`/`Rng` and
drop it into `SequentialSampler` (a true drop-in). **Rejected** — Ziggurat and
other rejection/Box–Muller transforms consume a variable, data-dependent number
of `next_u64()` draws, which destroys the low-discrepancy structure. The result
would be QMC-shaped points with no QMC benefit. Inversion is the only transform
that preserves the sequence's equidistribution.

### D2: Sobol over Halton
Sobol has better high-dimension behavior and is the standard choice for
integration. **Alternative:** Halton (simpler radical-inverse, no tables) —
rejected because it degrades earlier as `d` grows and offers no advantage at the
low `d` we target. Cost: a static direction-number table (data, not `unsafe`).

### D3: Seeded digital shift (RQMC) over plain Sobol
Plain Sobol is deterministic, so `standard_deviation` over raw points estimates a
meaningless quantity and yields no error bars. A random digital shift (XOR a
per-run random vector into the Sobol bits), seeded from the `seed_sampler` value,
restores an unbiased variance estimate and keeps runs reproducible. **Alternative:**
Owen scrambling — stronger but more complex; deferred. **Alternative:** no
randomization — rejected (breaks the `standard_deviation` contract).

### D4: Real double-double `erfc` (Path A) over f64-widening (Path B)
`Float106` has `exp`/`ln`/`sqrt` but no `erf`/`erfc`. The Normal inverse-CDF needs
a high-precision CDF `Φ(x) = ½·erfc(−x/√2)` to Halley-refine the f64 quantile seed
to double-double accuracy. **Path A (chosen):** implement `erfc`/`erf` at
`Float106` in `deep_causality_num` (series for small `|x|`, continued-fraction /
rational tail), then refine. **Path B (rejected):** widen the f64 quantile
(`Float106::from_f64(...)`) — reuses the documented f64-boundary pattern but caps
the Normal QMC sample at ~f64 accuracy, breaking the crate's "double-double
entropy end to end" promise. Path A also fills an independently useful num-crate
gap. Uniform and Bernoulli inversion need no transcendental at either precision.

### D5: `sample_index` becomes the Sobol point index
For QMC, the batch estimators feed indices `0..n` directly as Sobol point indices.
This reuses the existing index-threading in `expected_value` /
`standard_deviation` (`src/types/uncertain/uncertain_statistics.rs`) and
`estimate_probability` unchanged. The random-index path behind `Uncertain::sample()`
is not given a QMC variant (see Non-Goals).

### D6: Dimension pre-pass with static-structure validation
A deterministic DFS over the immutable tree assigns each non-`Point` distribution
leaf a stable dimension `0..d`, memoizable by root id. Encountering a `BindOp`, or
a `ConditionalOp` whose branches contain distinct distributions, yields
`Err(UncertainError::SamplingError("QMC requires a static stochastic structure"))`.
This makes the soundness boundary an enforced runtime contract, not documentation.

### D7: Sampler-discriminated cache key
Extend `SampleCacheKey` from `(usize, u64)` to `(usize, u64, SamplerKind)` so MC
and QMC entries for the same `(id, index)` never cross-serve within one process.
**Alternative:** clear the cache on sampler switch — rejected (fragile, hostile to
mixed workloads).

## Risks / Trade-offs

- **High effective dimension erases the benefit** → `(log N)^d / N` exceeds `1/√N`
  for large `d`. Mitigation: target/document low-`d` gates; the dimension pre-pass
  exposes `d`, enabling a future warning or auto-fallback to MC above a threshold.
- **Float106 `erfc` accuracy/edge cases** (tails, cancellation near zero) →
  wrong quantiles. Mitigation: land `erfc` first as an isolated, fully tested unit
  (reference values + round-trip `erf(erfinv)` checks) before any sampler wiring.
- **Silent misuse on dynamic trees** → wrong integrals. Mitigation: D6 turns this
  into a hard error rather than a quiet wrong answer.
- **Cache contamination across samplers** → corrupted results. Mitigation: D7.
- **Digital-shift quality** → biased estimates if the shift is poor. Mitigation:
  seed the shift from the same vetted `Xoshiro256`/`seed_sampler` source; validate
  with convergence and moment tests against analytic integrands.
- **Scope creep from `erfc`** → larger diff than "a sampler". Accepted and
  isolated as phase 1 with its own coverage; it is a prerequisite, not a detour.

## Migration Plan

Purely additive; no migration of existing data or APIs. Rollout follows the four
spec capabilities in dependency order: `double-double-erf` →
(`sobol-sequence`, `inverse-cdf-sampling`) → `qmc-sampler`. Each phase is
independently testable and shippable. Rollback at any phase is removal of the new
modules; default MC behavior and the seeded f64 paths are untouched throughout.

## Open Questions

- Final opt-in ergonomics: distinct `*_qmc` methods vs. a `SamplingMethod`
  argument on the existing batch methods. (Resolved at the `qmc-sampler` spec.)
- Whether to emit a diagnostic when the pre-pass finds `d` above a benefit
  threshold, or silently proceed. Deferred; not blocking.
