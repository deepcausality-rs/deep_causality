## 1. Phase 1 — Double-double erf/erfc (`deep_causality_num`)

- [x] 1.1 Add `src/float_106/erf.rs` implementing `erf` and `erfc` for `Float106` (all-positive series body for `|x|<1.5`, continued-fraction tail for `|x|>=1.5`; `erfc` computed directly in the tail to avoid cancellation). Inherent methods.
- [x] 1.2 Register the module in `src/float_106/mod.rs` (inherent methods on the already-exported `Float106`, so no new `lib.rs` export)
- [x] 1.3 Add `tests/float_double/double_erf_tests.rs` with mpmath-derived double-double references across body/tail, `erf+erfc=1`, oddness, bounds/monotonicity, tail-without-cancellation, NaN
- [x] 1.4 Register the test file in `tests/float_double/mod.rs` (`tests/BUILD.bazel` uses a `glob`, auto-included)
- [x] 1.5 `cargo test -p deep_causality_num` — 10/10 pass; clippy clean

## 2. Phase 2a — Inverse-CDF transforms (`deep_causality_rand`)

- [x] 2.1 Add `src/utils/inverse_cdf.rs`: `standard_normal_inverse_cdf` at `f64` (Acklam seed + dd Halley, downcast), clamped to finite endpoints
- [x] 2.2 `standard_normal_inverse_cdf_f106`: Acklam f64 seed, Halley refine on `Φ(x)-u=0` against full-precision `u` using `erfc`/`exp`
- [x] 2.3 Add `uniform_inverse_cdf<R: RealField>` (`low + u·(high−low)`) and `bernoulli_inverse_cdf` (`u < p`); Normal affine reuses `Normal::sample_from_zscore`
- [x] 2.4 Export the four transforms from `src/lib.rs`
- [x] 2.5 Add tests: dd quantile references, round-trip quantile↔CDF, monotonicity, antisymmetry, exact Uniform/Bernoulli, Float106 refinement beats f64-widening — 10/10 pass
- [x] 2.6 Register `tests/utils/{mod.rs,inverse_cdf_tests.rs}`, `tests/mod.rs`, and a `utils` `rust_test_suite` in `tests/BUILD.bazel`

## 3. Phase 2b — Sobol sequence + digital shift (`deep_causality_rand`)

- [x] 3.1 Add `src/types/qmc/sobol.rs` with a 16-dim direction-number table (extracted from `scipy.stats.qmc.Sobol`, Joe–Kuo 6.21201) and deterministic `coordinate(index, dim)` / `point(index, &mut [f64])` via the Gray-code XOR formula
- [x] 3.2 Add the seeded digital shift (`new_shifted(dim, seed)`; per-dim shift from `Xoshiro256::from_seed`, XORed into each value)
- [x] 3.3 Register `src/types/qmc/mod.rs`, add `MAX_SOBOL_DIM`, export `SobolSequence`/`MAX_SOBOL_DIM` from `src/lib.rs`; new `RngError::UnsupportedDimension` for the dim cap
- [x] 3.4 Add tests: scipy-reference points (exact), index-determinism, in-`[0,1)`, L2-discrepancy below pseudo-random, exact quadrant stratification, same/different-seed shift behavior
- [x] 3.5 Register `tests/utils/sobol_tests.rs` in `tests/utils/mod.rs` (Bazel `utils` target globs it); added Display test for the new error variant
- [x] 3.6 `Xoshiro256`/`RngCore` untouched; full rand suite (154 tests) green, so existing seeded paths unchanged

## 4. Phase 3 — QmcSampler (`deep_causality_uncertain`)

- [x] 4.1 Extend `SampleCacheKey` to `(usize, u64, SamplerKind)` in `global_cache.rs`; add `SamplerKind {Mc, Qmc}`; thread `SamplerKind::Mc` through `sample_with_index`; change `Sampler::sample` to take `sample_index` (SequentialSampler ignores it)
- [x] 4.2 Add `src/types/sampler/qmc_sampler.rs` with the dimension pre-pass: deterministic DFS assigning each non-`Point` leaf a stable dimension `0..d`
- [x] 4.3 In the pre-pass, reject `BindOp` and branch-divergent `ConditionalOp` (leaf-id-set comparison) with `UncertainError::SamplingError`
- [x] 4.4 Implement `Sampler<T>` for `QmcSampler`: Sobol point per index, each leaf drawn via inverse-CDF on its dimension, reusing the deterministic-op handling from `SequentialSampler`
- [x] 4.5 Seed the digital shift via `QmcSampler::new(root, Some(seed))`; `None` gives the raw sequence
- [x] 4.6 Register the sampler module; export `QmcSampler`/`SamplerKind`; add `Uncertain::root_node()` so the sampler is constructible
- [x] 4.7 Add `tests/types/sampler/qmc_sampler_tests.rs`: dimension count, reproducible/distinct index, Bind + divergent-Conditional rejection, shared-leaf conditional accepted, MC/QMC cache non-collision
- [x] 4.8 Register in `tests/types/sampler/mod.rs` (Bazel `types_sampler` globs it; deps already include ast + rusty-fork); updated existing tests for the new key/signature

## 5. Phase 4 — Opt-in batch API + convergence validation

- [x] 5.1 Added `expected_value_qmc` / `standard_deviation_qmc` (`uncertain_statistics.rs`) and `estimate_probability_qmc` (`uncertain_bool.rs`), each taking a `seed: u64`, routing through `QmcSampler` + `sample_with_index_qmc`
- [x] 5.2 Existing MC signatures and results unchanged; the 216-test suite (incl. pre-existing MC tests) is green
- [x] 5.3 SPRT methods (`to_bool`, `probability_exceeds`, `implicit_conditional`) deliberately given no QMC variant
- [x] 5.4 Convergence test: QMC error < MC error at `N=4096` on Uniform(0,1); reproducibility; non-degenerate QMC `standard_deviation`
- [x] 5.5 Point-only-tree and bool (`estimate_probability_qmc`) paths tested under QMC
- [x] 5.6 Tests registered (same `qmc_sampler_tests.rs` / `types_sampler` target)

## 6. Finalization

- [x] 6.1 `make format && make fix` — clean (also fixed the `uncertain_benchmarks` cache-key)
- [x] 6.2 `make build` (whole repo) green; the 3 changed crates' suites green: num 4303, rand 154, uncertain 216
- [x] 6.3 No `unsafe` added; no new external crate; lint posture intact
- [x] 6.4 Per-crate commit messages prepared and handed to the user to commit (per AGENTS golden rules; committing is the user's action)
