<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4H: the discrete uniform, and a premise that did not survive measurement

## Phases 1, 2 and 4

Declared with `unimplemented!()`; 10 tests observed failing **10 of 10 at the phase-1 body**.
Implemented as a multiply-shift with rejection: **12 passed, 0 failed** — the suite grew by two
during the audit, for the reason below.

## The task's premise was wrong, and the audit is what showed it

The task list said defect (a), `w % n`, "must fail the uniformity test at a range of 7". It does
not. Run against a range of 7, `w % 7` passed all ten tests.

The arithmetic says why. Modulo bias is governed by how many whole copies of the range fit in a
word: with `k = ⌊2^64 / n⌋` copies, the low `2^64 mod n` residues get `k+1` chances and the rest
get `k`, so the relative excess is `1/k`. At `n = 7` that is `1/2^61 ≈ 4e-19`. Detecting an effect
that size needs on the order of `10^38` draws.

The premise confused *the bias exists* with *the bias is visible*. It exists at every range that
divides no power of two; it is visible only where the range is an appreciable fraction of the word.

**Two tests were added so the suite detects what it claims to detect:**

| Test | Range | Separates |
|---|---|---|
| `a_range_near_the_word_size_is_uniform` | `12 297 829 382 473 034 411` | modulo from a correct draw |
| `a_word_from_the_short_block_is_redrawn` | the same | a rejection from no rejection |

That range is `2^64 · 2/3`, chosen so that exactly one whole copy plus half a second copy fits in a
word. Modulo then hands the lower half two chances and the upper half one, putting **2/3** of the
mass below `2^64 mod len` where a uniform draw puts **1/2** — a difference 20 000 draws resolve
easily.

The range of 7 kept its test, with its docstring corrected to say what it does prove: a
block-based map that rounds a boundary the wrong way fails at an awkward size, and defects (b) and
(c) below are both caught there.

## Dropping the rejection is invisible to every statistical test

Defect (e) — take the high half of `w · len` and skip the rejection — has a maximum per-index error
of one part in `2^64` at any range. No frequency test at any sample size can see it, including the
one at the biased range.

So it is not tested statistically. `a_word_from_the_short_block_is_redrawn` drives the sampler with
a scripted generator whose first word is `0`, which lies in the short block and must be discarded,
and whose second is `2^63`. The correct sampler returns the index for the second word and has drawn
**two** words; the sampler without rejection returns `0` after **one**. Both facts are asserted.

This is the same shape as the boundary-unreachable-by-sampling cases in 4A and 4E: where a property
sits at probability `2^-64`, a scripted generator is the instrument, not a larger sample.

## Phase 3 — the audit

| # | Defect | Result |
|---|---|---|
| (a) | `w % len`, the hand-written form this replaces | **caught** — 2 tests |
| (b) | off-by-one, returns `1..=len` | **caught** — 8 tests |
| (c) | narrowing, maps onto `len - 1` blocks | **caught** — 5 tests |
| (d) | the `usize` draw written separately, with modulo | **caught** — 1 test |
| (e) | the rejection loop deleted | **caught** — 1 test |
| (f) | rejection boundary `<` written as `<=` | **equivalent** |

(f) is a genuine equivalent mutant rather than a gap. It rejects one additional word out of `2^64`
and redraws it uniformly, so the output distribution is unchanged and the extra draw occurs with
probability `2^-64`. No test can distinguish it and none should be written to try.

(d) is worth noting for what it says about the design rather than the test: the `usize` impl
forwards to the `u64` impl instead of repeating the arithmetic, so the defect had to be introduced
by *un-forwarding* it. Two hand-written copies of a rejection loop are two chances to get the
threshold wrong, and `the_usize_draw_agrees_with_the_u64_draw` is what holds them together if
anyone ever splits them again.

## Why the loop terminates

The rejection probability is `(2^64 mod len) / 2^64`, strictly below `len / 2^64`. For any range a
caller would name — an array index, a lattice edge, a graph vertex — that is far below one draw in
a lifetime of draws, and the loop is expected to run once. At the deliberately hostile range used
by the bias test it is 1/4, still geometric. Unlike 4F's Poisson, no iteration cap is needed:
`words_at_the_top_of_the_range_still_yield_an_index` drives 1 000 draws from a generator pinned to
the top of the word range, where a rejection sampler does its work, and every one returns.
