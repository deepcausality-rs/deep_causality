<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4F: Poisson, and its domain

## Phases 1, 2 and 4

Declared with `unimplemented!()`; 11 tests observed failing **11 of 11 at the phase-1 body**.
Implemented as Knuth's product: **11 passed, 0 failed**.

## The test found a hang, not a failure

`the_draw_terminates_on_a_degenerate_generator` samples from `MaxRng`, whose every uniform draw is
`1 - 2^-53`. The product decays by that factor per step, so falling below `e^{-4}` would take about
`4 · 2^53 ≈ 3.6e16` iterations.

The test run did not fail. **It never returned**, and the whole suite had to be killed.

That is the hazard the task list anticipated in the abstract — "does **not** loop indefinitely" —
arriving concretely. The fix is a documented iteration cap, `MAX_ITERATIONS = 1_000_000`, after
which the draw returns the count it has reached. A sound generator never approaches it: at the
largest admissible rate the expected count is 501 and the tail decays geometrically, so the cap
costs nothing and converts a hang into a value.

**Two bounds, for two different reasons**, and it is worth keeping them distinct:

| Bound | Guards against | Where |
|---|---|---|
| `MAX_RATE = 500` | `e^{-λ}` underflowing to zero, below which the product can never fall | the constructor, as a refusal |
| `MAX_ITERATIONS` | a degenerate generator whose draws never decay the product | the sampler, as a cap |

`MAX_RATE` is set well below `f64`'s underflow point near 745 rather than at it, so the bound is a
property of the distribution rather than of the scalar it is sampled at — `e^{-500}` is
representable in `f64` and in `Float106` alike, and an `f32` caller is refused before reaching its
own narrower limit.

## Phase 3 — the audit

| # | Defect | Result |
|---|---|---|
| a | `p < l` for `p <= l` | **survived** — equivalent, see below |
| b | off-by-one, `return k + 1` | 6 failures — both mean tests, the variance, `P(K = 0)`, the zero-rate case |
| c | the `MAX_RATE` refusal removed | 1 failure, and it **fails rather than hangs** |

### (a) is equivalent, and the task said to say so either way

The two forms differ only when the product equals the threshold **exactly**. The product is a
running product of uniform draws and the threshold is `e^{-λ}`; exact equality is a measure-zero
coincidence that no generator in this workspace produces.

Both boundary generators were tried. `ZeroRng` drives the product to zero, which is below the
threshold under either comparison. `MaxRng` keeps it above under both. Neither separates them, and
neither could: reaching equality would need a draw contrived to hit one specific double.

Task 4F.4(a) asked for the verdict "either way", and explicitly said not to invent a test that
appears to distinguish them. The verdict is **equivalent**, and no test was added.

### (c) is the argument for having both bounds

Removing the rate refusal makes the out-of-range test fail — cleanly, in 0.11 seconds. Without the
iteration cap the same mutation would have hung the suite, because the sampler would have been
asked for a rate whose threshold underflows. One bound catches the mutation; the other keeps the
catching from becoming a hang.
