<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Phase 3: the defect audit

A suite that passes on correct code has proved nothing about itself. Before implementation begins,
each stage introduces deliberate defects into a throwaway implementation and confirms the suite
rejects each one.

This is not mutation testing, and it does not replace it. Phase 3 runs **before** phase 4, on defects
chosen because this change already knows they are reachable in the code being written. Phase 5 runs
after, mechanically, on whatever `cargo mutants` can reach. Phase 3 catches the defect you can name;
phase 5 catches the one you could not.

## How to use this

Write a throwaway implementation — it exists only for this audit and is discarded before phase 4
begins. Introduce each defect **one at a time**, run the suite, and record which test failed.

Two acceptance conditions, and the second is the one that gets skipped:

1. At least one test fails.
2. **The failing test's subject is the defective behaviour.** A test failing incidentally — because
   the defect corrupted a shared fixture, or because an unrelated assertion happened to depend on the
   value — does not count. That test would not have caught the defect on its own.

If a defect leaves the suite green, or is caught only incidentally, add a test that rejects it
directly and repeat the audit from the start.

## The classes

| # | Defect | Introduce it by | Catches a suite that |
|---|---|---|---|
| 1 | Off-by-one in an index or loop bound | `..n` → `..n-1`, or `..=n`; a start index shifted by one | Only tests sizes where the last element does not matter |
| 2 | Flipped comparison | `<` → `<=`, `>` → `>=`, or the operands swapped | Never tests exactly at a threshold |
| 3 | Inverted sign | Negate a term, or drop a leading `-` | Only asserts magnitudes, or only tests where the quantity is zero |
| 4 | Changed constant factor | Halve or double a coefficient; `3` → `2` in a derivative denominator | Only asserts a value is zero, or only checks a ratio that cancels the factor |
| 5 | Dropped normalisation | Remove a division by a sum, a count, or a norm | Only tests already-normalised input |
| 6 | Loosened tolerance | Widen an epsilon by several orders, or replace an exact check with an approximate one | Asserts only that a result is "close" with no case that distinguishes right from nearly right |
| 7 | Skipped case | Add an early `return` before a branch; drop an arm | Never reaches that branch, so coverage is the tell |
| 8 | Removed guard | Delete a validity check and let the computation proceed | Never offers the invalid input |
| 9 | **A plausible neighbour returned** | Return an input unchanged, return the first element, return the identity, return `bottom` — never a panic | Asserts only that a call succeeds, or that a result is finite |

Class 9 is the one that matters most and is the easiest to under-test. The failures this change
exists to fix are all of that shape: an unconverged iterate returned as a root, an under-oriented
PDAG returned as a CPDAG, `1.0` substituted for `0.5`, `inf` returned for a representable norm. Each
is a plausible value that type-checks and reads as success.

## Per-stage defects

Each stage runs the nine classes wherever its mathematics admits them, plus these, which are named
because the code is already known to be able to have them.

**C1, Meek.** Drop R4 entirely. Drop R3. Orient the compelled edge in the wrong direction. Omit the
non-adjacency check in each rule separately — R1's `c ≁ b`, R3's `c ≁ d`, R4's `b ≁ d`. Terminate
after one pass instead of at a fixpoint. Accept a non-chordal component. Confuse the two closures, so
the R1–R3 entry point applies R4.

**C3, `cbrt`.** Drop the sign handling so a negative argument returns `NaN` — this is the defect the
retired workaround actually has. Replace the derivative denominator's `3` with a `2`. Square instead
of cube-rooting inside the derivative. Return the argument unchanged. Guard the singularity at zero
and return a finite value, which is the silent-substitution failure this change exists to remove.

**C2, statistics.** Drop Bessel's correction. Change the entropy base. Skip at epsilon where the
policy says exactly zero, and the reverse. Remove the max-shift from log-sum-exp. Drop the ridge
penalty term. Halve the Gaussian normalisation. Place the maximum one bin past the end. Invert the
IRLS convergence test. Return the last iterate instead of a non-convergence error.

**C5, the verdict carrier.** Swap meet and join. Return `p` instead of `1 − p`. Exchange bottom and
top. Clamp the complement to the wrong bound.

**C4, `linear`.** Drop the scaling factor from the norm. Scale by the smaller component instead of
the larger. Omit the zero-maximum guard. Return the squared norm without the square root.

**C6, root finding.** Return the last iterate on non-convergence — the defect that already exists in
the electroweak solver. Invert the bracket check. Halve toward the wrong endpoint. Drop the
derivative from Newton's update. Compare against the wrong tolerance, or against an absolute
tolerance where the specification says relative.

## Recording the result

The stage's task notes record, per defect: what was introduced, which test failed, and whether that
test's subject was the defective behaviour. A defect that needed a new test records that too — the
new test is the audit's output, and it is the reason the audit was worth running.
