<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lean Verification Status — `deep_causality_linear`

Status as of 2026-08-25. This note summarizes the machine-checked formalization of the crate's 𝔽₂
layer. It is the crate-local view of the program described in
[`openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/archive/causal-algebra/Formalization.md).

## Summary

One statement is formalized, and it was chosen because it is load-bearing and invisible.

`ChainComplex::betti_number_over` in `deep_causality_topology` computes

    β_k = (n_k − rank ∂_k) − rank ∂_{k+1}

from two ranks this crate's elimination produces and one cell count. It never builds a kernel, never
builds a quotient, and never checks that the first bracket is a nullity — it substitutes
`n_k − rank ∂_k` for `dim ker ∂_k` and moves on. That substitution is rank–nullity, it is the step
that makes the whole homology path correct, and nothing in either crate's source states it.

- **Lean proofs (L1):** one file,
  [`lean/DeepCausalityFormal/Linear/RankNullity.lean`](../lean/DeepCausalityFormal/Linear/RankNullity.lean),
  carrying **4 theorems**. Every theorem is closed — **zero `sorry`**, and `#print axioms` reports
  only `propext`, `Classical.choice` and `Quot.sound` for each. The file is **Mathlib-backed**
  (`Mathlib.LinearAlgebra.Matrix.Rank`, `Mathlib.LinearAlgebra.FiniteDimensional.Lemmas`,
  `Mathlib.LinearAlgebra.Dimension.RankNullity`, `Mathlib.Algebra.Field.ZMod`), so it is checked by
  `lake build` — or `bazel test //lean:Linear` — not standalone with bare `lean`.
- **Rust witnesses (L2):** one `#[test]` per theorem id under
  [`tests/formalization_lean/rank_nullity_tests.rs`](tests/formalization_lean/rank_nullity_tests.rs).
  Lean proves ∀ over `Matrix (Fin m) (Fin n) (ZMod 2)`; each witness pins `PackedGf2<u64>` to the
  same statement across nine matrices — square, wide, tall, zero, full-rank, and the even-weight
  dependency whose rank differs between ℚ and 𝔽₂.

  The witnesses compute the two sides by **different routines**: the rank by elimination
  (`rank_gf2`) and the nullity by counting the columns of `kernel_basis_gf2`. The identity is
  therefore a claim about two independent computations agreeing, not an algebraic rearrangement of
  one.
- **The bridge:** each theorem carries a shared id (`linear.gf2.rank_nullity`,
  `linear.gf2.nullity_is_count_minus_rank`, `linear.gf2.rank_le_cell_count`,
  `linear.gf2.betti_from_ranks`) recorded in [`lean/THEOREM_MAP.md`](../lean/THEOREM_MAP.md) — **4
  linear ids, all proved and witnessed**. CI (`.github/workflows/formalization.yml`) fails if any
  Lean id lacks a tagged Rust file or a manifest row; this crate is in that check's grep list
  because a witness exists for every id it carries.
- **Model fidelity:** the Lean carrier is `ZMod 2`, a `Field` in Mathlib exactly when 2 is prime.
  The Rust carrier is `deep_causality_num::Gf2`, which implements `IntegralDomain` by hand and
  reaches `Field` through the tower's blanket — sound for the same reason. `Matrix.rank` is
  `finrank` of the range of `mulVecLin`, which is the number `rank_gf2` reaches by counting pivots.

## What is not formalized, and why

**The characteristic-zero path.** `HomologyField::Rational` runs fraction-free elimination over ℤ,
and rank–nullity holds over ℚ for the same reason it holds over 𝔽₂ — rank is a fraction-field
notion. The 𝔽₂ case is the one formalized because it is the one where the coefficient ring's being a
field is a *choice* the caller makes rather than a given, and because it is the case G-01 and G-02
were opened for.

**Termination and complexity of the eliminations.** Bounded, mechanical, and covered by the tests.

**The dense decompositions.** QR, the Hermitian eigenvalue sweep and the one-sided Jacobi SVD are
floating-point iterations. Their properties are approximate and quantified over a tolerance, which
is not what this layer is for; they are covered by tests against NumPy references instead.
