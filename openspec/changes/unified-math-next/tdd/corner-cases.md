<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Phase 2: the corner-case enumeration

Every stage fills a copy of this before its suite is judged complete. Enumerating in advance is what
separates a corner case that was considered from one that was found through a defect.

`AGENTS.md` carries the worked example. The topological-charge normalisation in
`deep_causality_topology` was eight times too small. The line computing it ran 513 times across the
suite. Every test used the identity gauge field, where the field strength is zero, so `q == 0` held
for any constant whatsoever. Four true assertions, full coverage, and a wrong answer that survived
until someone compared the docstring with the code.

That failure has a shape, and it is the reason for row C below.

## How to use this

Copy the table into the stage's task notes. For each row, either name the test that covers it or
write **n/a** with the reason the stage's inputs cannot produce that case. An empty cell is not an
answer; it means the enumeration is not finished.

A row is covered when a test *exercises* the case, not when a test merely accepts it. "Returns an
error" is coverage. "Does not panic" is coverage only where not panicking is the specified behaviour.

| # | Class | What it catches | Test or n/a + reason |
|---|---|---|---|
| A | Empty input | A loop that never runs, a fold that returns its identity as though it were an answer, a division by a zero count | |
| B | Single element | An off-by-one that only shows at the boundary; a variance that divides by `n - 1` | |
| C | **Two distinct quantities coincide** | The identity matrix, a diagonal, a symmetric matrix, a tie, a uniform distribution, a zero field. Several different formulas agree here, so a wrong one passes | |
| D | **An index expression degenerates** | A 2×2 has one off-diagonal entry at `j = 0`, where `a[i*n + j]` and `a[i*n - j]` are the same number. Also 1×n, n×1, and any size where a stride collapses | |
| E | Each documented threshold, both sides | A comparison that should be `<` written `<=`; a clamp that fires one step early or late. Test at the threshold, just below, and just above | |
| F | Zero | A guard that skips zero when it should include it, or divides by it | |
| G | Negative | A `sqrt`, `ln`, `powf(1/3)` or probability that has no meaning below zero, and either errors or silently returns `NaN` | |
| H | Exact domain boundary | A probability of exactly 0 or exactly 1; an angle at ±π; a value at the type's representable extreme | |
| I | Non-finite, where the type admits it | `NaN` and ±∞ reaching a comparison, a fold, or a tolerance check | |
| J | Overflow and underflow reach | A quantity representable in the type whose intermediate is not — the `vector_norm_l2` failure. Test at the type's own extremes, not at a fixed constant | |
| K | **Every precision-generic path at `f32`, `f64` and `Float106`** | A tolerance tuned to `f64` that is unreachable at `f32` and wastefully loose at `Float106`; an algorithm whose error is a different order at each | |

## Two rules that decide whether a row is really covered

**Never pin a quantity only where it vanishes.** If a test asserts a computed value is zero, the
suite must also assert that value at an input where it is non-zero and independently known.
Otherwise a wrong factor, a wrong sign, or a dropped normalisation all still pass — which is exactly
what happened to the topological charge.

**Never let one input serve two rows.** A uniform distribution is row C, and it is not also row H,
because at a uniform distribution the probabilities are neither 0 nor 1. Reusing it for both leaves
H untested while the table says otherwise.

## Per-stage minima

These are the cases each stage's inputs are already known to admit. They are a floor, not the whole
enumeration.

**C1, Meek.** The empty graph; one vertex; two vertices; a graph with no undirected edge; a graph
with no directed arc; a complete graph; a graph where a rule fires on the last edge of a pass so a
second pass is required; a chordless four-cycle for the chordality check; a PDAG where both
directions of one edge are compelled; and the R4 witness from the search if one exists.

**C3, `cbrt` and `ToPrimitive`.** Exact cubes at each precision; a negative argument, which is the
case `powf(1/3)` returns `NaN` for; a value near the type's maximum and near its minimum positive;
negative zero; a non-finite input; and, for the dual, a zero real part with a non-zero dual part,
where the derivative is infinite.

**C2, statistics.** A uniform distribution, which is row C and where entropy is `log2 n`; a
degenerate distribution with one outcome at probability 1, where entropy is exactly 0; an entry that
is positive but below epsilon, which the two zero policies treat differently; a negative entry; an
empty distribution; a constant column for binning; a value exactly on a bin boundary; the maximum
value, which must land in the last bin; a one-element sample for the Bessel-corrected variance; a
constant input for Pearson, where the variance is zero; a rank-deficient design for ridge at zero
penalty; and perfectly separable data for the logistic fit.

**C5, the verdict carrier.** `bottom` and `top` at each scalar; complement involution; absorption and
De Morgan at the lattice bounds; and a value at each end of the unit interval.

**C4, `linear` adoption.** A vector with a component near the type's maximum and one near its
minimum positive, at all three precisions; a zero vector; a one-element vector; a `CsrMatrix` with an
entirely empty row; an index outside the shape; and the out-of-range column that the MHD matvec
currently skips in silence.

**C6, root finding.** An interval whose endpoints share a sign; a degenerate interval where the two
endpoints are equal; a root lying exactly on an endpoint; a zero derivative at an iterate; a start
from which Newton diverges; and a function that cannot converge within the iteration cap.
