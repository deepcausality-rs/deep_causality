<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4C: Cauchy — quantiles only

## Phases 1, 2 and 4

Declared with `unimplemented!()`; 10 tests written and observed failing **9 of 10 at the phase-1
body** — the tenth is the marker test, which has no body and correctly passes. Implemented as
`x₀ + γ·tan(π(u − ½))`: **10 passed, 0 failed**.

## The measurement that justifies the prohibition

Task 4C.3(c) asked for a mean-based test run across five seeds, to record whether it is stable.
Run, at 20 000 draws each:

```
Cauchy sample means across five seeds:
  [0.684, -0.603, -0.409, -2.786, 3.651]
  spread = 6.44
```

A distribution with unit variance would put the sample mean within about 0.007 of the truth at this
sample size, so five seeds would span roughly 0.02. These span **6.44** — three hundred times
wider.

The practical consequence is concrete rather than theoretical. A test asserting `|mean| < 1.0` —
which looks like every other moment test in this crate — **passes on three of these seeds and fails
on two**. It would survive review and then fail intermittently where nobody can reproduce it.

That is why this module carries no moment assertion, and why the prohibition is written into the
module's own documentation rather than left as a convention.

## What replaces it

The parameters are identifiable, just not through moments: the location is the median, the scale is
half the interquartile range. Both are recovered from a shifted, scaled instance. A tail test —
`P(|X| > 1) = ½` for the standard Cauchy — separates it from a normal sampler, which puts about 32%
there.

## Phase 3 — the audit, and a second task-list correction

| # | Defect | Result |
|---|---|---|
| a | `tan(π·u)` instead of `tan(π(u − ½))` | **not a defect** |
| b | scale applied additively | 4 failures |
| c | the `π` dropped | 4 failures, including the IQR |
| d | location dropped | 1 failure, the shifted-instance test |

### (a) is not a defect, and the task list said it was

The task predicted that the half-turn offset "leaves the distribution correct but the **median**
wrong, so it is caught by 4C.2". Written and run: **10 passed, 0 failed.**

It is not wrong at all. `tan` has period `π`, and `πU` for `U ~ Uniform[0, 1)` is uniform on
`[0, π)`; the tangent of a uniform over any half-open interval of length `π` is the standard
Cauchy. Both forms sample the same distribution. They differ in which `u` maps to which `x`, which
no distributional test can see and no caller can observe.

This is the **second** predicted defect in this group that turned out not to be one — 4A's
`-ln(1 - u)` was the first. Both were caught by the discipline of actually writing the defect and
running it, rather than reasoning about whether the suite would catch it. A phase-3 audit that
only *reasons* would have recorded two false gaps and possibly added two tests pinning behaviour
that does not matter.
