<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4D: Weibull

## A protocol slip, corrected before it mattered

The implementation was written before the suite. That inverts phases 1 and 2, and the protocol's
reason is exactly the one at stake here: a test written after the code tends to encode what the
code does rather than what it should do.

Reverted to `unimplemented!()`, suite written against that surface, observed failing **10 of 10 at
the phase-1 body**, then the implementation restored. The order in the history is now the order the
protocol asks for.

## Phase 4

`λ·(−ln u)^{1/k}` for `u` from `Open01`: **10 passed, 0 failed**.

## The one legitimate oracle in this group

At `k = 1` the Weibull **is** the exponential with rate `1/λ`. That identity is analytic, so
checking one against the other is evidence rather than a sampler agreeing with itself — the only
place in group 4 where a second implementation is a valid reference.

It also separates the two readings of the exponent, which is why it earns its place twice.

## Phase 3 — the audit

| # | Defect | Caught by |
|---|---|---|
| a | exponent inverted, `^k` for `^{1/k}` | 4 failures — both mean tests, the median, and the all-scalars test |
| b | scale applied inside the power | 1 failure — the scale-multiplies-the-mean test |

**Defect (a) passes the `k = 1` identity test.** `^k` and `^{1/k}` agree at `k = 1` and nowhere
else, so a suite that only checked the exponential identity would have missed an inverted exponent
entirely. That is why every moment test here uses `k = 2`, and it is the concrete argument for
testing a distribution at a parameter where its special cases do not coincide — the same reason 4A
tests at rate 2 rather than rate 1 and 4B at a non-zero `μ`.
