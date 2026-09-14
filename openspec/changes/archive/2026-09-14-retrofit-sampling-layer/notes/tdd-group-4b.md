<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4B: LogNormal

## Phases 1, 2 and 4

`LogNormal<T>` declared with `unimplemented!()` bodies; 9 tests written and observed failing
**9 of 9 at the phase-1 body**. Implemented as `exp(μ + σ·Z)`: **9 passed, 0 failed**.

## What the suite pins, and why each choice

- **Mean against `e^{μ + σ²/2}`, median against `e^μ`, and that the two differ.** Confusing them is
  the usual error with this distribution. At `σ = 0.5` they differ by 13%, far outside either
  tolerance, and a third assertion requires the mean to exceed the median by at least 5% — so a
  suite that had them swapped would fail rather than agree with itself.
- **`σ = 0.5`, never 0.** At `σ = 0` the distribution is a point mass and the mean and median
  coincide, so a sampler confusing them would pass.
- **A non-zero `μ` in its own test.** At `μ = 0` a sampler that applies `σ` before the shift is
  indistinguishable from one that applies it after.
- **Strict positivity on every draw, not on a moment.** The support is `(0, ∞)`; a non-positive
  value means the exponential was not applied, which is a structural failure a mean would average
  away.
- **`ln(X) ~ N(μ, σ²)`**, the defining property. A sampler with the right support and the wrong
  shape fails here where positivity would not.

## Phase 3 — the audit

| # | Defect | Caught by |
|---|---|---|
| a | return the underlying normal, no exponentiation | 6 failures, including every positivity draw |
| b | `e^{σ(μ + Z)}` — scale applied before the shift | 2: the log-is-normal test and the non-zero-`μ` median |
| c | `e^μ` — the shift with no noise at all | 4, including both moment tests |

Defect (b) is the one the parameterisation tests exist for. It leaves the distribution log-normal
and the support correct; only the parameters are wrong, and at `μ = 0` it agrees with the correct
sampler on the mean. Only the non-zero-`μ` test and the log-is-normal test separate them.

## Phase 5 — deferred

The mutation run for this module aborted with *"cargo test failed in an unmutated tree, so no
mutants were tested"*. The cause was mine: 4C's phase 1 had already landed, and a phase-1 surface
fails deliberately, so the baseline the tool needs was red.

Mutation testing requires a green tree, which means it cannot overlap a phase-1 declaration for the
next distribution. The remaining runs are batched at the end of group 4, once every distribution is
implemented and the crate is green.
