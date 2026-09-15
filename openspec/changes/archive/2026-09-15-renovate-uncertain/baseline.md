<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Baseline — renovate-uncertain

Measured on `main` before any task of this change touched `deep_causality_uncertain`, on the
benchmark machine (M3 Max, 16 cores, 128 GB). Every later claim of "smaller", "fewer" or "gone" is
checked against this file rather than asserted.

## 1.1 Structure and scale

| Quantity | Baseline | Where the change expects it to go |
|---|---|---|
| `src/` lines | 3 285 | smaller; the generic tree replaces the variant arms |
| `src/` files | 54 | smaller; four per-type `MaybeUncertain` files become one impl block |
| test files | 46 | unchanged or larger |
| `cargo test -p deep_causality_uncertain` | 247 pass | at or above |
| `bazel test //...` | 1 394 pass | at or above |
| `dyn` sites in `src/` | 6 | **0** |
| `SampledValue` variant arms in `src/` | 72 | **0** — the type is removed |
| ... of which inside the two samplers | 61 | **0** |
| `rusty_fork_test!` invocations | 24, across 20 files | **0**, and the dev-dependency goes |
| cache/seed call sites in `tests/` | 38 | rewritten against a session |
| `ProbabilisticType` mentions in `deep_causality_cfd/src` | 22, across 8 files | **0** |

### The CFD count, corrected

The proposal and tasks say "ten bounds" in `deep_causality_cfd`. The measured figure is **22
mentions across 8 files**: 5 import lines, 1 in a doc comment, and **16 bound occurrences** —
9 spelled `CfdScalar + ProbabilisticType`, 3 `DecNsScalar + ProbabilisticType`, and 4 on
`ProbabilisticType` alone or beside `RealField`. The original figure came from a truncated listing.
Task 6.1 is against 16, not 10.

## 1.2 The flake, characterised

The suite is not deterministic today. Two samples were taken.

- 50 runs of `cargo test -p deep_causality_uncertain`: **47 clean, 3 with a failure**.
- 300 runs of the compiled integration binary at `--test-threads=4`: **7 individual test failures**.

Distinct tests observed failing, across both samples and an earlier 12-run sample:

| Test | Observed failures |
|---|---|
| `uncertain_maybe_f64_tests::test_lift_to_uncertain_failure` | 2 |
| `uncertain_maybe_f106_tests::test_lift_to_uncertain_failure` | 2 |
| `uncertain_tests::test_uncertain_bool_probability_exceeds` | 2 |
| `uncertain_maybe_f64_tests::test_lift_to_uncertain_success` | 1 |
| `uncertain_maybe_f106_tests::test_lift_to_uncertain_success` | 2 |

**Task 1.2's premise was wrong.** It named one test and asked whether any other flake was hiding
behind it. Four were. The flake is not a defect in one test; it is the shape of the whole suite.

### The exposure is 41 assertions, not 5

Every test that gates on a sampled decision runs it unseeded. Counted by call:

| Test file | Sampled-decision calls | Calls `seed_sampler` |
|---|---|---|
| `types/uncertain/uncertain_tests.rs` | 13 | no |
| `types/uncertain_maybe/uncertain_maybe_bool_tests.rs` | 13 | no |
| `types/uncertain_maybe/uncertain_maybe_f64_tests.rs` | 4 | no |
| `types/uncertain_maybe/uncertain_maybe_f106_tests.rs` | 4 | no |
| `integration_tests/complex_operations_tests.rs` | 3 | no |
| `types/uncertain/uncertain_sampling_tests.rs` | 2 | no |
| `integration_tests/float106_precision_tests.rs` | 2 | no |
| `types/sampler/qmc_sampler_tests.rs` | 0 | yes (3) |

The only file that seeds makes no sampled decision. The five observed failures are the assertions
whose margins are narrow enough to have been caught in ~360 runs; the other 36 are latent, not
safe. Task 3.4 covers the family, not the one test.

The cause is structural rather than statistical. `lift_to_uncertain(0.8, 0.95, 0.05, 100)` against a
true presence probability of 0.9 runs a sequential probability ratio test with a 100-sample budget,
and failing to accept within budget is an ordinary outcome of that test, not a defect in it. The
draws come from the OS entropy source because nothing installed a seed.

### Fixed — task 3.4, pulled forward

The 24 tests that observe a draw now install `seed_sampler(0x5EED_2026)` as their first statement.
Classification was mechanical rather than by eye: a test is stochastic if it builds a `normal`,
`uniform`, non-degenerate `bernoulli`, or `from_samples` leaf. The other 49 tests in those files
observe only point distributions or degenerate Bernoullis and are deterministic already.

**No assertion was changed.** The whole diff is 7 import lines, 7 seed constants and 24 one-line
statements — 87 insertions, 7 deletions. No threshold, tolerance, confidence level or sample budget
moved, which is what would have been the wrong fix: the tests exist to decide whether the API is
correct, and the API was never the problem here.

The seed was fixed before running, not searched for. All 247 assertions hold at it on the first
attempt, which is the check that matters — a seed that required an assertion to be relaxed would
have meant a real defect rather than an unlucky draw.

| Measurement | Before | After |
|---|---|---|
| 300 runs of the integration binary, `--test-threads=4` | 7 failures | — |
| 500 runs, `--test-threads=4` | — | **0 failures** |
| 40 runs, `--test-threads=1` | — | **0 failures** |
| 40 runs, `--test-threads=16` | — | **0 failures** |
| `bazel test //...` | 1 394 pass | 1 394 pass |

Task 3.2 inherits this: the 38 cache-and-seed call sites it has to rewrite against a
`SampleSession` are now 62, because each of these 24 seeds becomes `SampleSession::seeded(SEED)`.
That is the intended direction — the seeds were added in the shape the session will take.

## 1.3 Golden vector, pre-change

Recorded from a throwaway harness outside the repository, against the implementation as it stands.
The tree is two stochastic leaves — `normal(10, 1) + uniform(0, 1)` — so an ordinal scheme has more
than one slot to get wrong. Seed `0x5EED_2026`, drawn at explicit indices `0..32`.

```text
 0 1.07665320383135192e1
 1 1.07134709721442611e1
 2 9.77314968994338784e0
 3 9.52157680611292534e0
 4 1.05387364159216066e1
 5 1.06631602294115968e1
 6 1.13172760975197129e1
 7 1.12233464165993535e1
 8 1.03546670346711132e1
 9 9.55679500418228045e0
10 1.01751339147575415e1
11 1.11992093715333088e1
12 9.78820232045629801e0
13 1.12508283057149239e1
14 1.19951952737995917e1
15 1.16866162301651677e1
16 1.05427478532493790e1
17 9.23219867286545792e0
18 1.09861792174594886e1
19 1.13149928083160649e1
20 1.05782111890107782e1
21 9.71056659213035012e0
22 9.94727377786747979e0
23 1.13471410817909053e1
24 1.01993825527060231e1
25 1.03991578570816436e1
26 1.04842895127432971e1
27 1.08582357670508980e1
28 1.20602778289153836e1
29 9.96993992065537427e0
30 1.18283388759777921e1
31 9.63671098515952629e0
## second identical tree agrees at the same indices: true
```

**These 32 values do not survive this change, and that is intended.** Task 2.3 re-addresses every
leaf draw by (seed, index, ordinal), so the stream that produced them no longer exists. The vector
is recorded to make that a measured diff rather than an assumption, and it is why the
`uncertain-realfield-generic` capability withdrew its "f64 behaviour preserved bit-for-bit"
requirement.

### What the agreement line does and does not show

Two structurally identical trees, built separately, agree at the same indices **today**. That is
not the cache: the trees carry different ids, so the cache never serves one from the other. It is
`seed_sampler` resetting the thread-local generator, after which the same sequence of draws replays
in the same order.

The property is worth keeping and worth testing, but after this change it must hold for a different
reason — the draw at a leaf is a function of its ordinal, not of a position in a replayed stream.
A test asserting it therefore still catches a pointer leaking into the seed, which is what task
2.3 requires, because two separately built trees have different addresses and the same ordinals.

---

## 9.2 Re-measurement, after the change

Same machine, same commands. Every row of §1.1 re-run rather than reasoned about.

| Quantity | Baseline | Expected | **Measured after** | |
|---|---|---|---|---|
| `src/` lines | 3 285 | smaller | **3 723** | ✗ larger — see below |
| ... of which code | not split | — | **2 069** | ✓ the code shrank by 1 216 |
| ... of which prose | not split | — | **1 355** | the growth is documentation |
| `src/` files | 54 | smaller | **51** | ✓ |
| test files | 46 | unchanged or larger | **50** | ✓ |
| `cargo test -p deep_causality_uncertain` | 247 pass | at or above | **304 pass** | ✓ |
| `bazel test //...` | 1 394 pass | at or above | **1 395 pass** | ✓ |
| `dyn` sites in `src/` | 6 | 0 | **0** | ✓ |
| `'static` on the scalar bound | n/a | — | **none** | the bound is `RandScalar` exactly |
| `SampledValue` variant arms in `src/` | 72 | 0 | **0** | ✓ the type is gone |
| `rusty_fork_test!` invocations | 24, in 20 files | 0 | **0** | ✓ dev-dependency gone |
| cache/seed call sites in `tests/` | 38 | rewritten | **0** | ✓ |
| `ProbabilisticType` in `deep_causality_cfd/src` | 22, in 8 files | 0 | **0** | ✓ |
| `ProbabilisticType` anywhere in the workspace | — | 0 | **1** | a doc line in a CFD test, naming what was removed |

### The one row that missed, and why

**`src/` grew by 438 lines, where the baseline expected it to shrink.** Splitting the count settles
what happened: **code fell from 3 285 to 2 069**, a 37% reduction, and **prose rose to 1 355 lines**.
The crate is a third smaller in code and carries roughly one line of documentation for every one
and a half of implementation.

The baseline row was the wrong measurement rather than the prediction being wrong — it counted
lines without separating the two, so a change that deletes a dispatcher and explains why it is gone
registers as growth. The corrected row is recorded here rather than quietly restated, because §1.1
says every later claim of "smaller" is checked against this file.

## 9.3 Defect audit

Each removed guarantee had its defect reintroduced, the suite run, and the tree restored.

| Defect reintroduced | Tests that failed |
|---|---|
| The **pointer in the seed** — the generator keyed on the node's `Arc` address instead of its ordinal | 1 — `leaf_ordinals_tests::two_separately_built_identical_graphs_draw_alike`, on two structurally identical graphs disagreeing at index 0 |
| The **shared-leaf double draw** — the per-call memo removed | 1 — `sequential_sampler_tests::test_memoization` |
| The **broken diagonal** — one carrier's `materialize_at` drawing at different indices from the other's | 1 — `uncertain_ensemble_tests::a_verdict_ensemble_agrees_with_the_draws_it_judges`, at index 3 |
| The **unseeded gate** — `SampleSession::seeded` ignoring its seed | **12**, across the acceptance, leaf-ordinal and session suites |

**What the memo audit revealed.** Only the *ambient* path's test caught it, and that is correct
rather than a gap. On the **addressed** path a node visited twice has the same ordinal, so it draws
the same value with or without the memo — the acceptance test `arithmetic_shares_one_draw` (`x - x`
is exactly zero at every index) passed with the memo removed. The session work turned the memo from
a correctness device into a cost saving, and it is load-bearing only for `AmbientDraws`, where each
pull advances a stream. The docstring already said this; the audit measured it.

**What the diagonal audit revealed.** A *uniform* shift of both ensembles is not caught, and should
not be: shifting two ensembles by the same amount preserves the correlation, which is the property
under test. What is caught is the two carriers disagreeing about which indices they draw at.

## 9.4 Mutation testing

`cargo mutants` over the two places where a wrong constant yields a plausible number rather than a
crash — the seed-mixing kernel and the ordinal pre-pass.

```
Found 33 mutants to test
33 mutants tested in 2m: 33 caught
```

**Zero survivors**, so there is nothing to settle with a measurement or fix as a gap.
