<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## 1. Baseline

- [ ] 1.1 Record the pre-change baseline in one place: `cargo test -p deep_causality_uncertain` count, `bazel test //...` count, the six `dyn` sites, the 72 `SampledValue` variant arms, the 24 `rusty_fork_test!` invocations, and the src line count. Every later claim of "smaller" or "fewer" is checked against this row, not asserted.
- [ ] 1.2 Characterise the existing flake: run the suite 50 times, record which tests fail and at what rate. Verify: `uncertain_maybe_f106_tests::test_lift_to_uncertain_success` is among them, and no other flake is hiding behind it.
- [ ] 1.3 Record a seeded golden vector under the **current** implementation — 32 draws from a two-leaf tree at a fixed seed — so the change's effect on recorded sequences is a measured diff rather than an assumption.

## 2. The session and index-addressed draws

- [ ] 2.1 Write the seed-mixing kernel `mix(seed, index, ordinal) -> u64` with literal provenance: state the construction and its source, and pin corner rows (zero seed, zero index, zero ordinal, maximal values, and the three arguments pairwise swapped) against literal expected values. Verify: swapping any two arguments changes the result.
- [ ] 2.2 Add `SampleSession<R>` with `seeded(seed)` and `qmc(seed)` constructors, holding the seed, the sample counter and the optional Sobol sequence. No global is touched from it.
- [ ] 2.3 Add the ordinal pre-pass: one deterministic traversal assigning each distinct node an ordinal, deduping by node identity within that traversal. Verify by test that the pointer never reaches the generator — build two structurally identical trees separately, draw both at the same index under equal seeds, and assert the draws are equal.
- [ ] 2.4 Route `SequentialSampler` leaf draws through `Xoshiro256::from_seed(mix(..))` instead of the ambient RNG, keeping the existing per-call memo that makes `x + x` one draw. Verify: `x + x == 2 * x` at every index.
- [ ] 2.5 Add `Uncertain::sample_at(&self, &mut SampleSession<R>, index)` beside the existing `sample_with_index`, with the cache still present but unused by the new path. Verify: a seeded session reproduces its own vector across two processes.
- [ ] 2.6 Verify the QMC path agrees: a QMC session at a given index yields the same value as `QmcSampler` does today for that Sobol point, since dimension assignment already worked this way.

## 3. Removing the globals

- [ ] 3.1 Move every draw call site onto the session, then delete `types/cache/` (`GlobalSampleCache`, `with_global_cache`, `SamplerKind`, `SampleCacheKey`) and `seed_sampler` / `clear_sampler_seed` / `SAMPLER_SEED`. **Ask before deleting** — Golden Rule 2.
- [ ] 3.2 Rewrite the 38 cache-and-seed call sites across the six test files against a session.
- [ ] 3.3 Convert the 24 `rusty_fork_test!` invocations to ordinary tests and drop `rusty-fork` from the dev-dependencies. Verify: the suite passes with `--test-threads` at the default and at 1.
- [ ] 3.4 Fix the flaky presence-gate test by seeding it. The assertion stays exact — no wider tolerance, no larger budget. Verify: 100 consecutive runs agree.
- [ ] 3.5 Verify no mutable global remains: `src/` contains no `static mut`, no `OnceLock`, no `thread_local!`, and `NEXT_UNCERTAIN_ID` is gone or justified in place.
- [ ] 3.6 Group close-out: `cargo test -p deep_causality_uncertain`, `make format && make fix`, and a commit message.

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

- [ ] 6.1 Drop `+ ProbabilisticType` from the ten `deep_causality_cfd` bounds. Verify: the uncertain march compiles at `f32`, which it could not before — that compile is the test.
- [ ] 6.2 Update the two `deep_causality_quantum` files that name the removed bound.
- [ ] 6.3 Verify the 45 alias call sites in `deep_causality` need no edit, and that the four aliases still resolve.
- [ ] 6.4 Update the three example crates that construct uncertain values; check `cargo run` for each, since examples are verified by running rather than by unit tests.

## 7. The ensemble carrier

- [ ] 7.1 Add `Collectable<F: HKT>` to `deep_causality_haft` beside `Foldable`, with the rationale recorded at the trait: `Foldable` consumes, `Pure` builds one element, `Semigroupal` pairs without extending, so nothing existing can build a rank-1 container from a sequence.
- [ ] 7.2 Implement it for `DenseVectorWitness` in `deep_causality_linear` and for `CausalTensorWitness` (rank 1) in `deep_causality_tensor`. Verify the empty case returns the empty container for both.
- [ ] 7.3 Add `Uncertain::materialize<W>(&self, &mut SampleSession<R>, n)`. Verify: instantiating at both witnesses from one signature yields the same `n` values in the same order.
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
