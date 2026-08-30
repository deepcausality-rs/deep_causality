<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lean Verification Status — `deep_causality_homology`

Status as of 2026-08-30. This note summarizes the machine-checked formalization of the crate's
chain-complex layer. It is the crate-local view of the program described in
[`openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/archive/causal-algebra/Formalization.md).

## Summary

Two statements are formalized. They were chosen because together they close a hole that had been
open since the Betti-number path was first verified.

`lean/DeepCausalityFormal/Linear/RankNullity.lean` proves `gf2_betti_from_ranks`: that

    β_k = (n_k − rank ∂_k) − rank ∂_{k+1}

is `dim H_k`. It proves it under a hypothesis it never supplies —

```lean
(hchain : LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin)
```

— which is the chain condition `∂ₖ ∘ ∂ₖ₊₁ = 0`. Before this crate existed, that hypothesis was
unproved in Lean, unstated in the `ChainComplex` trait, and unasserted in the conformance harness.
Every Betti number the workspace computed rested on an assumption written down in exactly one place:
as an argument to a theorem.

This crate is where the assumption became an obligation on implementors, so it is where the
discharge belongs.

- **Lean proofs (L1):** one file,
  [`lean/DeepCausalityFormal/Homology/ChainCondition.lean`](../lean/DeepCausalityFormal/Homology/ChainCondition.lean),
  carrying **2 theorems** plus the rank–nullity lemma they stand on. Every theorem is closed —
  **zero `sorry`**. The file is **Mathlib-backed** (`Mathlib.LinearAlgebra.Matrix.ToLin`,
  `Mathlib.LinearAlgebra.Matrix.Rank`, `Mathlib.LinearAlgebra.Dimension.RankNullity`,
  `Mathlib.Algebra.Field.ZMod`), so it is checked by `lake build` — or `bazel test //lean:Homology`
  — not standalone with bare `lean`.
- **Rust witnesses (L2):** one `#[test]` per theorem id under
  [`tests/formalization_lean/chain_condition_tests.rs`](tests/formalization_lean/chain_condition_tests.rs).
  Lean proves ∀ over `Matrix (Fin m) (Fin n) (ZMod 2)`; each witness pins the same statement across
  the nine complexes in `utils_tests::reference_spaces`, at every grade of each.
- **The bridge:** each theorem carries a shared id (`homology.chain.dd_zero_implies_range_le_ker`,
  `homology.chain.betti_from_dd_zero`) recorded in
  [`lean/THEOREM_MAP.md`](../lean/THEOREM_MAP.md) — **2 homology ids, both proved and witnessed**.
  CI (`.github/workflows/formalization.yml`) fails if any Lean id lacks a tagged Rust file or a
  manifest row; this crate is in that check's grep list.

## Why the statement is about matrices and not subspaces

`im ∂ₖ₊₁ ⊆ ker ∂ₖ` is what homology needs and is not something a test can check: it quantifies over
a subspace. `∂ₖ ⬝ ∂ₖ₊₁ = 0` is the same condition as a matrix identity, and a test *can* check that
— multiply two boundary matrices, compare with zero.

So the first theorem is the implication between them, and the second restates the Betti identity
over the checkable side. That is the whole content of the file: it converts an assumption into an
obligation the Rust suite discharges at every grade of every shipped complex.

The witnesses compute the two sides by **different routines**. The hypothesis is established by
forming `∂ₖ ⬝ ∂ₖ₊₁` entry by entry from the sparse matrices; the conclusion comes from
`betti_number_over`, which forms no product and builds no kernel — it subtracts two ranks from a
cell count. Their agreement is evidence rather than an algebraic rearrangement of one computation.

Coefficients widen past `i8` before the product. The entries are incidence numbers in `{−1, 0, 1}`,
but a column of a large complex sums many of them, and an accumulator that wrapped to zero would
report success on a broken complex in release builds.

## The witness discriminates

`test_the_witness_rejects_a_broken_chain_condition` flips one incidence sign of `∂₂` on the
2-sphere and asserts the composite stops vanishing. A theorem whose hypothesis is checked by a test
that cannot fail says nothing about this code. The malformed complex keeps every shape intact and
every rank plausible; only the product changes.

## Model fidelity

The Lean carrier is `ZMod 2`, a `Field` in Mathlib exactly when 2 is prime. The Rust carrier is
`deep_causality_num::Gf2`, which implements `IntegralDomain` by hand and reaches `Field` through the
tower's blanket — sound for the same reason. `Matrix.rank` is `finrank` of the range of
`mulVecLin`, which is the number `rank_gf2` reaches by counting pivots.

The field matches `RankNullity.lean` deliberately: the conclusion here is the hypothesis there, and
a mismatch in the coefficient field would leave two unrelated theorems rather than a composition.

`ChainCondition.lean` restates rank–nullity locally rather than importing
`DeepCausalityFormal.Linear.RankNullity`, because each `lean_test` target globs a single namespace
and a cross-namespace import would not resolve under Bazel.

## What is not formalized, and why

**The characteristic-zero path.** `HomologyField::Rational` runs fraction-free elimination over ℤ,
and both theorems hold over ℚ for the same reasons. The 𝔽₂ case is formalized because it is the one
where the coefficient ring's being a field is a *choice* the caller makes rather than a given, and
because it is the case `RankNullity.lean` already fixed.

**That `C_k` is determined by `n_k`.** It would formalize the decision that `Gf2Chain` identifies
its chain group by `(degree, len)` and carries no complex handle. In Lean `Fin n → F2` depends on
`n` definitionally, so the statement is true by `rfl` — neither load-bearing nor invisible, which is
the bar the rest of this file meets.

**That a given implementor satisfies `∂∘∂ = 0`.** Lean proves what follows *from* the chain
condition. Whether `SimplicialComplex`, `CellComplex<C>` or `LatticeComplex` satisfies it is a
property of their boundary construction, checked by the conformance harness at every grade rather
than proved once.
