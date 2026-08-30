<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Phase 1 revisions, found by writing the suite

Task 2.15 asks whether the suite needed an addition to the public surface in order to compile. It
did, twice, and both are recorded here rather than folded in silently — that is the point of
observing the failure before implementing.

## 1. The access traits were declared and never implemented

Phase 1 declared `MatrixView`, `RowOps` and `MatrixBuild`, and declared the four containers. It did
not declare `impl MatrixView for DenseMatrix<T>` or any of the other eleven impls. The traits existed
and nothing satisfied them, so no test could call `rows()`, `get()` or `zeros()` on anything.

Reading the declarations, this is invisible: the traits look complete and the containers look
complete. It shows up the moment a caller tries to use one through the other, which is the first
thing the suite does.

**Revision.** `src/types/<container>/ops/mod.rs` for each of the four, carrying the access-trait
impls with `todo!()` bodies. `RowOps` is present for `DenseMatrix` and `PackedGf2` and deliberately
absent for `CsrMatrix`, with the reason in that module's header.

## 2. The law markers reached nothing without the structural operators

`Ring` is `AbelianGroup + MulMonoid + Distributive + Annihilating`, and those supertraits need `Add`,
`Sub`, `Neg`, `Mul`, `Zero` and `One` to be *present as impls*. Phase 1 wrote the markers —
`Associative<Additive>`, `Distributive` and the rest — and stopped there, so `DenseMatrix<f64>: Ring`
did not hold and neither did `Module<f64>`.

This is the same shape as the defect the change already inherited from `deep_causality_sparse`,
where `CsrMatrix` sat at `AbelianGroup` because two markers were missing. Here the markers were
present and the operators were missing. Both directions produce a container the tower cannot see.

**Revision.** The same `ops/mod.rs` files carry `Zero`, `One`, `Add`, `Sub`, `Neg`, `Mul`, `Mul<S>`
and `MulAssign<S>` per container, bounded on the weakest tower trait each needs — `Add` on
`CommutativeSemiring`, `Sub` and `Neg` on `CommutativeRing`, because ℕ has no additive inverses.

## What this says about the phase order

Both findings are exactly what phase 2 is for. A suite written after the implementation would have
been written against whatever the implementation happened to expose, and neither gap would have
appeared as a gap — the tests would simply never have tried.

Neither revision changed a signature the suite had already assumed. The additions are impls of
already-declared traits and already-implied operators, so no test assertion was weakened to
accommodate them.
