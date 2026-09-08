<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# C2 — `deep_causality_stats`

## Phase 2 — the suite against the unimplemented surface

351 tests, 330 failing on `not implemented: phase 1: declared, not implemented`. The 21 that passed
are `entropy_config_tests`: the parameter types have real bodies, so there is nothing unimplemented
behind them. Zero passing tests outside that file, which is the check that matters — a suite that
compiled but silently passed would be the failure mode worth catching.

## Phase 3 — defect audit

Ten classes injected into the shipped implementation, suite run, implementation restored.

| Injected defect | Tests failing |
|---|---|
| Bessel's correction dropped (`n − 1 → n`) | 40 |
| Entropy base changed from bits to nats | 54 |
| Zero policy skips at epsilon instead of zero | **0 → 3** |
| Max-shift removed from log-sum-exp | 24 |
| Ridge penalty never reaches the diagonal | 10 |
| Gaussian normalisation halved | 6 |
| Equal-width maximum lands one bin past the end | 11 |
| IRLS convergence test inverted | 3 |
| Pearson covariance becomes a variance | 14 |
| Equal-frequency tie-break takes the higher bin | 2 |

One survived the first pass. The policy tests used entries at `0.0625`, which every precision holds
comfortably, so moving the cutoff from zero to `epsilon` changed nothing. The case that separates
them is `[1, δ]` with `δ = epsilon/1000`: `log2(1) = 0` contributes exactly nothing, so the whole
answer is the `δ` term — keeping it gives a small positive number, dropping it gives exactly zero,
and those are distinguishable however small `δ` is.

## Defects the suites exposed in code the crate does not own

**`Float106::infinity()` and `neg_infinity()` set the low word to `±inf`.** A double-double's low
word is a correction to the high word, and an infinity has none, so `two_sum` evaluated `inf − inf`
and returned a correct high word beside a NaN low word. `is_nan` reads only the high word, so
nothing reported it and every later operation was contaminated. `Div` already carried the guard
with its reasoning written out; `Add` and `Mul` did not. Fixed in `deep_causality_num`, which still
passes 3839/3839.

## Numerical choices the suite forced

**Pearson's denominator.** `sqrt(sxx)·sqrt(syy)` rounds three times and returns
`0.9999999999999998` for `y = 2x + 3`; `(sxx·syy).sqrt()` rounds once and is exact, because for an
affine relation the product is an exact square. But the product leaves the representable range at
magnitudes each factor survives — centred sums near `1e200` square past the maximum, and denormal
ones square to zero, which would then be divided by. The exact form is used where the product is
finite and non-zero and the scaled form elsewhere, so only the extremes pay the extra rounding.

**Equal-frequency binning with ties.** A block of equal values cannot straddle a boundary, so when
it spans several bins one must take it. Neither simple rule works on both shipped cases: the first
rank sends the four copies in `[1, 2, 2, 2, 2, 3]` into bin 0 beside the minimum, giving `5, 0, 1`,
and the midpoint sends the six copies in `[1, 1, 1, 1, 1, 1, 2, 3, 4, 5]` out of bin 0 even though
they contain the minimum. Largest share, ties to the lower bin, satisfies both.

## Three places the suite was right and the implementation was wrong

- A **negative ridge penalty** is accepted while the diagonal survives. `λ = −2` on a design of
  `[2]` gives `4 − 2 = 2` and `β = 6`; only `λ = −4`, which cancels the design, is refused, and the
  vanishing pivot already caught that. The blanket rejection was a rule nothing asked for.
- A **logistic label outside `[0, 1]`** is `NegativeProbability`, not `DimensionMismatch`. A
  logistic label *is* a probability, at both ends.
- A **singular IRLS Hessian** means two different things, and the iteration number separates them.
  On the first pass `β = 0`, every weight is `¼` and `H = ¼XᵀX + λI`, so a singularity is a
  property of the design. Later the weights have moved, and a singularity means they collapsed —
  separation, where the likelihood has no finite maximum.

## Precision

Four scalars, not the three the plan named. `BFloat16` was added because at about two decimal
digits it is the strongest available test that the implementation assumes nothing about how much
precision it has. It is a truncated `f32`: the same eight exponent bits, the mantissa cut from 23
to 7, so `f32`'s reach at a hundred-thousandth of its resolution.

Three fixtures could not carry over, and each is a fact about the type:

- **`cancel_offset` 64, not 100.** The binding constraint is the *sum*. Integers are exact only to
  256 and a three-element fixture sums to about three times the offset, so 100 gives 303 — past
  256, where the spacing is already 2, and the mean is wrong before the variance is reached.
- **No thousand-term reduction.** Once a running sum reaches one, an addend of a thousandth is
  below `epsilon · sum` and vanishes. `BF16.reduction` is `1.0` to say so rather than to state a
  tolerance any implementation could meet.
- **`nudge` 0.25.** The spacing at magnitude ten is `7.8e-2`, so `f32`'s `1e-3` step would not move
  the value.

One tolerance changed on measurement rather than judgement: `F32.zero` went from `1e-6` to `1e-5`.
The residual of an exactly-determined two-by-two solve at `f32` is `1.1920929e-6` — ten ulps — and
an ulp at magnitude ten in `f32` is already `1e-6`, so the old floor sat inside the noise it was
meant to be above.

## The one shared tolerance table

Six suites had declared their own per-precision constants under five different names, three of them
having independently reinvented the same two-row distinction:

| moments | log_sum_exp | logistic | entropy | ridge | shared |
|---|---|---|---|---|---|
| `arith` | `native` | `invariant` | `TOL_EXACT` | `EXACT` | `native` / `solve` |
| `literal` | `decimal` | `literal` | `TOL_IRRATIONAL` | `DECIMAL` | `literal` |

The values disagreed for no stated reason — `f32` ranged from `1e-4` to `4e-6` — and a reader had
no way to tell which was considered correct. The rows are now named for computation kinds rather
than for suites, so nothing needs a local override: ridge's `1e-4` at `f32` was not arbitrary, it
is the `solve` row, and every suite that performs a solve gets it.

Four things stay local because they are not precision facts, each saying so at the site: density's
trapezoid grid error, logistic's fit tolerance and the slack it buys, `log_sum_exp`'s published
constants, and the sample fixtures.
