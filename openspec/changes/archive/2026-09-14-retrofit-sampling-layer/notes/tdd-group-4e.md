<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# 4E: Categorical

## Phases 1, 2 and 4

Declared with `unimplemented!()`; 9 tests observed failing **9 of 9 at the phase-1 body**.
Implemented as a cumulative scan against a uniform draw scaled by the weight total:
**11 passed, 0 failed** after the audit added two.

A design fix found while writing the suite: `len`, `is_empty` and `weights` were behind the
`RealField` bound, so a caller holding a constructed value could not read its shape without proving
something about its scalar. They are accessors and need no algebra; they moved to an unbounded
impl block.

## Phase 3 — the audit

| # | Defect | Result |
|---|---|---|
| a | scan without scaling by the total | 6 failures |
| b | off-by-one, `return i + 1` | 7 failures, several as an in-range panic |
| c | `<=` for `<`, so a zero weight can win | **survived**; closed, now caught |
| d | `unreachable!()` in place of the fallback | **survived**; equivalent, see below |

### (c) survived at probability `2^-53`, and this is now a pattern

`a_zero_weight_is_never_drawn` passes against the defective comparison however many draws it takes.
With `<=`, a zero-weight category wins only when the running remainder is **exactly** zero, which a
real generator produces at `2^-53`.

This is the fourth time in this change that a boundary has turned out to be unreachable by
sampling — after group 3's `Open01` guard, group 3's limb composition, and 4A's `ln(u)` guard. The
pattern is specific enough to name: **a comparison at a boundary of the unit interval cannot be
tested by drawing from the unit interval.** It needs a generator built to land on the boundary.

`ZeroRng` was already in the harness from 4A. `MaxRng` — all-ones words, the upper boundary — was
added here, so the remaining suites have both.

The new test uses weights `[0, 1]`: with a zero draw the remainder starts at zero and the first
weight is zero, so `<` moves past it and `<=` selects it. Verified to fail under the mutant.

### (d) is an equivalent mutant, and the reasoning is worth keeping

The scan's fallback returns the last index if the loop exhausts. Replacing it with
`unreachable!()` survives, because **no draw reaches it**: the remainder starts at `u · total` with
`u < 1`, so it is strictly less than the sum of the weights and some comparison must fire first.
`MaxRng` was tried and does not reach it either — at `u = 1 - 2^-53` the product still rounds below
the total for every weight vector, since `total · 2^-53` is exactly half an ulp and ties round to
even.

So the two forms are indistinguishable by behaviour. The safe return is kept anyway: the argument
for `unreachable!()` is that it documents an invariant, and the argument against it is that if the
invariant is ever broken by accumulated rounding in the subtraction — which is harder to rule out
than the initial product — the cost is a panic in production rather than a valid index. That trade
is not close.

Recorded here rather than in `.cargo/mutants.toml`, because it is a property of this fallback
rather than of a mutation operator.
