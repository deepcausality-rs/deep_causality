/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Homology — the chain condition, and the Betti identity standing on it.

Rust source: `deep_causality_unified_math/deep_causality_homology/src/traits/chain_complex.rs` (`ChainComplex`, and its provided
`betti_number_over`), over `deep_causality_linear`'s 𝔽₂ elimination.

Why this file exists. `DeepCausalityFormal/Linear/RankNullity.lean` proves `gf2_betti_from_ranks`:
that `(n_k − rank ∂_k) − rank ∂_{k+1}` is `dim H_k`. It proves it *under a hypothesis it never
supplies* —

    hchain : LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin

That hypothesis is the chain condition `∂ₖ ∘ ∂ₖ₊₁ = 0`. Until this file, it was unproved in Lean,
unstated in the Rust trait, and unasserted in the conformance harness: every Betti number the
workspace computes rested on an assumption written down in exactly one place, as an argument to a
theorem. `deep_causality_homology` is where that assumption became an obligation on implementors,
so it is where the discharge belongs.

What is proved. The bridge from a *matrix* identity to the *subspace* inclusion, and then the Betti
identity restated over the matrix identity. The direction matters: `∂ₖ ⬝ ∂ₖ₊₁ = 0` is a statement a
Rust test can check by multiplying two `CsrMatrix<i8>` and comparing with zero, which
`test_the_boundary_of_a_boundary_is_zero` does at every grade of every fixture. The subspace
inclusion is not directly checkable. Proving the implication is what lets the checkable statement
stand in for the one the Betti proof needs.

The carrier is `ZMod 2`, matching `RankNullity.lean`, because the two files compose: the conclusion
here is the hypothesis there, and a mismatch in the coefficient field would make them two unrelated
theorems. `HomologyField::Gf2` is the case where the coefficient ring's being a field is a choice
the caller makes rather than a given.

Rust witness: `deep_causality_unified_math/deep_causality_homology/tests/formalization_lean/chain_condition_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks. Mirror any new import into
`cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those roots
plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.LinearAlgebra.Matrix.ToLin
import Mathlib.LinearAlgebra.Matrix.Rank
import Mathlib.LinearAlgebra.FiniteDimensional.Lemmas
import Mathlib.LinearAlgebra.Dimension.RankNullity
import Mathlib.Algebra.Field.ZMod

namespace DeepCausalityFormal.Homology.ChainCondition

open Module

/-- 2 is prime, which is what makes `ZMod 2` a field rather than only a ring. -/
instance : Fact (Nat.Prime 2) := ⟨Nat.prime_two⟩

/-- 𝔽₂, the field `HomologyField::Gf2` takes its ranks over. -/
abbrev F2 := ZMod 2

/-- A boundary operator `∂ : C_n → C_m`, as the matrix the Rust side stores.

`ChainComplex::boundary_matrix` returns `Cow<'_, CsrMatrix<i8>>`, whose entries are incidence
numbers in `{−1, 0, 1}`. Reduced mod 2 those are the entries here; `-1` and `1` are the same element
of 𝔽₂, which is the definition of the mod-2 chain complex rather than a lossy conversion. -/
abbrev Boundary (m n : ℕ) := Matrix (Fin m) (Fin n) F2

/-- The chain group `C_n` over 𝔽₂: `n` coordinates, one per cell. -/
abbrev Chain (n : ℕ) := Fin n → F2

/-- Rank–nullity over 𝔽₂, restated here so this file composes with `RankNullity.lean` without
importing across namespaces (each `lean_test` target globs one namespace).

This is `linear.gf2.nullity_is_count_minus_rank` in the direction the Rust code relies on it —
subtraction over ℕ, matching `n_k.saturating_sub(rank_k)`. -/
theorem nullity_is_count_minus_rank {m n : ℕ} (d : Boundary m n) :
    finrank F2 (LinearMap.ker d.mulVecLin) = n - d.rank := by
  have h := LinearMap.finrank_range_add_finrank_ker (K := F2) (V := Chain n) (V₂ := Chain m)
    d.mulVecLin
  have : d.rank + finrank F2 (LinearMap.ker d.mulVecLin) = n := by
    simpa [Matrix.rank, Module.finrank_pi] using h
  omega

/-- **The chain condition, as a matrix identity, implies the subspace inclusion.**

`∂ₖ ⬝ ∂ₖ₊₁ = 0 → im ∂ₖ₊₁ ⊆ ker ∂ₖ`.

This is the hypothesis `linear.gf2.betti_from_ranks` takes and does not supply. The left side is
what a Rust test can check — multiply two boundary matrices, compare with zero — and the right side
is what the homology quotient needs in order to be a group at all.

THEOREM_MAP: `homology.chain.dd_zero_implies_range_le_ker` -/
theorem dd_zero_implies_range_le_ker {n_prev n_k n_next : ℕ}
    (dk : Boundary n_prev n_k) (dk1 : Boundary n_k n_next) (h : dk * dk1 = 0) :
    LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin := by
  rintro _ ⟨x, rfl⟩
  -- `Matrix.mulVecLin_mul` is already a `simp` lemma, so the composite
  -- `dk.mulVecLin (dk1.mulVecLin x)` rewrites to `(dk * dk1).mulVecLin x` on its own; `h` sends
  -- that to zero. Passing either explicitly is redundant and the linter says so.
  simp [h]

/-- The `k`-th mod-2 homology group: cycles modulo boundaries.

Stated over the matrix identity rather than the subspace inclusion, so that the hypothesis is the
one the Rust side checks. -/
abbrev HomologyGroup {n_prev n_k n_next : ℕ}
    (dk : Boundary n_prev n_k) (dk1 : Boundary n_k n_next) (_h : dk * dk1 = 0) :=
  (LinearMap.ker dk.mulVecLin) ⧸
    ((LinearMap.range dk1.mulVecLin).comap (LinearMap.ker dk.mulVecLin).subtype)

/-- **The Betti identity, standing on the chain condition rather than assuming it.**

`dim H_k = (n_k − rank ∂_k) − rank ∂_{k+1}`, given `∂ₖ ⬝ ∂ₖ₊₁ = 0`.

`linear.gf2.betti_from_ranks` is this theorem with the subspace inclusion as its hypothesis. This
one takes the matrix identity instead, so every hypothesis it has is one the Rust conformance
harness verifies. What `ChainComplex::betti_number_over` computes — three integers, two of them
ranks the elimination counts and one a cell count — is the dimension of homology, and no kernel and
no quotient is built on the Rust side to get there.

THEOREM_MAP: `homology.chain.betti_from_dd_zero` -/
theorem betti_from_dd_zero {n_prev n_k n_next : ℕ}
    (dk : Boundary n_prev n_k) (dk1 : Boundary n_k n_next) (h : dk * dk1 = 0) :
    finrank F2 (HomologyGroup dk dk1 h) = (n_k - dk.rank) - dk1.rank := by
  have hchain := dd_zero_implies_range_le_ker dk dk1 h
  have hb : finrank F2 ((LinearMap.range dk1.mulVecLin).comap
      (LinearMap.ker dk.mulVecLin).subtype) = dk1.rank :=
    (Submodule.comapSubtypeEquivOfLe hchain).finrank_eq
  have hq := Submodule.finrank_quotient (R := F2)
    (M := (LinearMap.ker dk.mulVecLin))
    ((LinearMap.range dk1.mulVecLin).comap (LinearMap.ker dk.mulVecLin).subtype)
  have hk := nullity_is_count_minus_rank dk
  simpa [HomologyGroup, hb, hk] using hq

end DeepCausalityFormal.Homology.ChainCondition
