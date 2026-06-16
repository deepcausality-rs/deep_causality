## 1. Phase 1 — Double-double erf/erfc (`deep_causality_num`)

- [ ] 1.1 Add `src/float_106/erf.rs` implementing `erf` and `erfc` for `Float106` (series for small `|x|`, continued-fraction/rational tail; `erfc` computed directly for large `x` to avoid cancellation)
- [ ] 1.2 Register the module in its parent `mod` and export `erf`/`erfc` from `src/lib.rs`
- [ ] 1.3 Add `tests/float_106/erf_tests.rs` covering reference values across small/moderate/tail regions, `erf(x)+erfc(x)=1`, oddness `erf(-x)=-erf(x)`, and tail-without-cancellation
- [ ] 1.4 Register the test file in `tests/.../mod.rs` and `tests/BUILD.bazel`
- [ ] 1.5 `cargo test -p deep_causality_num`; confirm double-double accuracy on all cases

## 2. Phase 2a — Inverse-CDF transforms (`deep_causality_rand`)

- [ ] 2.1 Add `src/utils/inverse_cdf.rs`: `inverse_normal_cdf` at `f64` (Acklam/Wichura), monotone, finite endpoints
- [ ] 2.2 Extend it to `Float106`: f64 quantile as initial guess, Halley/Newton refine on `Φ(x)-u=0` using `erfc`/`exp`
- [ ] 2.3 Add Uniform (`low + u·(high−low)`) and Bernoulli (`u < p`) inversions for `f64` and `Float106`, sharing parameter types with the forward `sample` surface
- [ ] 2.4 Export the transforms from `src/lib.rs`
- [ ] 2.5 Add tests: round-trip quantile↔CDF at both precisions, monotonicity, one-uniform-in/one-out, exact Uniform/Bernoulli, Float106 refinement beats f64-widening
- [ ] 2.6 Register test files in `mod.rs` and `tests/BUILD.bazel`

## 3. Phase 2b — Sobol sequence + digital shift (`deep_causality_rand`)

- [ ] 3.1 Add `src/types/qmc/sobol.rs` with the static direction-number table and a deterministic `point(i, d) -> [f64; d]` in `[0,1)^d`
- [ ] 3.2 Add the seeded digital shift (per-run shift XORed into the bits; seed → shift deterministic) in `src/types/qmc/mod.rs`
- [ ] 3.3 Add the `LowDiscrepancySequence` surface and export from `src/lib.rs`
- [ ] 3.4 Add tests: index-determinism (any call order), coordinates in `[0,1)`, discrepancy below pseudo-random, same-seed reproducibility, different-seed independence
- [ ] 3.5 Register test files in `mod.rs` and `tests/BUILD.bazel`
- [ ] 3.6 Confirm `Xoshiro256`/`RngCore` and existing seeded `f64`/`f32` paths are untouched and bit-identical

## 4. Phase 3 — QmcSampler (`deep_causality_uncertain`)

- [ ] 4.1 Extend `SampleCacheKey` to `(usize, u64, SamplerKind)` in `src/types/cache/global_cache.rs`; thread `SamplerKind` through `with_global_cache`/`get_or_compute` callers
- [ ] 4.2 Add `src/types/sampler/qmc_sampler.rs` with the dimension pre-pass: deterministic DFS assigning each non-`Point` leaf a stable dimension `0..d`, memoizable by root id
- [ ] 4.3 In the pre-pass, reject `BindOp` and branch-divergent `ConditionalOp` with `UncertainError::SamplingError`
- [ ] 4.4 Implement `Sampler<T>` for `QmcSampler`: build the shifted Sobol point for the index, walk the tree drawing each leaf via inverse-CDF on its dimension, reusing the deterministic-op handling from `SequentialSampler`
- [ ] 4.5 Seed the digital shift from the active `seed_sampler` value, else OS entropy
- [ ] 4.6 Register the sampler module and export `QmcSampler`/`SamplerKind` from `src/lib.rs`
- [ ] 4.7 Add tests: stable per-leaf coordinate across indices, exactly `d` dimensions used, dynamic-structure rejection, seeded reproducibility, MC/QMC cache non-collision
- [ ] 4.8 Register test files in `mod.rs` and `tests/BUILD.bazel`

## 5. Phase 4 — Opt-in batch API + convergence validation

- [ ] 5.1 Add the QMC opt-in on `expected_value`, `standard_deviation`, `estimate_probability` (dedicated methods or a `SamplingMethod` argument), routing through `QmcSampler`
- [ ] 5.2 Keep existing MC signatures and results unchanged for non-opt-in callers (default `SequentialSampler`)
- [ ] 5.3 Confirm SPRT methods (`to_bool`, `probability_exceeds`, `implicit_conditional`) expose no QMC variant
- [ ] 5.4 Add convergence tests: QMC error ≤ MC error at equal `N` on a low-dimension analytic integrand, with faster rate as `N` grows; non-degenerate QMC `standard_deviation`
- [ ] 5.5 Add tests for the certain/arithmetic-only and bool paths under QMC
- [ ] 5.6 Register test files in `mod.rs` and `tests/BUILD.bazel`

## 6. Finalization

- [ ] 6.1 `make format && make fix` (≥3 crates changed)
- [ ] 6.2 `make build` and `make test` across the repo; confirm no regressions
- [ ] 6.3 Verify `unsafe_code = "forbid"` holds and no new external crate was added
- [ ] 6.4 Prepare a commit message and ask the user to commit (per AGENTS golden rules)
