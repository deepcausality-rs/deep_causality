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
- [x] 1.6 Clippy clean, no `#[allow]` added. `cargo` 161/161, `bazel` 24/24 targets.
      Bazel caught `os_random_rng_tests.rs`, feature-gated behind `os-random`, which `cargo test`
      never compiled — three more word-draw sites, migrated the same way. Commit message prepared.

## 2. The move: distributions from `rand` to `stats`

- [ ] 2.1 Phase 1. Add `deep_causality_rand` to `stats`'s dependencies; confirm the graph stays
      acyclic. `git mv` the distribution modules into `stats` (history preserved, per the repo
      rule): `Normal`, `Uniform`, `Bernoulli`, `StandardUniform`, `StandardNormal`, `Open01`,
      `OpenClosed01`, the `Distribution` trait, the four inverse-CDF functions, the three
      distribution error types. Bodies `unimplemented!()` where a signature changes; unchanged
      bodies move as they are. Build both crates under `cargo` and `bazel`.
- [ ] 2.2 Phase 2. Suite in `stats`: every moved item is reachable from the crate root; a density
      and its sampler are used together in one test with one import; `rand` exports no
      distribution type. Run; record in `notes/tdd-group-2.md`.
- [ ] 2.3 Phase 3. Defect audit: (a) an item left exported from `rand` as well — must fail the
      "no distribution type remains" scenario; (b) a moved item that silently lost its error
      variant.
- [ ] 2.4 Phase 4. Complete the move. Confirm `rand`'s own suite still passes at its baseline
      minus only the distribution tests, which move with their code.
- [ ] 2.5 Clippy clean on both crates. Prepare the group-2 commit message.

## 3. Precision as a parameter in `stats`

- [ ] 3.1 Phase 1. Declare `RandWidth` with its four impls; declare the blanket
      `impl<T: RealField + FromPrimitive + RandWidth> Distribution<T> for StandardUniform` and the
      generic `StandardNormal`; delete `dist_float_32.rs`, `dist_float_64.rs`,
      `dist_float_common.rs`. Bodies `unimplemented!()`. Build; **confirm no E0119**.
- [ ] 3.2 Phase 2. Suite: uniform mean within 0.02 of 0.5 and `E[x²]` within 0.02 of 1/3 at `f32`,
      `f64`, `Float106`; normal variance within 0.05 of 1 at the same three; `BFloat16` draws a
      uniform and a finite normal; **more than half of 200 `Float106` draws differ from their own
      `f64` round trip**; at least 1 000 draws per scalar all in `[0, 1)`; a `BFloat16` mean over a
      count small enough to avoid saturation is within 0.05 of 0.5. Run; record.
- [ ] 3.3 Phase 3. Defect audit: (a) a single-word draw for `Float106` — **must** fail the low-limb
      scenario; this is the load-bearing test of the group, since every moment test passes without
      it; (b) `WORDS = 1` for `Float106`, the same defect via the constant; (c) `StandardNormal`
      computing Box–Muller in `f64` and widening — must fail at `Float106`; (d) a narrow scalar
      clamped to the largest value below 1 rather than redrawn — confirm an atom-of-mass check
      exists and add one if not.
- [ ] 3.4 Phase 4. Implement. Phase 5: mutation testing on the uniform and normal bodies; the
      redraw loop and the word-count loop are the interesting targets.
- [ ] 3.5 Record the line delta against the 2 293-line `rand` baseline. Clippy clean. Prepare the
      group-3 commit message.

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

- [ ] 4A.1 Phase 1. `Exponential<T>` with `new(rate) -> Result`, body `unimplemented!()`. Alongside
      it the harness the other five reuse: a deterministic word source, a moment helper taking a
      sample count and a tolerance, and a parameter-rejection helper.
- [ ] 4A.2 Phase 2. Mean of 200 000 draws at rate 2 within 1% of 0.5. Variance within 5% of
      `1/λ² = 0.25`. Drawn at `f32`, `f64`, `Float106`. Rejection: non-positive rate, `NaN` rate,
      infinite rate. The `ln(u)` guard: 10 000 draws, none infinite or `NaN`. Run against the
      unimplemented API; record the failing output and count in `notes/tdd-group-4a.md`.
- [ ] 4A.3 Phase 3. Defect audit — write each, confirm rejection, record which test caught it:
      (a) `-ln(1-u)/λ` where `u` comes from a half-open draw that can be 0 — must fail the
      `ln(u)` guard, not the mean, since `1-u` reaching 1 gives `ln(1) = 0` and a spuriously
      finite result while `u = 0` gives an infinity; (b) `-ln(u)*λ` instead of `/λ` — must fail
      the mean at any rate other than 1, which is why the test uses rate 2; (c) a constructor
      accepting rate 0.
- [ ] 4A.4 Phase 4. Implement. Phase 5: `scripts/mutants.sh` on the module; survivors get a
      verdict, equivalents go to `.cargo/mutants.toml` with a reason.
- [ ] 4A.5 Clippy clean. Prepare the 4A commit message.

### 4B. LogNormal

- [ ] 4B.1 Phase 1. `LogNormal<T>` with `new(mu, sigma) -> Result`, body `unimplemented!()`.
- [ ] 4B.2 Phase 2. Mean of 200 000 draws at `(0, 0.5)` within 1% of `e^{σ²/2} = e^0.125 =
      1.1331`. Every draw strictly positive — the support is `(0, ∞)`, and a non-positive value is
      a structural failure rather than a statistical one, so this is asserted on **every** draw
      rather than on a moment. Median within 2% of `e^μ`. All three scalars. Rejection:
      non-positive sigma, non-finite mu or sigma. Run; record.
- [ ] 4B.3 Phase 3. Defect audit: (a) returning the underlying normal without exponentiating —
      must fail the positivity assertion on roughly half the draws; (b) `e^{μ + σZ}` with sigma
      applied before the mean shift as `e^{σ(μ + Z)}` — must fail the mean while leaving the
      median plausible at `μ = 0`, which is why the test uses a non-zero `σ`; (c) the mean asserted
      against `e^μ` rather than `e^{μ+σ²/2}`, the textbook confusion between the median and the
      mean of a lognormal — confirm the suite distinguishes them.
- [ ] 4B.4 Phase 4. Implement. Phase 5: mutation testing.
- [ ] 4B.5 Clippy clean. Prepare the 4B commit message.

### 4C. Cauchy — quantiles only

- [ ] 4C.1 Phase 1. `Cauchy<T>` with `new(location, scale) -> Result`, body `unimplemented!()`.
- [ ] 4C.2 Phase 2. Median of 20 000 standard draws within 0.05 of 0. Interquartile range within
      0.1 of 2. Location and scale recovered from a shifted, scaled instance. All three scalars.
      Rejection: non-positive scale, non-finite parameters. Run; record.
- [ ] 4C.3 Phase 3, and the audit that justifies the whole approach. Defect audit:
      (a) `tan(π·u)` instead of `tan(π(u − ½))` — a half-turn offset that leaves the *distribution*
      correct but the **median** wrong, so it is caught by 4C.2 and would be invisible to any
      moment test; (b) scale applied additively rather than multiplicatively — must fail the IQR;
      (c) **a mean-based test written deliberately** — run it across at least five seeds and record
      that it passes on some and fails on others. That instability is the evidence for the
      prohibition below, and recording it is the point of the exercise.
- [ ] 4C.4 The test module contains **no** assertion on a sample mean or variance, and its
      docstring states why: the Cauchy distribution has neither, the defining integrals diverge, a
      sample mean does not converge but wanders with a Cauchy distribution of its own. Such a test
      is meaningless rather than merely weak, and it survives review while failing intermittently
      on a seed change. Cite the 4C.3(c) measurement.
- [ ] 4C.5 Phase 4. Implement. Phase 5: mutation testing.
- [ ] 4C.6 Clippy clean. Prepare the 4C commit message.

### 4D. Weibull

- [ ] 4D.1 Phase 1. `Weibull<T>` with `new(shape, scale) -> Result`, body `unimplemented!()`.
- [ ] 4D.2 Phase 2. Mean of 200 000 draws at `(k=2, λ=1)` within 1% of `λ·Γ(1 + 1/k) =
      sqrt(pi)/2 = 0.8862`. A second parameter pair at `k = 1`, where the Weibull **is** the
      exponential with rate `1/λ`, cross-checked against 4A's closed form — the one place a
      second implementation is a legitimate oracle, because the identity is analytic rather than a
      reimplementation. Every draw non-negative. All three scalars. Rejection: non-positive shape
      or scale, non-finite parameters. The `ln(u)` guard. Run; record.
- [ ] 4D.3 Phase 3. Defect audit: (a) the exponent inverted, `^k` for `^(1/k)` — must fail the
      mean at `k = 2` and pass at `k = 1`, which is why both are tested; (b) scale applied inside
      the power rather than outside; (c) the `k = 1` case disagreeing with the exponential.
- [ ] 4D.4 Phase 4. Implement. Phase 5: mutation testing.
- [ ] 4D.5 Clippy clean. Prepare the 4D commit message.

### 4E. Categorical

- [ ] 4E.1 Phase 1. `Categorical<T>` with `new(weights) -> Result`, body `unimplemented!()`.
- [ ] 4E.2 Phase 2. Weights `1:3:6` give frequencies within 0.01 of `0.1`, `0.3`, `0.6` over
      200 000 draws. **Unnormalised** weights give the same frequencies as their normalised
      counterparts — the sampler normalises, the caller does not have to. A single-weight vector
      always returns index 0. A weight of exactly zero is never selected. Every returned index is
      in range. All three scalars. Rejection: empty vector, any negative weight, weights summing
      to zero, any non-finite weight. Run; record.
- [ ] 4E.3 Phase 3. Defect audit: (a) scanning without dividing by the weight total — must fail on
      the unnormalised case and pass on the normalised one, which is why both are tested;
      (b) an off-by-one returning `i+1` — must fail the in-range assertion on the last index;
      (c) a zero-weight category selected when the cumulative comparison uses `<=` rather than `<`;
      (d) floating-point drift on the final category, where the accumulated remainder can leave the
      last comparison marginal — confirm the fallback returns the last index rather than panicking.
- [ ] 4E.4 Phase 4. Implement. Phase 5: mutation testing.
- [ ] 4E.5 Clippy clean. Prepare the 4E commit message.

### 4F. Poisson — and its domain

- [ ] 4F.1 Phase 1. `Poisson<T>` with `new(lambda) -> Result`, body `unimplemented!()`. The
      declaration states the algorithm's valid range in its docstring from the outset, since the
      range is part of the contract rather than an implementation detail.
- [ ] 4F.2 Phase 2. Mean within 1% and variance within 5% of `lambda` at `lambda = 3`, over
      200 000 draws — the Poisson's mean and variance are equal, and a sampler that gets the mean
      right while getting the variance wrong is a common failure, so both are asserted.
      `lambda = 0` gives every draw `0`. Every draw is a non-negative integer. All three scalars.
      Run; record.
- [ ] 4F.3 Phase 2b, the domain. A `lambda` beyond the documented range — including one large
      enough that `exp(-lambda)` underflows to zero at the working scalar, which is near 700 at
      `f64` and far sooner at `f32` — returns an error or a correct value. It does **not** loop
      indefinitely. The test carries a timeout or an iteration cap so a regression fails rather
      than hangs the suite.
- [ ] 4F.4 Phase 3. Defect audit: (a) `p < l` for `p <= l` in the termination comparison —
      determine whether the suite distinguishes them and **record the verdict either way**; if it
      does not, say so rather than inventing a test that appears to; (b) returning `k+1` for `k`,
      an off-by-one that shifts the mean by exactly 1 and is caught at `lambda = 3` but would hide
      at large `lambda`; (c) the underflow case looping — must be caught by 4F.3's cap.
- [ ] 4F.5 Phase 4. Implement, either refusing an out-of-range `lambda` or switching method.
      Which of the two is an implementation decision; that one of them happens is not.
      Phase 5: mutation testing, with the termination loop as the target of interest.
- [ ] 4F.6 Clippy clean. Prepare the 4F commit message.

### 4H. Discrete uniform — unweighted choice over a range

Ordered after 4E because it shares the in-range and rejection tests, and because Categorical's
cumulative scan is the weighted sibling of the same operation.

- [ ] 4H.1 Phase 1. `UniformInt` with `new(range) -> Result`, body `unimplemented!()`. The
      docstring names the method used to avoid modulo bias from the outset, since freedom from bias
      is the contract rather than an implementation detail.
- [ ] 4H.2 Phase 2. 200 000 draws over a range of **7** — a size dividing no power of two, so a
      biased implementation shows — give each index within 1% of `1/7`. Ranges of 1, 2 and 1000
      always return in-range values, and a range of 1 always returns its single value. Rejection:
      empty range, inverted range. Run; record in `notes/tdd-group-4h.md`.
- [ ] 4H.3 Phase 3. Defect audit: (a) **`next_u64() % n`**, the hand-written form this replaces —
      the low residues occur once more often than the high ones, so it must fail the uniformity
      test at a range of 7 while passing every in-range assertion. This is the defect the whole
      distribution exists to remove, and the audit is the evidence that the test detects it;
      (b) an off-by-one giving `1..=n` instead of `0..n`; (c) a rejection loop that retries on the
      wrong condition and silently narrows the range.
- [ ] 4H.4 Phase 4. Implement. Phase 5: mutation testing, with the rejection condition as the
      target of interest.
- [ ] 4H.5 Clippy clean. Prepare the 4H commit message.

### 4G. Group close-out

- [ ] 4G.1 All seven run at `f32`, `f64` and `Float106` in one table-driven test, so a scalar
      added later is one row rather than seven edits. `UniformInt` returns an index rather than a
      scalar, so it is exercised for its generator-precision independence rather than its own.
- [ ] 4G.2 Confirm `stats`'s suite is at or above its 1.1 baseline plus the new tests.
- [ ] 4G.3 Clippy clean across the crate. Prepare the group-4 summary commit message.

## 5. Consumers, and the `uncertain` migration

- [ ] 5.1 Remove `RealRng`; confirm nothing outside its own test file referenced it.
- [ ] 5.2 Retype the remaining hardcoded `f64`: `Bernoulli::new`/`p` in `stats`,
      `Rng::random_bool` and `SobolSequence::coordinate`/`point` in `rand`. Document the two
      fixed-width exceptions at the item, with a test that two `Float106` Sobol coordinates within
      the 32-bit resolution compare equal.
- [ ] 5.3 **The umbrella.** Migrate `uncertain`'s distribution layer to `stats`:
      `types/distribution/mod.rs` (`Bernoulli`, `Normal`, `Uniform`, `Distribution`),
      `errors/uncertain_error.rs` (the three distribution errors), and the four inverse-CDF
      imports in `types/sampler/qmc_sampler.rs`. What remains of `rand` there is `Xoshiro256`,
      `rng()`, `SobolSequence` and `MAX_SOBOL_DIM` — entropy only.
- [ ] 5.4 Fix the two measured word-draw sites,
      `uncertain/src/types/sampler/sampler_seed.rs:52-53`, `random::<u64>()` → `random_word()`.
      Do not enter that crate further: `SampledValue`, the sample cache and the lazy graph belong
      to its own renovation.
- [ ] 5.5 Assert the endpoint: a search of `uncertain`'s sources for `deep_causality_rand` returns
      only generator, thread-RNG and Sobol references. Its suite passes at or above baseline.
- [ ] 5.6 Migrate the other distribution consumers' imports: `physics` (6 uses), `discovery` (1).
      Remove the `Distribution` clause from `Normal<F>`'s struct definition.
- [ ] 5.7 **Transition `topology` to `stats` only.** Thirteen `rand` references, counted, and each
      has a destination:
      - [ ] 5.7.a The six `RngType: deep_causality_rand::Rng` bounds in `gauge_field_lattice/`
            and `link_variable/` become the generator trait re-exported from `stats`.
      - [ ] 5.7.b `cubical_regge_geometry/metropolis.rs` imports `Normal`, `StandardUniform` and
            `StandardNormal` from `stats`, and **both leaked `where` clauses are removed** — the
            tier-7 leak this change exists to repair.
      - [ ] 5.7.c The edge pick, `(rng.next_u64() as usize) % num_edges` at
            `metropolis.rs:122`, becomes a `UniformInt` draw from 4H. This is the one operation
            that needed a new distribution, and it is also the one place topology's randomness is
            currently biased.
      - [ ] 5.7.d `RandomField` in `types/gauge/link_variable/random.rs` drops its hand-rolled
            `rng.random::<f64>() - 0.5` for a `stats` draw at the caller's scalar. Its docstring
            currently says it "bridges the gap between `deep_causality_rand` and algebraic types";
            that gap is what this change closes. Add a test that the gauge field draws at `f32` and
            `Float106`, a precision it does not have today.
      - [ ] 5.7.e `LawRng` in `utils_tests/hkt_law_utils.rs` stays unchanged: it is a hand-rolled
            deterministic generator depending on neither crate, and its determinism is the point.
      - [ ] 5.7.f Remove `deep_causality_rand` from `topology`'s manifest; add
            `deep_causality_stats`. Confirm the crate builds and its suite passes at or above
            baseline.
- [ ] 5.8 Assert the endpoint by search: `topology`'s sources contain no `deep_causality_rand`
      reference outside `utils_tests`, and its manifest names none.
- [ ] 5.9 Confirm the entropy-only consumers need **no** edit: `algorithms`, `tensor`,
      `ultragraph`, `data_structures`. Build `quantum` with `--features qpu` for the transitive
      path through `uncertain`.
- [ ] 5.10 Clippy clean across every touched crate. Prepare the group-5 commit message.

## 6. The diagonal traversal in `haft`

Independent of groups 1–5; it closes a gap that exists today for `ZipTensorWitness` and
`ZipDenseVectorWitness` regardless of sampling.

- [ ] 6.1 Phase 1. Declare the traversal in `haft` beside `Traversable`, bounded on `Semigroupal`
      of the inner witness and taking the seed as a parameter. Implement for
      `CausalTensorWitness` and `DenseVectorWitness`. All bodies `unimplemented!()`.
- [ ] 6.2 Phase 2. Suite: a 2×2 field of 50 draws per cell returns **50** fields of shape `[2, 2]`,
      and field *i* holds draw *i* of every cell — **assert exact values, not the count**; the
      empty structure returns the seed; ragged columns truncate the ensemble to the shortest while
      every field retains every cell. Run; record in `notes/tdd-group-6.md`.
- [ ] 6.3 Phase 3. Defect audit: (a) the cartesian applicative in place of the diagonal — must fail
      on the count, 6 250 000 against 50; (b) pairing draw *i* with draw *i+1* — must fail the
      exact-value assertion, which is why that assertion is on values and not counts;
      (c) truncating the field cells instead of the ensemble.
- [ ] 6.4 Phase 4. Implement. Phase 5: mutation testing on the traversal.
- [ ] 6.5 Clippy clean. Prepare the group-6 commit message.

## 7. Example and documentation

- [ ] 7.1 New example `examples/mathematics_examples/composable_multi_math/ensemble_x_lattice_ising/`
      (`main.rs` + `README.md`, matching the seven siblings). The 2D Ising model on a periodic
      lattice: `rand` supplies entropy, `stats` the acceptance draw, `tensor` each lattice and the
      ensemble of replicas, `haft` the map and fold.

      Chosen because it carries a closed-form check, as every example in that folder does:
      Onsager's exact `Tc = 2 / ln(1 + sqrt(2)) ~= 2.2692`. Verified before proposing — a 16×16
      lattice over 2 000 sweeps gives `|m|` of 0.986 at `T = 1.5`, 0.732 at `Tc` and 0.119 at
      `T = 3.5`. The ensemble layer too: `<|m|> = 0.9338 ± 0.0292` over 32 replicas at `T = 2.0`.

      - [ ] 7.1.a The ensemble as nested witnessed containers,
            `CausalTensor<CausalTensor<FloatType>>`; observables by `fmap`, ensemble mean by
            `fold`.
      - [ ] 7.1.b The susceptibility `chi = beta * N * (<m^2> - <m>^2)` through the **diagonal**
            traversal from group 6. The reason is physical: a susceptibility is a fluctuation, so
            it is meaningful only if replica *i*'s observables stay paired. The cartesian traversal
            would combine replica 3's energy with replica 17's magnetisation. Assert against a
            directly computed variance.
      - [ ] 7.1.c Precision per part: the Metropolis draw and the ensemble mean are noise-bound and
            run at `f32`; `chi` near `Tc` is cancellation-bound, since `<m^2>` and `<m>^2` nearly
            coincide there, and needs `f64` or `Float106`. One table, three precisions.
      - [ ] 7.1.d Register in `examples/mathematics_examples/Cargo.toml` as
            `[[example]] name = "ensemble_x_lattice_ising_examples"` and in that crate's
            `BUILD.bazel` with the `srcs` glob and `crate_root` its siblings use.
      - [ ] 7.1.e Add the row to `composable_multi_math/README.md` under "HKT-Only Composition".
      - [ ] 7.1.f Repo example conventions: one `FloatType` alias in `main.rs`, no local lift
            helpers, no raw `f64` except at the display boundary through `lower`.

      Deferred: the same composition over `LatticeGaugeField` and `try_metropolis_sweep`, which
      `topology` already carries — Wilson loops in a U(1) gauge theory. Ising first because its
      exact answer is unambiguous; the gauge version is a capstone once this is proven.

- [ ] 7.2 Document in `stats`: distributions live here because their mathematics does, and a
      lazy sampler is Arrow-shaped rather than witness-shaped. Document in `rand`: this crate
      supplies entropy and makes no distributional claim.
- [ ] 7.3 Update `openspec/notes/unified_math/hkt_gaps.md` §3.4 and `hkt_uncertain.md` B7/B8:
      B7's "six per-type files to collapse" premise is superseded by the crate boundary; B8 is
      closed; `RealRng`'s zero-consumer status is resolved by removal.
- [ ] 7.4 Update `deep_causality_unified_math/README.md`: the Monte Carlo example loses its
      `where StandardUniform: Distribution<S>` clause; the crate table's `rand` and `stats` rows
      state the new division; the tier diagram is unchanged, since `stats -> rand` is downhill.

## 8. Close-out

- [ ] 8.1 `bazel test //...` green across the workspace.
- [ ] 8.2 Coverage on every changed file; any miss explained rather than excused.
- [ ] 8.3 `openspec validate retrofit-sampling-layer --strict`.
- [ ] 8.4 Prepare the final batch commit message, naming the breaking changes for the release
      notes: distributions moving `rand` → `stats`, the `StandardUniform` split, the `Rng::random`
      split, the retyped `f64` surface, and the `RealRng` removal.
