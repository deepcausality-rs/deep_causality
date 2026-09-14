<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4A: Exponential, and the shared harness

## The harness

`src/utils_tests/sampling.rs`, extending the crate's existing test-support module rather than
standing up a parallel one. It gives the other six suites: a fixed `SUITE_SEED`, `draws`,
`moments`, `sorted_draws`, `quantile`, `assert_near`, `expect_refused`, `expect_accepted`, `lift`,
and — added during the audit — `ZeroRng` and `assert_guards_against_zero`.

Nothing in it computes an expected value by a second route. The caller supplies the closed form; a
sampler checked against a reimplementation of itself checks neither.

`Exponential::new` returns `Result<Self, StatsError>`, reusing the crate's existing error rather
than adding a seventh error type. `StatsError` already carries `NonPositiveScale`,
`NonFiniteInput`, `NegativeProbability` and `EmptyInput` — exactly the parameter rejections these
distributions need — and `gaussian_log_density` already returns it.

## Phase 1 and 2

`Exponential<T>` declared with `unimplemented!()` bodies; suite of 10 written against it.

```
1 passed, 9 failed
```

All 9 failures are the phase-1 panic. The one pass is the harness determinism check, which is
correct: the harness is implemented, so it must pass while the distribution does not.

## Phase 4

`-ln(u) / λ` for `u` from `Open01`. **11 passed, 0 failed** (10 plus the guard test added below).

## Phase 3 — the audit, and a task-list correction

### (a) was mis-specified, and the measurement says so

The task list called for `-ln(1 - u)/λ` with `u` from a half-open draw, asserting it "must fail
the `ln(u)` guard". It does not fail, and it should not: with `u ∈ [0, 1)`, `1 - u ∈ (0, 1]`, so
the logarithm is always finite. **That form is a valid alternative implementation, not a defect.**

Written and run: 10 passed, 0 failed. The task was wrong, not the suite.

### The real hazard, and it survived

The defect is the other direction: `-ln(u)/λ` with `u` from the **half-open** draw, where `u` can
be zero and the result an infinity.

That survived the whole suite, including the 10 000-draw finiteness check — because a zero draw has
probability `2^-53`, one in nine quadrillion. No sampling test reaches it however many draws it
takes. This is the same shape as group 3's `Open01` guard, and it is worth stating as a pattern:
**an inverse-CDF guard cannot be tested by sampling.**

Closed with `ZeroRng`, a generator whose words are all zero, promoted into the shared harness so
the other six suites get it for free. A correct guard redraws forever there, so the assertion is
that the call does **not** return within a bounded wait. Verified: with the half-open draw in
place, the test fails with "the sampler returned a value from an all-zero generator".

### The audit in full

| # | Defect | Result |
|---|---|---|
| a | `-ln(1-u)` from a half-open draw | **not a defect** — valid alternative, task list corrected |
| a' | `-ln(u)` from a half-open draw | **survived**; closed with `ZeroRng`, now caught |
| b | `-ln(u) * λ` instead of `/λ` | 4 failures — mean, variance, and the rate/scale test |
| c | constructor accepts rate 0 | 1 failure — the non-positive-rate test |

Defect (b) is why the suite tests at rate 2 and rate 4 rather than rate 1: at `λ = 1` the rate and
the scale coincide and a sampler that confuses them passes.

`deep_causality_stats`: **557 passed, 0 failed.**

## Phase 5 — mutation testing

**2 caught, 0 missed, 3 unviable.** No survivors.

The three unviable are mutations of `Exponential::new`'s return that do not typecheck. The two
caught are the sampler's negation and division, both pinned by the mean and rate/scale tests.

A small surface, and the audit above did the load-bearing work: mutation testing confirms nothing
is left unpinned rather than finding something new.
