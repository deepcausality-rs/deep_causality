<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Tasks: retrofitting the sampling layer

Built under `openspec/specs/unified-math-tdd-protocol/spec.md`: five phases per group, in order —
(1) a compiling API with `unimplemented!()` bodies, (2) the full suite written against it and
observed failing for the intended reason, (3) a deliberate-defect audit, (4) implementation judged
finished only by the audited suite, (5) mutation testing. A commit message is prepared at each
group boundary once tests and clippy are green. Nothing is committed here; the user commits.

Baselines to record at 1.1 and preserve or beat: `rand` 154 tests, `stats` and `uncertain` counts
to be measured. Bazel globs `tests/<dir>/*_tests.rs`, so new files in existing directories need no
BUILD change; a new directory needs one glob line.

Order: 1 → 2 → 3 → 4 → 5 → 7. Group 6 is independent and may run at any point.

**Every group leaves the workspace green.** A breaking change to a shared crate repairs its
consumers in the same group, not a later one — `bazel test //...` must pass at every group
boundary, because a commit that breaks the tree is a commit nobody can bisect through. Group 1
proved this the hard way: it cut `uncertain`'s two word-draw sites and the repair was filed under
5.4, which left the workspace broken across four groups. The repair moved to 1.7.

## 1. The entropy crate: what stays

- [x] 1.1 Record the baselines in `notes/baselines.md`. Every later group compares against these.
      Measured at `eaa17aaf3`: `rand` 154, `stats` 467, `uncertain` 248, `topology` 1697, all
      green. `topology` added to the list because 5.7 migrates it.
- [x] 1.2 Phase 1. In `rand`: declared `StandardWord` and `StandardBool`; removed the `u64`,
      `u32` and `bool` impls from `StandardUniform`; split `Rng::random` into `random_word` and
      `random_boolean`. Bodies `unimplemented!()`. `cargo` and `bazel` both build; the two new
      bodies were confirmed to panic rather than return.
      Internal callers redirected to `StandardWord`, which is the separation doing its job:
      `dist_float_common.rs`'s three bit kernels, `uniform_f32.rs`, `uniform_f64.rs` and
      `Bernoulli`'s fixed-point comparison all draw a *word* to build their result, and each said
      `StandardUniform` only because one type used to mean both.
- [x] 1.3 Phase 2. Suite written: `tests/types/dist/uniform/standard_word_bool_tests.rs`, 7 tests,
      covered by the existing glob. Observed failing — **103 passed, 58 failed**, and all 58 panic
      lines name the phase-1 body, so no failure is a compile error or a panic from elsewhere.
      Recorded in `notes/tdd-group-1.md`, including the three migrated test files and the one
      assertion that legitimately changed (the Boolean parity rule).
- [x] 1.4 Phase 3. Defect audit complete, all three rejected. (a) parity Boolean → caught by
      `test_standard_bool`, 1 failure. (b) `random_word` on the numerical path → rejected by the
      **compiler**, `error[E0277]`, since `StandardUniform` no longer implements
      `Distribution<u64>`; the type system enforces the split, which is stronger than a test.
      (c) constant Boolean → caught by 3 tests. Recorded in `notes/tdd-group-1.md`.
- [x] 1.5 Phase 4. Implemented: `StandardWord` returns `next_u64`/`next_u32`, `StandardBool`
      takes the top bit. **161 passed, 0 failed** — the 154 baseline plus exactly the 7 new tests,
      no regression.
- [x] 1.5b Phase 5. Mutation testing: **6 caught, 0 missed, 2 timeout**. The `>>`→`<<` mutant is
      the phase-3 parity defect by another route and is rejected, so the top-bit choice is pinned.
      The two timeouts mutate the `u64` arm to a constant; confirmed by hand that the suite
      **hangs** rather than passes, because the ziggurat's two rejection loops consume words until
      a condition changes and a constant word freezes them. Verdict: caught, not survived — not
      added to `.cargo/mutants.toml`, since they are not equivalent to the original. Underlying
      finding recorded: the ziggurat has no iteration cap, and group 3's Box-Muller removes the
      loop rather than inheriting it.
- [x] 1.7 Repair the consumers group 1 breaks, in group 1. `uncertain`'s `next_sample_index`
      drew a sample *index* through `random::<u64>()` at
      `types/sampler/sampler_seed.rs:52-53`; an index is a machine word, as that function's own
      docstring says, so both sites now use `random_word`. Verified: `cargo build --workspace
      --all-targets` clean, `bazel test //...` green. Moved here from 5.4 — see the ordering note
      at the head of this file.
- [x] 1.6 Clippy clean, no `#[allow]` added. `cargo` 161/161, `bazel` 24/24 targets.
      Bazel caught `os_random_rng_tests.rs`, feature-gated behind `os-random`, which `cargo test`
      never compiled — three more word-draw sites, migrated the same way. Commit message prepared.

## 2. The move: distributions from `rand` to `stats`

- [x] 2.1 Phase 1. `stats` gained `deep_causality_rand`; graph confirmed acyclic (tier 4 → 2).
      24 files moved by `git mv`, history preserved — 14 source, 10 test. **Corrected against the
      plan:** `Uniform<X>` and `Distribution` stay in `rand`. `impl SampleUniform for f64` can only
      be written where the trait lives, that trait backs `Rng::random_range`, and every external
      caller of it draws an **integer** range. `Distribution` is the bridge both crates speak.
      Both specs and the proposal were corrected before the code was written.
      `Rng` lost `random`/`random_iter`/`map`; they return as `stats`'s `RandomExt`, an extension
      trait blanket-implemented for every `Rng`. `RealRng` removed here rather than at 5.1, since
      its blanket names the moved `StandardNormal` and could not compile.
- [x] 2.2 Phase 2. Moved suites run in `stats`; new `tests/traits/random_ext_tests.rs` covers the
      numerical draw over a generator. The `RealRng` test was **restated, not deleted** — the
      property it pinned (one body, several scalars) is what group 3 delivers, so it now carries
      the two-clause bound explicitly, recording the cost a reader can see collapse in group 3.
      Recorded in `notes/tdd-group-2.md`.
- [x] 2.3 Phase 3. The audit was run by the compiler rather than by hand, and more thoroughly:
      removing each item from `rand` turned every consumer of it into a compile error, so the
      "no analytic distribution remains" scenario is enforced by the type system. A pre-existing
      defect surfaced instead — `rand` carried **two different `[0, 1)` kernels** behind two APIs
      documented as the same draw, returning `15.0` and `14.999999` for the same word. The two
      range tests asserted the divergence; they now assert that the paths agree.
- [x] 2.4 Phase 4. Move complete. `rand` 101 (154 baseline, 53 moved out), `stats` 529 (467
      baseline, 62 moved in including 5 new). Combined 630 against 621: +7 group-1 word/bool,
      +5 `RandomExt`, −3 numerical generator tests that moved rather than duplicated.
- [x] 2.5 Clippy clean on both crates, no `#[allow]` added. `cargo build --workspace
      --all-targets` clean. Consumers repaired **in this group** per the group-1 invariant:
      `uncertain` (3 sites), `topology` (all 13, `rand` dropped from its manifest), `physics` (3
      files), `algorithms` (6), `discovery` and the examples. Commit message prepared.
- [x] 2.6 Bazel: added `rust_test_suite` targets for the three new `stats` test directories, and
      removed `rand`'s now-empty `extensions` suite — an empty glob is an error to bazel and
      invisible to cargo. Both crates: 45 of 45 targets pass.

## 3. Precision as a parameter in `stats`

- [x] 3.1 Phase 1. `RandWidth` declared with four impls; `StandardUniform`, `Open01`,
      `OpenClosed01` and `StandardNormal` each declared as one blanket over
      `RealField + FromPrimitive + RandWidth`, bodies `unimplemented!()`. **No E0119** — and it
      cannot return, because `stats` owns no word sampler to collide with. Deleted the three
      per-type files, the shared bit kernels, and the ziggurat sampler and tables: **12 per-type
      implementations become 4 generic ones**. `cargo` and `bazel` both build; 30 failures, all
      the phase-1 panic at the two declared sites.
- [x] 3.2 Phase 2. `tests/types/distr/unit_interval/precision_tests.rs`, 10 tests. Observed
      failing: **501 passed, 39 failed**, and all 39 are the `unimplemented!()` panic at the four
      declared bodies — `StandardUniform` 18, `Open01` 7, `OpenClosed01` 5, `StandardNormal` 9.
      No compile error, no panic from elsewhere. Recorded in `notes/tdd-group-3.md`.
- [x] 3.3 Phase 3. Defect audit. (a) single-word draw → **1 failure**, the low-limb test at
      `0/200`; 539 of 540 tests passed with a `Float106` that was secretly an `f64`, which is the
      measured argument for the requirement. (b) `WORDS = 1` → 2 failures, cause and consequence.
      (c) Box–Muller narrowed through `f64` → caught by `test_double_double_entropy`, a suite moved
      in group 2. (d) clamp instead of redraw → **survived**; the suite was repaired.
- [x] 3.3b The missing atom-of-mass check, added and calibrated by measurement over 200 000
      `BFloat16` draws: the most frequent value takes 0.00429 of them when redrawn and 0.00584 when
      clamped, a 36% excess on one point. The test asserts below 0.005. Re-running defect (d)
      against it now fails, so the gap is closed rather than noted.
- [x] 3.4b Phase 5. Mutation testing: first run 4 missed, all repaired, and **none was a lazily
      written test**. Three were one dead statement — the `scale` update was the loop's last line
      and `WORDS <= 2`, so it was never read, making every mutation of it equivalent; fixed by
      deepening before use rather than after. One was `Open01`'s zero guard at probability
      `2^-53`, unreachable by sampling; closed with an all-zero generator. The last was `+=` to
      `-=` on the second limb, a `1e-16` shift no distributional test can see; closed with
      scripted words asserting `hi + lo * 2^-53` exactly. All verified to fail under their mutant.
      15 timeouts are unsatisfiable rejection loops — caught, not survived.
- [x] 3.4 Phase 4. Implemented: **546 passed, 0 failed**. Four generic bodies replace twelve
      per-type implementations. An implementation change the audit prompted — the intermediate
      moved into `f64` and converted once — is kept for being right by construction, but the
      docstring records that the measurement I expected to justify it **did not move**, rather
      than implying a fix that was not one.
- [x] 3.5 Line delta measured across the pair, which is the meaningful figure since the move
      shifted mass between them: `rand` 2293 → **1396**, `stats` 2788 → **3499**, combined
      5081 → **4895**. **186 lines net removed** while gaining `BFloat16` sampling and precision
      as a parameter. Clippy clean after fixing two `manual_assign_op` sites and one unused import
      — fixed, never `#[allow]`ed. `no_std` builds. Consumers at baseline: `rand` 101,
      `uncertain` 248, `topology` 1697, `physics` 1775. Commit message prepared.

## 4. Seven distributions

Each of 4A to 4H is a self-contained five-phase cycle with its own commit boundary: declare,
write the suite and watch it fail, audit it against deliberate defects, implement, mutate.

Two rules hold across all six. Tests assert against the **closed form**, never against a second
implementation of the same sampler — a sampler checked against a reimplementation of itself checks
neither. And each distribution is exercised at `f32`, `f64` and `Float106`, since a sampler that
only works at one scalar has not been retrofitted.

Order: 4A first, because it establishes the shared harness — the deterministic word source, the
moment helper and the parameter-rejection pattern the others reuse. 4B to 4H are independent of
each other after that, except that 4H follows 4E, whose in-range and rejection tests it shares.

### 4A. Exponential — and the shared harness

- [x] 4A.1 Phase 1. `Exponential<T>` declared with `unimplemented!()` bodies. Harness added to the
      crate's **existing** `utils_tests` module rather than a parallel one: `SUITE_SEED`, `draws`,
      `moments`, `sorted_draws`, `quantile`, `assert_near`, `expect_refused`, `expect_accepted`,
      `lift`. `StatsError` is reused rather than adding a seventh error type — it already carries
      `NonPositiveScale`, `NonFiniteInput`, `NegativeProbability` and `EmptyInput`.
- [x] 4A.2 Phase 2. 10 tests written, observed failing: **1 passed, 9 failed**, all 9 the
      phase-1 panic. The single pass is the harness determinism check, which must pass while the
      distribution does not. Adds memorylessness, which characterises the exponential among all
      continuous distributions and which a right-mean/wrong-shape sampler fails.
- [x] 4A.3 Phase 3. Defect audit, with a **correction to this task**. (a) as written is not a
      defect: with `u` in `[0, 1)`, `1 - u` is in `(0, 1]` and the logarithm is always finite.
      Written and run — 10 passed, 0 failed. The real hazard is the other direction, `-ln(u)` from
      the half-open draw, and it **survived** the whole suite including the 10 000-draw finiteness
      check, because a zero draw sits at probability `2^-53`. An inverse-CDF guard cannot be
      tested by sampling. Closed with `ZeroRng`, promoted into the shared harness for the other
      six suites. (b) `*λ` for `/λ` → 4 failures. (c) rate 0 accepted → 1 failure.
- [x] 4A.4 Phase 4. Implemented: **557 passed, 0 failed** crate-wide.
- [x] 4A.4b Phase 5. Mutation testing: **2 caught, 0 missed**, 3 unviable. No survivors.
- [x] 4A.5 Clippy clean. Prepare the 4A commit message.

### 4B. LogNormal

- [x] 4B.1 Phase 1. `LogNormal<T>` with `new(mu, sigma) -> Result`, body `unimplemented!()`.
- [x] 4B.2 Phase 2. Mean of 200 000 draws at `(0, 0.5)` within 1% of `e^{σ²/2} = e^0.125 =
      1.1331`. Every draw strictly positive — the support is `(0, ∞)`, and a non-positive value is
      a structural failure rather than a statistical one, so this is asserted on **every** draw
      rather than on a moment. Median within 2% of `e^μ`. All three scalars. Rejection:
      non-positive sigma, non-finite mu or sigma. Run; record.
- [x] 4B.3 Phase 3. Defect audit: (a) returning the underlying normal without exponentiating —
      must fail the positivity assertion on roughly half the draws; (b) `e^{μ + σZ}` with sigma
      applied before the mean shift as `e^{σ(μ + Z)}` — must fail the mean while leaving the
      median plausible at `μ = 0`, which is why the test uses a non-zero `σ`; (c) the mean asserted
      against `e^μ` rather than `e^{μ+σ²/2}`, the textbook confusion between the median and the
      mean of a lognormal — confirm the suite distinguishes them.
- [x] 4B.4 Phase 4. Implement. Phase 5: mutation testing.
- [x] 4B.5 Clippy clean. Prepare the 4B commit message.

### 4C. Cauchy — quantiles only

- [x] 4C.1 Phase 1. `Cauchy<T>` with `new(location, scale) -> Result`, body `unimplemented!()`.
- [x] 4C.2 Phase 2. Median of 20 000 standard draws within 0.05 of 0. Interquartile range within
      0.1 of 2. Location and scale recovered from a shifted, scaled instance. All three scalars.
      Rejection: non-positive scale, non-finite parameters. Run; record.
- [x] 4C.3 Phase 3, and the audit that justifies the whole approach. Defect audit:
      (a) `tan(π·u)` instead of `tan(π(u − ½))` — a half-turn offset that leaves the *distribution*
      correct but the **median** wrong, so it is caught by 4C.2 and would be invisible to any
      moment test; (b) scale applied additively rather than multiplicatively — must fail the IQR;
      (c) **a mean-based test written deliberately** — run it across at least five seeds and record
      that it passes on some and fails on others. That instability is the evidence for the
      prohibition below, and recording it is the point of the exercise.
- [x] 4C.4 The test module contains **no** assertion on a sample mean or variance, and its
      docstring states why: the Cauchy distribution has neither, the defining integrals diverge, a
      sample mean does not converge but wanders with a Cauchy distribution of its own. Such a test
      is meaningless rather than merely weak, and it survives review while failing intermittently
      on a seed change. Cite the 4C.3(c) measurement.
- [x] 4C.5 Phase 4. Implement. Phase 5: mutation testing.
- [x] 4C.6 Clippy clean. Prepare the 4C commit message.

### 4D. Weibull

- [x] 4D.1 Phase 1. `Weibull<T>` with `new(shape, scale) -> Result`, body `unimplemented!()`.
- [x] 4D.2 Phase 2. Mean of 200 000 draws at `(k=2, λ=1)` within 1% of `λ·Γ(1 + 1/k) =
      sqrt(pi)/2 = 0.8862`. A second parameter pair at `k = 1`, where the Weibull **is** the
      exponential with rate `1/λ`, cross-checked against 4A's closed form — the one place a
      second implementation is a legitimate oracle, because the identity is analytic rather than a
      reimplementation. Every draw non-negative. All three scalars. Rejection: non-positive shape
      or scale, non-finite parameters. The `ln(u)` guard. Run; record.
- [x] 4D.3 Phase 3. Defect audit: (a) the exponent inverted, `^k` for `^(1/k)` — must fail the
      mean at `k = 2` and pass at `k = 1`, which is why both are tested; (b) scale applied inside
      the power rather than outside; (c) the `k = 1` case disagreeing with the exponential.
- [x] 4D.4 Phase 4. Implement. Phase 5: mutation testing.
- [x] 4D.5 Clippy clean. Prepare the 4D commit message.

### 4E. Categorical

- [x] 4E.1 Phase 1. `Categorical<T>` with `new(weights) -> Result`, body `unimplemented!()`.
- [x] 4E.2 Phase 2. Weights `1:3:6` give frequencies within 0.01 of `0.1`, `0.3`, `0.6` over
      200 000 draws. **Unnormalised** weights give the same frequencies as their normalised
      counterparts — the sampler normalises, the caller does not have to. A single-weight vector
      always returns index 0. A weight of exactly zero is never selected. Every returned index is
      in range. All three scalars. Rejection: empty vector, any negative weight, weights summing
      to zero, any non-finite weight. Run; record.
- [x] 4E.3 Phase 3. Defect audit: (a) scanning without dividing by the weight total — must fail on
      the unnormalised case and pass on the normalised one, which is why both are tested;
      (b) an off-by-one returning `i+1` — must fail the in-range assertion on the last index;
      (c) a zero-weight category selected when the cumulative comparison uses `<=` rather than `<`;
      (d) floating-point drift on the final category, where the accumulated remainder can leave the
      last comparison marginal — confirm the fallback returns the last index rather than panicking.
- [x] 4E.4 Phase 4. Implement. Phase 5: mutation testing.
- [x] 4E.5 Clippy clean. Prepare the 4E commit message.

### 4F. Poisson — and its domain

- [x] 4F.1 Phase 1. `Poisson<T>` with `new(lambda) -> Result`, body `unimplemented!()`. The
      declaration states the algorithm's valid range in its docstring from the outset, since the
      range is part of the contract rather than an implementation detail.
- [x] 4F.2 Phase 2. Mean within 1% and variance within 5% of `lambda` at `lambda = 3`, over
      200 000 draws — the Poisson's mean and variance are equal, and a sampler that gets the mean
      right while getting the variance wrong is a common failure, so both are asserted.
      `lambda = 0` gives every draw `0`. Every draw is a non-negative integer. All three scalars.
      Run; record.
- [x] 4F.3 Phase 2b, the domain. A `lambda` beyond the documented range — including one large
      enough that `exp(-lambda)` underflows to zero at the working scalar, which is near 700 at
      `f64` and far sooner at `f32` — returns an error or a correct value. It does **not** loop
      indefinitely. The test carries a timeout or an iteration cap so a regression fails rather
      than hangs the suite.
- [x] 4F.4 Phase 3. Defect audit: (a) `p < l` for `p <= l` in the termination comparison —
      determine whether the suite distinguishes them and **record the verdict either way**; if it
      does not, say so rather than inventing a test that appears to; (b) returning `k+1` for `k`,
      an off-by-one that shifts the mean by exactly 1 and is caught at `lambda = 3` but would hide
      at large `lambda`; (c) the underflow case looping — must be caught by 4F.3's cap.
- [x] 4F.5 Phase 4. Implement, either refusing an out-of-range `lambda` or switching method.
      Which of the two is an implementation decision; that one of them happens is not.
      Phase 5: mutation testing, with the termination loop as the target of interest.
- [x] 4F.6 Clippy clean. Prepare the 4F commit message.

### 4H. Discrete uniform — unweighted choice over a range

Ordered after 4E because it shares the in-range and rejection tests, and because Categorical's
cumulative scan is the weighted sibling of the same operation.

- [x] 4H.1 Phase 1. `UniformInt` with `new(range) -> Result`, body `unimplemented!()`. The
      docstring names the method used to avoid modulo bias from the outset, since freedom from bias
      is the contract rather than an implementation detail.
- [x] 4H.2 Phase 2. 700 000 draws over a range of **7** give each index within 1% of `1/7`. Ranges
      of 1, 2 and 1000 always return in-range values, and a range of 1 always returns its single
      value. Rejection: the empty range. Run; record in `notes/tdd-group-4h.md`.

      **Corrected while writing 4H.3.** This task and 4H.3(a) were drafted on the premise that a
      uniformity test at range 7 detects modulo bias. It does not, and the audit measured why:
      `2^64 / 7` is `2^61`, so `w % 7` over-weights the low residues by one part in `2^61` —
      4e-19 relative, below any feasible sample size. Range 7 still earns its place as an awkward
      size for a block-based map, but it is not the bias test. Two tests were added that do detect
      the defect, and both are recorded under 4H.3.
- [x] 4H.3 Phase 3. Defect audit: (a) **`next_u64() % n`**, the hand-written form this replaces;
      (b) an off-by-one giving `1..=n` instead of `0..n`; (c) a rejection loop that retries on the
      wrong condition and silently narrows the range; (d) the `usize` draw written separately from
      the `u64` draw; (e) the rejection dropped entirely.

      Defect (a) needs a range where the bias exists: at `len = 12 297 829 382 473 034 411` one
      whole copy of the range fits in a word and half a second, so modulo puts 2/3 of its mass in
      the lower half against a uniform 1/2. Defect (e) is statistically invisible at every range —
      it moves each index by at most one part in `2^64` — so it is caught deterministically
      instead, by a scripted generator whose first word falls in the short block and must be
      thrown away.
- [x] 4H.4 Phase 4. Implement. Phase 5: mutation testing, with the rejection condition as the
      target of interest.
- [x] 4H.5 Clippy clean. Prepare the 4H commit message.

### 4G. Group close-out

- [x] 4G.1 All seven run at `f32`, `f64` and `Float106` in one table-driven test, so a scalar
      added later is one row rather than seven edits. `UniformInt` returns an index rather than a
      scalar, so it is exercised for its generator-precision independence rather than its own —
      the table asserts its index sequence is identical at all three scalars.
- [x] 4G.2 Confirm `stats`'s suite is at or above its 1.1 baseline plus the new tests. **624
      against a baseline of 467.** Recorded in `notes/baselines.md`.
- [x] 4G.3 Clippy clean across the crate. Prepare the group-4 summary commit message.
- [x] 4G.4 Phase 5 for the whole group, batched onto a green tree: 140 mutants, 75 caught, 31
      unviable, 20 timed out, 14 survived. Four survivors were real gaps and are closed; the rest
      are exact equivalences. Recorded in `notes/mutation-group-4.md`.

## 5. Consumers, and the `uncertain` migration

- [x] 5.1 Remove `RealRng`; confirm nothing outside its own test file referenced it. Removed in
      group 1; the only surviving mention is a comment in `standard_normal_tests.rs` recording
      that it had no consumer.
- [x] 5.2 Retype the remaining hardcoded `f64`: `Bernoulli::new`/`p` in `stats`,
      `Rng::random_bool` and `SobolSequence::coordinate`/`point` in `rand`. Document the two
      fixed-width exceptions at the item, with a test that two `Float106` Sobol coordinates within
      the 32-bit resolution compare equal.

      **`random_bool` carried a defect, found by writing the endpoint test first.** The body was
      `word / u64::MAX <= p`, which reads `0 <= 0` on a zero word: an event of probability zero
      fired once in every `2^64` draws. No sampling test reaches that; a zero generator reaches it
      on the first draw, and did. The unit-draw comparison that replaces it makes `p = 0` exact,
      and `p = 1` is special-cased because a narrow significand can round a draw from `[0, 1)`
      onto exactly `1.0` — at `f32` about once in `2^25` draws, which would make a certain event
      occasionally fail to occur.

      The two fixed-width exceptions are `Bernoulli`'s `2^-64` quantisation, which buys exactness
      at both endpoints, and Sobol's `2^-32` resolution, which is the direction-number table's.
- [x] 5.3 **The umbrella.** Migrate `uncertain`'s distribution layer to `stats`:
      `types/distribution/mod.rs` (`Bernoulli`, `Normal`, `Uniform`, `Distribution`),
      `errors/uncertain_error.rs` (the three distribution errors), and the four inverse-CDF
      imports in `types/sampler/qmc_sampler.rs`. What remains of `rand` there is `Xoshiro256`,
      `rng()`, `SobolSequence` and `MAX_SOBOL_DIM` — entropy only.
- [x] 5.4 The two word-draw sites at `uncertain/src/types/sampler/sampler_seed.rs:52-53` were
      repaired in **1.7**, because group 1 is what broke them and a group must leave the tree
      green. Nothing further is owed here. Do not enter that crate beyond the import moves in 5.3:
      `SampledValue`, the sample cache and the lazy graph belong to its own renovation.
- [x] 5.5 Assert the endpoint: a search of `uncertain`'s sources for `deep_causality_rand` returns
      only generator, thread-RNG, Sobol **and range-sampler** references. Its suite passes at or
      above baseline — 248, exactly the baseline.

      **Corrected.** The endpoint as written assumed `Uniform<X>` would move to `stats`. It cannot:
      it is built on `SampleUniform`, which can only be implemented in the crate that owns it, and
      the spec settles the question — range sampling stays in the entropy crate and the `stats`
      re-export covers the generator traits and nothing else. So `uncertain` names `rand` for
      `Uniform` and `UniformDistributionError`, and that is truthful rather than a leftover. What
      the endpoint does assert, and what holds: **no shaped distribution reaches `uncertain` from
      `rand` any more.**
- [x] 5.6 Migrate the other distribution consumers' imports: `physics` (6 uses), `discovery` (1) —
      done in group 3. `Normal<F>` carried `StandardNormal: Distribution<F>` on its **struct
      definition** and on its constructor impl; both are gone. A bound on the data says something
      the type does not need: `Normal` holds two scalars and knows nothing about drawing from them.
      The sampler states the requirement where the draw happens.
- [x] 5.7 **Transition `topology` to `stats` only.** Thirteen `rand` references, counted, and each
      has a destination:
      - [x] 5.7.a The six `RngType: deep_causality_rand::Rng` bounds in `gauge_field_lattice/`
            and `link_variable/` become the generator trait re-exported from `stats`.
      - [x] 5.7.b `cubical_regge_geometry/metropolis.rs` imports `Normal`, `StandardUniform` and
            `StandardNormal` from `stats`, and **both leaked `where` clauses are removed** — the
            tier-7 leak this change exists to repair.
      - [x] 5.7.c The edge pick, `(rng.next_u64() as usize) % num_edges` at
            `metropolis.rs:122`, becomes a `UniformInt` draw from 4H. This is the one operation
            that needed a new distribution, and it is also the one place topology's randomness is
            currently biased.
      - [x] 5.7.d `RandomField` in `types/gauge/link_variable/random.rs` drops its hand-rolled
            `rng.random::<f64>() - 0.5` for a `stats` draw at the caller's scalar. Its docstring
            currently says it "bridges the gap between `deep_causality_rand` and algebraic types";
            that gap is what this change closes. Add a test that the gauge field draws at `f32` and
            `Float106`, a precision it does not have today.
      - [x] 5.7.e `LawRng` in `utils_tests/hkt_law_utils.rs` stays unchanged: it is a hand-rolled
            deterministic generator depending on neither crate, and its determinism is the point.
      - [x] 5.7.f Remove `deep_causality_rand` from `topology`'s manifest; add
            `deep_causality_stats`. Confirm the crate builds and its suite passes at or above
            baseline.
- [x] 5.8 Assert the endpoint by search: `topology`'s sources contain no `deep_causality_rand`
      reference outside `utils_tests`, and its manifest names none.
- [x] 5.9 Confirm the entropy-only consumers need **no** edit: `algorithms`, `tensor`,
      `ultragraph`, `data_structures`. Build `quantum` with `--features qpu` for the transitive
      path through `uncertain`.
- [x] 5.10 Clippy clean across every touched crate. Prepare the group-5 commit message.

## 6. The diagonal traversal in `haft`

Independent of groups 1–5; it closes a gap that exists today for `ZipTensorWitness` and
`ZipDenseVectorWitness` regardless of sampling.

- [x] 6.1 Phase 1. Declare the traversal in `haft` beside `Traversable`, bounded on `Semigroupal`
      of the inner witness and taking the seed as a parameter. Implement for
      `CausalTensorWitness` and `DenseVectorWitness`. All bodies `unimplemented!()`.
- [x] 6.2 Phase 2. Suite: a 2×2 field of 50 draws per cell returns **50** fields of shape `[2, 2]`,
      and field *i* holds draw *i* of every cell — **assert exact values, not the count**; the
      empty structure returns the seed; ragged columns truncate the ensemble to the shortest while
      every field retains every cell. Run; record in `notes/tdd-group-6.md`. Six tests for the
      tensor witness, four for the vector one; 10 of 10 observed failing at the phase-1 body.
- [x] 6.3 Phase 3. Defect audit: (a) the cartesian applicative in place of the diagonal;
      (b) pairing draw *i* with draw *i+1*; (c) truncating the field cells instead of the ensemble.

      **(a) cannot be written.** Substituting the cartesian traversal does not compile —
      `error[E0277]: the trait bound ZipTensorWitness: Applicative<ZipTensorWitness> is not
      satisfied` — and the reverse substitution fails for the mirror reason. Each traversal takes
      exactly the witness whose pairing it means. What is kept is the measurement of what the
      cartesian one does instead: `3^4` against 3.

      **(b) was run twice, and the first form did not make the point.** `skip(1)` shortens the
      result, so a count test catches it too. Rotating instead leaves the length exactly right and
      only the values wrong: the count test **passes** and four value tests fail. That is the
      evidence for writing the assertion on values.
- [x] 6.4 Phase 4. Implement. Phase 5: mutation testing on the traversal.

      **Mutation testing reaches nothing here.** All 13 mutants generated for `sequence_zip` were
      unviable: every one replaces the body with a constructed `M::Type<CausalTensor<A>>`, an
      opaque generic associated type no expression can build, and the body holds no operator to
      flip. A run reporting 0 missed is not evidence. The scripted audit in 6.3 is the phase-5
      evidence, which is why (b′) was worth constructing carefully.
- [x] 6.5 Clippy clean. Prepare the group-6 commit message.

## 7. Example and documentation

- [x] 7.1 New example `examples/mathematics_examples/composable_multi_math/ensemble_x_lattice_ising/`
      (`main.rs` + `README.md`, matching the seven siblings). The 2D Ising model on a periodic
      lattice: `rand` supplies entropy, `stats` the acceptance draw, `tensor` each lattice and the
      ensemble of replicas, `haft` the map and fold.

      Chosen because it carries a closed-form check, as every example in that folder does:
      Onsager's exact `Tc = 2 / ln(1 + sqrt(2)) ~= 2.2692`. Verified before proposing — a 16×16
      lattice over 2 000 sweeps gives `|m|` of 0.986 at `T = 1.5`, 0.732 at `Tc` and 0.119 at
      `T = 3.5`. The ensemble layer too: `<|m|> = 0.9338 ± 0.0292` over 32 replicas at `T = 2.0`.

      - [x] 7.1.a The ensemble as nested witnessed containers,
            `CausalTensor<CausalTensor<FloatType>>`; observables by `fmap`, ensemble mean by
            `fold`.
      - [x] 7.1.b The susceptibility `chi = beta * N * (<m^2> - <m>^2)` through the **diagonal**
            traversal from group 6. The reason is physical: a susceptibility is a fluctuation, so
            it is meaningful only if replica *i*'s observables stay paired. The cartesian traversal
            would combine replica 3's energy with replica 17's magnetisation. Assert against a
            directly computed variance.
      - [x] 7.1.c Precision per part: the Metropolis draw and the ensemble mean are noise-bound and
            run at `f32`. One table, three precisions.

            **The rest of this task's premise was wrong, and the table now measures why.** It said
            `chi` near `Tc` is cancellation-bound and needs a wider scalar. Two measurements say
            otherwise. First, the cancellation is *mildest* at `Tc`: `chi` is proportional to the
            variance and a critical point is where the variance diverges, so the fluctuation is
            largest exactly where the quantity is asked for — `<|m|> = 0.74 +/- 0.15` at `Tc`
            against `0.99 +/- 0.01` at `T = 1.5`. Second, at a 16x16 lattice no scalar can
            disagree at all: `|m| = k/N` is dyadic, so the reduction needs `2 log2(N) + log2(R)` =
            21 bits, inside `f32`'s 24, and all three scalars return **bit-identical** results.

            The table is now a 2x2 — representable or not, fluctuation large or small — over
            `L = 16` and `L = 32`, the second chosen because the same arithmetic puts the threshold
            at 25 bits there. Only the cell where both go wrong costs anything: `7.7e-5` against
            `3e-8`, `4.4e-10` and `1.4e-7`.
      - [x] 7.1.d Register in `examples/mathematics_examples/Cargo.toml` as
            `[[example]] name = "ensemble_x_lattice_ising_examples"` and in that crate's
            `BUILD.bazel` with the `srcs` glob and `crate_root` its siblings use.
      - [x] 7.1.e Add the row to `composable_multi_math/README.md` under "HKT-Only Composition".
      - [x] 7.1.f Repo example conventions: one `FloatType` alias in `main.rs`, no local lift
            helpers, no raw `f64` except at the display boundary through `lower`.

      Deferred: the same composition over `LatticeGaugeField` and `try_metropolis_sweep`, which
      `topology` already carries — Wilson loops in a U(1) gauge theory. Ising first because its
      exact answer is unambiguous; the gauge version is a capstone once this is proven.

- [x] 7.2 Document in `stats`: distributions live here because their mathematics does, and a
      lazy sampler is Arrow-shaped rather than witness-shaped. Document in `rand`: this crate
      supplies entropy and makes no distributional claim.
- [x] 7.3 Update `openspec/notes/unified_math/hkt_gaps.md` §3.4 and `hkt_uncertain.md` B7/B8:
      B7's "six per-type files to collapse" premise is superseded by the crate boundary; B8 is
      closed; `RealRng`'s zero-consumer status is resolved by removal.
- [x] 7.4 Update `deep_causality_unified_math/README.md`: the Monte Carlo example loses its
      `where StandardUniform: Distribution<S>` clause; the crate table's `rand` and `stats` rows
      state the new division; the tier diagram is unchanged, since `stats -> rand` is downhill.

      The replacement signature was compiled before being written into the README, and the check
      was kept as `traits/generic_sampling_tests.rs` — it is the spec scenario "it compiles with no
      `StandardUniform: Distribution<S>` clause", which had no test until now.

## 8. Close-out

- [x] 8.1 `bazel test //...` green across the workspace. **1 390 tests pass** at `0ada8bdce`.
- [x] 8.2 Coverage on every changed file; any miss explained rather than excused. Recorded in
      `notes/coverage.md`: twelve files at 100%, the rest at 92-99%, and every miss named.

      Two are worth carrying out of this change. The four residual lines across `poisson`,
      `bernoulli` and `sobol` are `Unexecuted instantiation` placeholders — `llvm-cov` counts each
      monomorphisation, and `Poisson<_>` is not one that runs — so the region figures are the truer
      reading. And `rng.rs`'s three uncovered lines are `Rng::fill`, which **no test can reach**:
      `Fill` has no implementors anywhere in the workspace, so the method is uncallable. That is
      dead public surface rather than a coverage gap, it predates this change, and removing it is
      not a close-out's decision to make.
- [x] 8.3 `openspec validate retrofit-sampling-layer --strict` — valid.
- [x] 8.4 Prepare the final batch commit message, naming the breaking changes for the release
      notes: distributions moving `rand` → `stats`, the `StandardUniform` split, the `Rng::random`
      split, the retyped `f64` surface, and the `RealRng` removal.
