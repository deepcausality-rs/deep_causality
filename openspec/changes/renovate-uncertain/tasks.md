<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## 1. Baseline

- [x] 1.1 Record the pre-change baseline in one place: `cargo test -p deep_causality_uncertain` count, `bazel test //...` count, the six `dyn` sites, the 72 `SampledValue` variant arms, the 24 `rusty_fork_test!` invocations, and the src line count. Every later claim of "smaller" or "fewer" is checked against this row, not asserted.
- [x] 1.2 Characterise the existing flake: run the suite 50 times, record which tests fail and at what rate. Verify: `uncertain_maybe_f106_tests::test_lift_to_uncertain_success` is among them, and no other flake is hiding behind it.
- [x] 1.3 Record a seeded golden vector under the **current** implementation — 32 draws from a two-leaf tree at a fixed seed — so the change's effect on recorded sequences is a measured diff rather than an assumption.

## 2. The session and index-addressed draws

- [x] 2.1 Write the seed-mixing kernel `mix(seed, index, ordinal) -> u64` with literal provenance: state the construction and its source, and pin corner rows (zero seed, zero index, zero ordinal, maximal values, and the three arguments pairwise swapped) against literal expected values. Verify: swapping any two arguments changes the result.
- [x] 2.2 Add `SampleSession` with `seeded(seed)`, `qmc(seed)` and `from_entropy()` constructors, holding the seed, the sample counter and the mode. It carries no scalar parameter and no Sobol sequence — see design D10. No global is touched from it except `from_entropy`, which draws one seed.
- [x] 2.3 Add the ordinal pre-pass: one deterministic traversal assigning each distinct node an ordinal, deduping by node identity within that traversal. Verify by test that the pointer never reaches the generator — build two structurally identical trees separately, draw both at the same index under equal seeds, and assert the draws are equal.
- [x] 2.4 Route `SequentialSampler` leaf draws through `Xoshiro256::from_seed(mix(..))` instead of the ambient RNG, keeping the existing per-call memo that makes `x + x` one draw. Verify: `x + x == 2 * x` at every index.
- [x] 2.5 Add `Uncertain::sample_at(&self, &mut SampleSession, index)` beside the existing `sample_with_index`, with the cache still present but unused by the new path. Verify: a seeded session reproduces its own vector across two processes.
- [x] 2.6 Verify the QMC path agrees: a QMC session at a given index yields the same value as `QmcSampler` does today for that Sobol point, since dimension assignment already worked this way.

## 3. Removing the globals

- [x] 3.1 Move every draw call site onto the session, then delete `types/cache/` (`GlobalSampleCache`, `with_global_cache`, `SamplerKind`, `SampleCacheKey`) and `seed_sampler` / `clear_sampler_seed` / `SAMPLER_SEED`. **Ask before deleting** — Golden Rule 2.
- [x] 3.2 Rewrite the cache-and-seed call sites against a session: 38 originally, plus the 24 seeds task 3.4 added, across ten test files.
- [x] 3.3 Remove the last three `rusty_fork_test!` blocks and drop `rusty-fork` from the dev-dependencies. **Already cut from 24 blocks in 20 files to 3 in 3.** The rule the measurement established: the fork is needed only where a test reaches into the global sample cache directly — `cache_tests`, `uncertain_sampling_tests` and `uncertain_statistics_tests` all call `with_global_cache` to clear it and insert entries pinning what a statistic reads. All three go with the cache in 3.1, and with them three of the five `cfg(not(miri))` gates. The other two, in `types/sampler/mod.rs`, cite Miri's soft-float drift and are unrelated — leave them. Verified at 0 failures in 500 runs at `--test-threads=16`, 80 at 1 and 80 at the default.
- [x] 3.4 Fix the flaky presence-gate test by seeding it. The assertion stays exact — no wider tolerance, no larger budget. Verify: 100 consecutive runs agree.
- [x] 3.5 Verify no mutable global remains: `src/` contains no `static mut`, no `OnceLock`, no `thread_local!`, and `NEXT_UNCERTAIN_ID` is gone or justified in place.
- [x] 3.6 Group close-out: `cargo test -p deep_causality_uncertain`, `make format && make fix`, and a commit message.

## 3B. Defects the verification found (resolved before the retrofit)

Nine verification agents measured group 4's assumptions against the tree before any of it was
written. Three assumptions were false and five pre-existing defects surfaced, all of them in crates
this crate draws through. They are fixed first, so that group 4's acceptance test asserts against a
correct stack rather than around a broken one.

- [x] 3B.1 Replace the naive left-to-right accumulation in `deep_causality_stats` with pairwise summation: `mean`, `dispersion` (both the direct and the rescaled pass), and `MeanAccumulator`. Measured at `BFloat16`, seeded, N(100, 5): the mean reads 114.5 at n = 256, 32.75 at n = 1000 and 3.28 at n = 10000 against a true 100, and N(0, 1) — the case every existing test uses — gives a standard deviation of 0.51 against a true 1.0. The sum stalls once it passes 256 because an increment rounds away at that magnitude, and it stays finite, so the module's existing saturation fallback never fires.

      **This is the estimator's defect and the discriminator is what settles it.** `BFloat16` keeps `f32`'s whole exponent range and gives two significant decimal digits, and `deep_causality_num` rounds every individual operation correctly. What the format promises is per-operation; approximating a sum of `n` terms is the estimator's job. The mean of that sample is 100.0 to well inside the scalar's own resolution of ±0.5 — so the answer is representable, and a 67% error is 170× the resolution. Pairwise summation, in the same type and over the same correctly-rounded operations, returns it. A limit of the format would survive any arrangement; this one does not.

      Verify: at `BFloat16` the estimate lands within the scalar's own resolution at every n up to 100 000 for a distribution whose mean is far from zero — N(0, 1) cannot detect this defect at all, which is how it survived; `Float106` values unmoved; `f64` and `f32` move by at most 2 ULP and are re-recorded.

      **Done.** N(100, 5) at `BFloat16` now reads 100.0000 at n = 256, 1000 and 10 000 where it read 114.5, 32.75 and 3.28; N(0, 1) reads a standard deviation of 1.0078 where it read 0.5117. No existing assertion needed re-recording — the 1–2 ULP moves at `f64` and `f32` were inside every shipped tolerance. The same fold appeared in three more reductions of the same crate and they were fixed with it: `entropy`'s normalising sum, its rescaled pass and its surprisal sum, and `log_sum_exp`. One `PairwiseSum` type and one `pairwise_sum` helper serve all eight sites. This does not leave the crate clean of the defect — see 3B.10 for the four reductions it does not reach.
- [x] 3B.2 Make one summation serve both entry points. The slice `mean` and `MeanAccumulator` must agree bit-for-bit, which is what `MeanAccumulator`'s docstring already promises; a midpoint-split tree and a carry tree do not agree for a non-power-of-two count. Verify: the two agree on the same sequence at every scalar, asserted rather than described. **Done** — the carry arrangement makes which partial sum lands in which slot a function of the count alone, so agreement holds by construction; pinned at a thousand `BFloat16` observations, a count that is neither small nor a power of two, where a differing association is visible in the result.
- [x] 3B.3 Fix the guarantee `moments.rs` documents about itself: `mean([3e38; 1000])` at `BFloat16` returns 7.68e37 where the module docstring promises 3.004e38. Verify with the literal from the docstring. **Done** — 3.004e38. The direct sum saturates at the second observation, so the whole answer came from the rescaled pass, which was summing a thousand ones left to right and stalling at 256.
- [x] 3B.4 Add the scalar-generic standard-normal quantile `deep_causality_stats` does not have. The QMC path draws by inverse CDF and `standard_normal_inverse_cdf` exists only at `f64` and `Float106`; `erfc`, which the Halley refinement needs, is an inherent method on `Float106` and is on neither `Float` nor `Real`, so a generic quantile cannot be refined at `R`. Keep the engine at `Float106` and reassemble the limbs into `R`. Not a table keyed by type. Verify: bit-identical to today's `f64` and to today's `Float106` value over the real Sobol lattice and the clamp boundaries, asserted in the stats suite. **Done** — `standard_normal_inverse_cdf_at<R>(u: f64) -> R`, bit-identical at both over the uniform sweep, the `k/2³²` lattice at both ends, both clamp boundaries and their neighbours, the denormal floor and the endpoints. The input stays `f64` because it is a coordinate rather than a value, which also dissolves the tail-saturation the verification found: nothing rounds the coordinate into a narrow scalar any more.
- [x] 3B.5 Record the two narrow-scalar facts that are representation limits rather than defects, each with the measurement that settles it, at the constructor that carries them: `Bernoulli::new(p)` is exactly certain for `p ≥ 0.998` at `BFloat16` because the scalar has no value between 0.9921875 and 1.0, and an observation count is not exact past 256 there (`from_usize(999)` is 1000, `from_usize(10000)` is 9984). Verify: both pinned by test, and the count's effect shown to sit below one ULP of the answer rather than asserted to be harmless. **Done**, and both figures from the verification were wrong. The largest `BFloat16` below one is `0.99609375`, not `0.9921875`, so the certainty boundary is `0.998047`. There is no counterpart at zero at all — the format keeps `f32`'s subnormals, so a small probability stays small; the verification's `0.0078` lower bound does not exist. The count test asserts the ordering `divisor error < epsilon` rather than claiming harmlessness.
- [x] 3B.6 `Rng::random_range` asserts where `Uniform::new` returns `UniformDistributionError::EmptyRange`, so a range the caller's scalar collapses aborts instead of erroring — `random_range(1000.0..1001.0)` panics at `BFloat16`. Establish which of the two contracts the crate means, and make the code and the documentation say the same thing. Verify: a `BFloat16` regression test pins whichever it is. **Done, and it is not a defect.** The two contracts are deliberate and different: `random_range` is an infallible convenience over a range the caller wrote, and `Uniform::new` is a constructor that can be handed bounds computed at runtime. `#[track_caller]` already puts the panic at the call site. What was missing was the `# Panics` section saying so, and the note that a range with distinct endpoints can still be empty at a narrow scalar. The test pins both refusals and a third case one power of two lower where the range is not empty, so the refusal is about those values and not about the type.
- [x] 3B.7 `tests/types/sampled_value/` has no `rust_test_suite` in `deep_causality_uncertain/BUILD.bazel`, so its 12 tests run under cargo and never under `bazel test //...` — 271 against 283. Close it, and add a cargo-versus-Bazel count check to the group close-out, because every Bazel glob is one flat non-recursive directory and any directory the rewrite adds is invisible the same way. **Done** — 33 test targets to 34, and the 12 tests pass under `bazel test` for the first time. A workspace-wide sweep for the same hole in other crates returned 86 candidates and all but this one were false positives: the other crates glob recursively, which this one does not.
- [x] 3B.8 Correct the task record: 6.1, 6.2 and 6.3 are ticked and are not done. `ProbabilisticType` is still spelled at 21 `deep_causality_cfd` lines across 7 files and 4 `deep_causality` lines; the bound cannot be dropped before group 4 removes it, so that work lands with group 4 rather than before it.
- [x] 3B.9 Correct three documented claims the measurements falsified: `deep_causality_stats`'s `lib.rs` says no public signature names a concrete float while four quantile transforms do; `precision.rs` and the `BFloat16` moments suite record a thousand-term reduction as meaningless at that width, which was true of the arrangement and not of the type — `BF16.reduction` is a tolerance again at `5e-2`, and the two cases the claim excluded now run; and `deep_causality_num`'s `lift` module says "three shipped scalars" where there are four.
- [x] 3B.10 The same accumulation defect in `deep_causality_stats`'s multi-accumulator reductions — `correlation`, `covariance`, `ridge` and `logistic` — which run several running sums at once rather than one. Witness: `pearson` over a perfectly correlated pair returned `1.0078` at n = 100 and `1.0156` at n = 1000 at `BFloat16`, a correlation coefficient above one, which is not merely inaccurate but outside the quantity's range.

      **Done.** Fourteen accumulation sites across the crate now sum through `PairwiseSum`. `pearson` returns exactly 1.0 at both counts. `column_means` read 32.75 against a true 104.995 and now reads 105.5, inside the scalar's resolution. `covariance_matrix` read 67072 for a variance of 83417 and now reads 83456, inside one spacing there. The `p × p` accumulators cost `usize::BITS` partial sums per cell while the pass runs; the row-major traversal is unchanged, so the data is still read once.

      Measured note on the fixture: the first covariance witness was miscalibrated — deviations of 0.01 around a mean of 100 sit below `BFloat16`'s spacing of 0.5 there, so it measured the format's resolution rather than the arrangement of the sums. Rebuilt on a ramp whose deviations are of order `n/4`.

- [x] 3B.11 Give the fixes a suite that would catch their regression, across the range and not at one witness. New `tests/types/pairwise_sum_tests.rs` covers the accumulator through `utils_tests::accumulation` — corner rows A, B, D, F, G, H, I, J and K, the counts either side of every power of two to 1025, the occupancy-equals-count invariant at every count to 300, order independence, sign and cancellation invariants, and the long-reduction cases at all four scalars. `correlation` gains the range bound `|r| ≤ 1` at five counts, oddness in each argument, and exactness at `±1`; `covariance` gains symmetry, agreement with `mean` and `variance` over the same columns, column-negation, and a closed-form value at `BFloat16`; `ridge` and `logistic` gain long-design cases.

      Calibrated by the scripted defect audit rather than asserted: degenerating the carry to one slot — which is exactly a left-to-right fold — fails **ten** tests across four files, and restoring it passes all 659. Two findings came out of that audit rather than out of review. The covariance cases as first written asserted *agreement* between the matrix and `mean`/`variance`, which both sides lose identically, so they passed under the defect until a closed-form value was added; and the tolerance in that value case was double-scaled, because the file's own `assert_close` already multiplies by `|expected|`. Recorded because a test that cannot fail is worse than no test.

      `ridge` and `logistic` are stated as what they are: they do not run at `BFloat16`, because a solve that squares the condition number cannot carry two decimal digits however the sums are arranged, so at `f32` and wider they pass under both arrangements. That is written at each of them rather than left for a reader to discover.

- [x] 3B.12 One finding from the new suite, in the accumulator itself: `total()` folded from a `T::zero()` seed, and `(+0) + (−0)` is `+0`, so a sum that is genuinely negative zero came back positive. It now folds from the first occupied slot, and returns zero only when nothing was added.

## 4. Precision as a parameter

- [ ] 4.1 Collapse `DistributionEnum`'s three sample impls into one `impl<T: RandScalar> DistributionEnum<T>`, with the Bernoulli branch serving the Boolean case. Verify: the `f64` and `Float106` bodies were identical, so the collapse changes no value — assert equality against the recorded baseline.
- [ ] 4.2 Make the node representation generic: `Node<R>` and `Sample<R> { Real(R), Bool(bool) }`, with one `Distribution(DistributionEnum<R>)` arm replacing the three per-type arms.
- [ ] 4.3 Make both samplers and the SPRT evaluator generic in `R`. Verify: the 72 `SampledValue` variant arms are gone; report the new count.
- [ ] 4.4 Drop `Uncertain<T: ProbabilisticType>` to `Uncertain<T>` with bounds on impl blocks, then remove `ProbabilisticType`, `IntoSampledValue`, `FromSampledValue`, `SampledValue` and `UncertainReal`. **Ask before deleting the files.**
- [ ] 4.5 Verify zero concrete scalars: no implementation, constant or match arm in `src/` names `f32`, `f64`, `Float106` or `BFloat16` outside a display boundary.
- [ ] 4.6 Add the acceptance test: build, sample, and take statistics of an `Uncertain<R>` at `BFloat16` and at `f32` — scalars the crate names nowhere. This is the test that decides whether the retrofit succeeded; a count of deleted files does not.

## 5. Probabilities in the caller's scalar

- [ ] 5.1 Change the five comparison thresholds (`greater_than`, `less_than`, `equals`, `approx_eq`, `within_range`) and the `ComparisonOp` node threshold to `R`.
- [ ] 5.2 Change `bernoulli(p: R)`, `probability_exceeds`, `estimate_probability -> R`, and the SPRT threshold, confidence and epsilon to `R`. The probability estimate is `lift_count::<R>(hits) / lift_count::<R>(n)`.
- [ ] 5.3 Replace the `FunctionOpF64` and `FunctionOpBool` arms' `Arc<dyn Fn>` with a function type parameter, following `deep_causality_calculus`'s `Euler` / `Rk4` / `Diff`.
- [ ] 5.4 Document at the Bernoulli constructor that the parameter is honoured to a multiple of `2^-64` whatever scalar states it, and that this is the representation's bound rather than the scalar's.
- [ ] 5.5 Reimplement `MaybeUncertain<R>` as one impl block over the generic tree, replacing the four per-type files, with the presence and value channels sampled at one index. Keep the type name and the `MaybeUncertainF64` / `MaybeUncertainF106` aliases.
- [ ] 5.6 Group close-out: suite green, `make format && make fix`, commit message.

## 6. Consumers

- [ ] 6.1 Drop `+ ProbabilisticType` from the sixteen `deep_causality_cfd` bound occurrences across 8 files (measured; the earlier figure of ten came from a truncated listing). Verify: the uncertain march compiles at `f32`, which it could not before — that compile is the test.
- [ ] 6.2 Update the two `deep_causality_quantum` files that name the removed bound.
- [ ] 6.3 Verify the 45 alias call sites in `deep_causality` need no edit, and that the four aliases still resolve.
- [ ] 6.4 Update the three example crates that construct uncertain values; check `cargo run` for each, since examples are verified by running rather than by unit tests.

## 7. The ensemble carrier

- [x] 7.1 Add `Collectable<F: HKT>` to `deep_causality_haft` beside `Foldable`, with the rationale recorded at the trait: `Foldable` consumes, `Pure` builds one element, `Semigroupal` pairs without extending, so nothing existing can build a rank-1 container from a sequence.
- [x] 7.2 Implement it for `DenseVectorWitness` in `deep_causality_linear` and for `CausalTensorWitness` (rank 1) in `deep_causality_tensor`. Verify the empty case returns the empty container for both.
- [ ] 7.3 Add `Uncertain::materialize<W>(&self, &mut SampleSession, n)`. Verify: instantiating at both witnesses from one signature yields the same `n` values in the same order.
- [ ] 7.4 Add `deep_causality_haft` to `deep_causality_uncertain`'s dependencies and verify the crate names neither `deep_causality_linear` nor `deep_causality_tensor`, and that its tier is unchanged.
- [ ] 7.5 Write the correlation test with **exact values**, not a count: materialise two quantities from one session, zip them through `ZipDenseVectorWitness`, and assert the i-th pair is the i-th draw of each. A count cannot tell a correct diagonal from an off-by-one one.
- [ ] 7.6 Document the composition surface: the zip witnesses and `sequence_zip` for correlated draws, the cartesian `Traversable::sequence` named as the hazard with its measured 50⁴ result, and the memory arithmetic that says not to materialise a field.

## 8. The Arrow instance

- [ ] 8.1 Implement `Arrow for Uncertain<R>` with `In = SampleIndex`, `Out = Result<R, UncertainError>`. Verify: running at one index twice agrees, with nothing stored between.
- [ ] 8.2 Verify composition is static — compose with a downstream arrow and confirm the composite type is a concrete generic struct with no trait object in it.
- [ ] 8.3 Remove the `PureOp`, `FmapOp`, `ApplyOp` and `BindOp` arms, the `SampledFmapFn` / `SampledBindFn` traits, and the QMC and sequential sampler branches that guard them. **Ask before deleting.** Verify the QMC guard against data-dependent structure still rejects what it rejected before, through whatever arm now represents it.
- [ ] 8.4 Verify zero `dyn` in `deep_causality_uncertain/src`.

## 9. Close-out

- [ ] 9.1 Update the crate README and the module documentation: the session replaces the seed functions, the graph is Arrow-shaped, the ensemble is a witnessed container, and precision is a parameter. Document what the code does, not how it got here.
- [ ] 9.2 Re-measure every figure from task 1.1 and record the deltas — src lines, test count, `dyn` sites, variant arms, fork-isolated tests.
- [ ] 9.3 Run the scripted defect audit: for each removed guarantee, reintroduce the defect it guarded against and confirm a test fails. Cover at least the pointer-in-seed defect, the shared-leaf double-draw, the cartesian-instead-of-diagonal traversal, and the unseeded gate.
- [ ] 9.4 Run `cargo mutants` over the seed-mixing kernel and the ordinal pre-pass, the two places where a wrong constant yields a plausible number rather than a crash. Record survivors with the measurement that settles them, or fix the gap.
- [ ] 9.5 `bazel test //...` green, `make format && make fix` clean, every new test file registered in its `mod.rs` and in `BUILD.bazel`.
- [ ] 9.6 Update `openspec/notes/unified_math/hkt_uncertain.md`: stages 2, 3 and 4 are done, B1/B3/B4/B5/B9 are closed, and §B5's leaf-address sketch is corrected to the ordinal scheme.
- [ ] 9.7 Final commit message, and hand the change over for archive.
