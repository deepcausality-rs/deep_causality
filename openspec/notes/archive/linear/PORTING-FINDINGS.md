<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# What porting the sparse suite found

Task 2.13, and the real discharge of 4.11.

`deep_causality_linear` **reimplements** `CsrMatrix` and the CG solvers rather than moving the files.
A literal `git mv` would have been faithful by construction; a reimplementation is faithful only if
checked. The crate being replaced ships 120 tests across 12 files, and those tests are the record of
how it behaves. Porting them is the check.

Six divergences, all found this way and none by reading the code.

## 1. The CG signatures were reordered, silently

The worst of them.

```
sparse:  cg_solve_preconditioned(apply, diag_a, b,            tolerance, max_iterations)
linear:  cg_solve_preconditioned(apply, b,      inv_diagonal, max_iterations, tolerance)
                                        ^^^^^^^^^^^^^^^^^^^^  swapped, and both are &[R]
```

Two independent breaks in one signature.

**The order.** `diag_a` and `b` are both `&[R]`, so a repointed caller passing the old order
compiles and preconditions on the right-hand side.

**The meaning.** `diag_a` is the **diagonal of A**, from which the solver forms `M⁻¹ = diag(1/diag_a)`
itself. The replacement took `inv_diagonal`, the **reciprocal**, and applied it directly. Same type,
inverse quantity. A caller handing over a diagonal where a reciprocal is expected computes a
different preconditioner, converges to the same answer more slowly, and reports nothing.

`../../../specs/neumann-poisson/spec.md` names this function normatively, and phase 5 repoints every
caller. Restored to the original signature and semantics, including the rule that a diagonal entry at
or below zero is treated as `1` — no preconditioning on that row, which keeps the preconditioner
positive definite for clipped diagonals.

## 2. `from_triplets` dropped duplicate entries instead of summing them

The original sums duplicate `(row, col)` triplets. The replacement kept both, which is not a CSR
matrix: two entries at one position, and every later read returns whichever the linear scan reaches
first. Fixed, including the case where the sum cancels to zero and the entry becomes structurally
absent.

## 3. There was no `Display`

The original has one. The replacement had none, so `format!("{m}")` stopped compiling.

Restored to the original format character for character — a shape header, then one bracketed row per
line with each entry right-padded to eight columns at three decimal places, and `[Empty]` for a
zero-dimension matrix. Printing the structural zeros costs `rows * cols` rather than the stored
count, which is the old behaviour and is kept.

A rendering that prints the three CSR arrays would be more useful for debugging a sparse structure.
It would also be a different string, and `linear-matrix-representations` requires identical results.

## 4. `new()` and `default()` disagreed on the row-pointer array

The original leaves it empty; the replacement wrote `[0]`. `row_indices()` is public, so this is a
result rather than an implementation detail. Matched to the original.

## 5. An inconsistency inherited on purpose

`new()` gives `[]` and `with_capacity(0, 0, 0)` gives `[0]`, so two zero-shaped matrices compare
**unequal**. Probed against the published crate: this is the original's behaviour, not something the
port introduced.

Reproduced deliberately and pinned by a test that says so. A caller comparing two empty matrices gets
`false` today and must keep getting it. Worth settling when the old surface is retired in phase 5,
where changing it costs nothing.

## 6. The HKT witness had one trait instead of eight

`CsrMatrixWitness` implemented `HKT` alone, which skipped fourteen ported tests.

`Functor`, `Foldable`, `Pure`, `Applicative` and `CoMonad` are now implemented, matching the
original's semantics — map and fold visit the **stored** entries, `pure` builds the 1×1, `extract`
reads the first stored entry, and `extend` rotates each stored position to the front so that
`extend(extract) == id`.

`Monad` and `Adjunction` stay absent. The original claims both and its `bind` violates monad right
identity; `Adjunction::counit` is written in terms of that `bind` and inherits the defect. See
`../../unified_math/HKT-LAW-FINDINGS.md`. Nothing outside the two crates' own tests uses either, so the omission reaches
no consumer.

## 7. Only the owned operator forms existed

The original implements `Add`, `Sub`, `Mul` and the assigning forms for every combination of owned
and borrowed operands. The replacement had the owned forms only, which skipped sixteen ported tests
and would have broken any call site written as `&a + &b` — the common shape, since neither operand is
being consumed.

Closed: `&a + &b`, `a + &b`, `&a + b`, `AddAssign` in both forms, the same four for `Sub`, `-&a`, and
`&a * &b`. The skipped tests are restored and pass.

## The lesson worth keeping

Every one of these is invisible when reading the two implementations side by side. Each is a place
where the replacement is a reasonable design that differs from the thing it replaces, and where the
difference reaches a caller. The suite that came with the original is what caught them.
