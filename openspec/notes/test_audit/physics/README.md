<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Test-suite audit — findings, and the case for a dedicated change

This folder holds the evidence for a change set of its own. It was produced while closing
`unified-math-next` task group 7 (C6), where mutation testing over
`deep_causality_physics` was about to run and the question came up of whether the suite it would be
measured against was worth measuring. It is not, in a way that turned out to be large enough that
folding the repair into that programme would have swamped it.

**Nothing here is a proposal.** It is what was measured, what was proved, what was already repaired
in passing, and what a dedicated change would have to take on.

## Contents

| File | What it is |
|---|---|
| `physics-inventory.md` | The full per-file inventory for `deep_causality_physics`: 1751 tests across 190 files, classified, with the worst files ranked and the calibration recorded |
| `audit_tests.py` | The scanner that produced it. Re-runnable; prints the class counts and writes the per-site inventory to `/tmp/audit_inv.json` |
| `audit_tests_v2.py` | Risk-first scanner for answer-blind observations, optional assertions, unexplained oracles, discarded results, and degenerate fixtures; writes `/tmp/physics_test_audit_v2.json` |
| `agentic-audit-comparison.md` | Rerun results for the current 1766-test tree, differences from the original scanner, manually checked findings, limitations, and repair order |

## The finding in one line

**Nine per cent of the physics suite asserts nothing about the answer, and ninety per cent tests a
single hand-chosen input.**

| Class | Count | Share | Meaning |
|---|---|---|---|
| single-input | 1579 | 90.2% | One hand-chosen input. No loop, no table, no range. |
| cherry-picked | 629 | 35.9% | One input, a magic literal expectation, no stated provenance. |
| tautology | 116 | 6.6% | No assertion says anything about the answer. |
| circular | 49 | 2.8% | The expected value is recomputed in the test from the kernel's own formula. |
| no-assertion | 1 | 0.1% | No assertion at all, and no helper that asserts. |

Counts are after the repairs described below; the audit opened at 120 tautologies.

## The proof that it matters

Claims about test quality are cheap, so this one was made falsifiable. `weber_number`'s body was
replaced with `PropagatingEffect::pure(R::zero())` — **reporting zero for every input in the
world** — and the entire `kernels/fluids/wrappers_tests.rs` suite **passed**. After the rewrite the
same one-line defect **fails** it.

The claim has to be bounded, too. Blunter defects were tried first — Weber losing the square on its
velocity, Knudsen inverted — and the *old* suite caught those. The old tests are not uniformly
worthless. Their failure mode is specific:

* a single hand-chosen input catches an error that moves every value, and misses one the chosen
  input happens to be blind to (`L` against `L²` when the test uses `L = 1`);
* a `.is_ok()` assertion catches nothing whatsoever;
* a circular expectation catches a typo in the kernel but never a misread specification, because the
  test was derived from the same reading.

## Calibration — three false-positive classes removed

The first scan over-reported and was corrected three times, each against real code. This is recorded
because the corrected numbers are the ones to plan from, and because the same mistakes are easy to
repeat:

1. **`assert!(x.is_err())` is a real claim.** An error-path test asserting only a refusal is doing its
   job. First pass: 535 "weak-only". After: 169.
2. **`assert!(!x.is_ok())` is also an error claim.** It matches `.is_ok()` textually and was being
   counted as weak. 169 → 120.
3. **Assertions may live in a helper.** `constants_accessors_f64` calls `check_constants::<f64>()`,
   which asserts. 48 "assertion-free" → 1.

A residual limitation, stated so it is not mistaken for a work list: **single-input** counts any test
without a loop or table, so a test pinning one genuinely singular case — a boundary, a refusal — is
counted beside a value test that should have swept a range. It is a ceiling, not a backlog.

And one class the scanner cannot decide: a *conversion* is not a *re-solve*.
`kernels/astro/solver_convergence_tests.rs` builds its expected position as `a(cos E − e)` from a
root obtained by an independent bisection. The detector flags it as circular. It is not.

## What was already repaired, in passing

So the dedicated change does not redo it:

| File | Before | After |
|---|---|---|
| `kernels/fluids/dimensionless_tests.rs` | 40 tests, 8 circular, 22 cherry-picked, one input each | 32 tests over ~100 oracle rows, 3 cross-kernel identities, 4 scaling laws, every divisor guard, zero-numerator corner cases |
| `kernels/fluids/wrappers_tests.rs` | 119 tests, 80 tautology | the 18 dimensionless wrappers assert the carried value against the oracle, over every table row |
| 8 further `wrappers_tests.rs` files | `assert!(effect.is_ok())` | 36 tautologies converted to delegation assertions against the kernel |

`scripts/physics_oracles.py` was added for this and is the pattern to extend: textbook definitions
evaluated in **50-digit decimal**, so the expectation is nearer the true value than any `f64`
evaluation, and the test measures the kernel's error against the mathematics rather than against a
second copy of itself.

## Why the rest is not mechanical

The transformer that converted those 36 moved the tautology count by four. It only fires where a
wrapper is a true passthrough — `Ok(v) => PropagatingEffect::pure(v)` — and where the test matches
`let x = w(..); assert!(x.is_ok())` exactly. **65 of 170 wrappers re-wrap their result** (into a
`PhysicalField`, say), so the value types differ and the comparison does not even typecheck.

The remainder is therefore family-by-family work: an oracle per kernel family, then table-driven
tests written against it. That is the shape of the dedicated change, and it is why it is one.

## The standard any rewrite should meet

1. **An oracle independent of the implementation** — a cited published value, a value from
   `scripts/physics_oracles.py`, a closed form evaluated by hand, or an invariant the answer must
   satisfy. Never the kernel's formula retyped.
2. **A range, not a point** — table-driven across the physical range, log-spaced where the quantity
   is scale-free, including the extremes the kernel claims to support.
3. **Corner cases named in advance** — zero, one, boundaries, smallest and largest representable
   inputs, and any discontinuity the physics has.
4. **Error paths and improper state** — every documented refusal exercised, every invariant violation
   rejected rather than absorbed.

Cross-quantity identities are the strongest instrument available and should be used wherever the
physics offers them. `Pe = Re·Pr`, `Ra = Gr·Pr` and `Le = Sc/Pr` each relate kernels that share no
code, so no consistently-retyped formula can satisfy them, and an error in any one is caught by the
other two.

## Scope worth considering for the dedicated change

Highest severity first, since these are the tests that catch nothing:

1. **116 tautologies + 49 circular.** Bounded, and the whole of the "asserts nothing" problem.
2. **629 cherry-picked.** The bulk. Needs an oracle per family; the largest piece of work.
3. **1579 single-input.** Overlaps (2) heavily; mostly closed by the same table-driven rewrites.

Only `deep_causality_physics` was scanned. `audit_tests.py` takes a root directory and the same
classes almost certainly exist in the other 29 crates — worth measuring before the scope is fixed
rather than after.
