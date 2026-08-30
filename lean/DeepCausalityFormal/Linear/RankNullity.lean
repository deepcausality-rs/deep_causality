/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Linear — rank–nullity over 𝔽₂, and the Betti-number identity read off it.

Rust source: `deep_causality_unified_math/deep_causality_linear/src/algorithms/gf2.rs` (`rank_gf2`, `kernel_basis_gf2`) over
`deep_causality_unified_math/deep_causality_linear/src/types/packed_gf2/` (`PackedGf2<W>`), consumed by
`deep_causality_unified_math/deep_causality_topology/src/traits/chain_complex.rs` (`ChainComplex::betti_number_over`) through
`deep_causality_unified_math/deep_causality_topology/src/types/homology_field/mod.rs` (`HomologyField::Gf2`).

Why this statement and not a general one. `betti_number_over` computes

    β_k = (n_k − rank ∂_k) − rank ∂_{k+1}

and the first bracket is `dim ker ∂_k` **only because** rank–nullity holds. The Rust code never
computes a kernel dimension: it subtracts a rank from a cell count and calls the result a nullity.
That substitution is the load-bearing step, it is invisible in the source, and it is what this file
proves — over 𝔽₂ specifically, because `HomologyField::Gf2` is the case where the coefficient
ring's being a field is a choice rather than a given. `ZMod 2` is `Field` in Mathlib exactly as
`Gf2` reaches `Field` through the tower's blanket in `deep_causality_algebra`.

The identity is stated for a matrix rather than an abstract linear map because a matrix is what the
Rust side has: `PackedGf2<W>` is `Matrix (Fin m) (Fin n) (ZMod 2)` with the rows bit-packed, and
`Matrix.rank` is by definition `finrank` of the range of `mulVecLin`, which is the same number the
mod-2 elimination counts pivots to get.

Rust witness: `deep_causality_unified_math/deep_causality_linear/tests/formalization_lean/rank_nullity_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.LinearAlgebra.Matrix.Rank
import Mathlib.LinearAlgebra.FiniteDimensional.Lemmas
import Mathlib.LinearAlgebra.Dimension.RankNullity
import Mathlib.Algebra.Field.ZMod

namespace DeepCausalityFormal.Linear.RankNullity

open Module

/-- 2 is prime, which is what makes `ZMod 2` a field rather than only a ring.

Stated as an instance because Mathlib gates `ZMod p`'s `Field` instance on it. It is the same
condition the Rust side meets structurally: `Gf2` implements `IntegralDomain` by hand and reaches
`Field` through the tower's blanket, which is sound only because 2 is prime. -/
instance : Fact (Nat.Prime 2) := ⟨Nat.prime_two⟩

/-- 𝔽₂, the field `HomologyField::Gf2` takes its ranks over. -/
abbrev F2 := ZMod 2

/-- The chain group `C_n` over 𝔽₂: `n` coordinates, one per cell. -/
abbrev Chain (n : ℕ) := Fin n → F2

/-- A boundary operator `∂ : C_n → C_m`, as the matrix the Rust side stores. -/
abbrev Boundary (m n : ℕ) := Matrix (Fin m) (Fin n) F2

/-- **Rank–nullity over 𝔽₂.**

`rank ∂ + dim ker ∂ = n`, where `n` is the number of columns — the number of `n`-cells.

THEOREM_MAP: `linear.gf2.rank_nullity` -/
theorem gf2_rank_nullity {m n : ℕ} (d : Boundary m n) :
    d.rank + finrank F2 (LinearMap.ker d.mulVecLin) = n := by
  have h := LinearMap.finrank_range_add_finrank_ker (K := F2) (V := Chain n) (V₂ := Chain m)
    d.mulVecLin
  simpa [Matrix.rank, Module.finrank_pi] using h

/-- The substitution `betti_number_over` performs: the nullity it never computes is the cell count
minus the rank it does compute.

This is the identity the Rust code relies on, in the direction it relies on it — subtraction over ℕ
rather than addition, matching `n_k.saturating_sub(rank_k)`.

THEOREM_MAP: `linear.gf2.nullity_is_count_minus_rank` -/
theorem gf2_nullity_is_count_minus_rank {m n : ℕ} (d : Boundary m n) :
    finrank F2 (LinearMap.ker d.mulVecLin) = n - d.rank := by
  have h := gf2_rank_nullity d
  omega

/-- The rank never exceeds the cell count, so the subtraction above never saturates.

`saturating_sub` in the Rust body is a floor that is never reached at this step: the truncation
would silently turn a negative nullity into zero, and there is no negative nullity to turn.

THEOREM_MAP: `linear.gf2.rank_le_cell_count` -/
theorem gf2_rank_le_cell_count {m n : ℕ} (d : Boundary m n) : d.rank ≤ n := by
  have h := gf2_rank_nullity d
  omega

/-- The `k`-th mod-2 homology group: cycles modulo boundaries.

`B` is the image of `∂_{k+1}` read as a subspace *of the kernel of `∂_k`*, which needs
`im ∂_{k+1} ≤ ker ∂_k` — the chain condition `∂_k ∘ ∂_{k+1} = 0`. -/
abbrev Homology {n_prev n_k n_next : ℕ}
    (dk : Boundary n_prev n_k) (dk1 : Boundary n_k n_next)
    (_hchain : LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin) :=
  (LinearMap.ker dk.mulVecLin) ⧸
    ((LinearMap.range dk1.mulVecLin).comap (LinearMap.ker dk.mulVecLin).subtype)

/-- **What `betti_number_over` computes is the dimension of homology.**

`(n_k − rank ∂_k) − rank ∂_{k+1}` — three integers, two of them ranks the elimination counts and
one a cell count — equals `dim H_k`. No kernel and no quotient is ever built on the Rust side; this
is the theorem that says it does not have to be.

THEOREM_MAP: `linear.gf2.betti_from_ranks` -/
theorem gf2_betti_from_ranks {n_prev n_k n_next : ℕ}
    (dk : Boundary n_prev n_k) (dk1 : Boundary n_k n_next)
    (hchain : LinearMap.range dk1.mulVecLin ≤ LinearMap.ker dk.mulVecLin) :
    finrank F2 (Homology dk dk1 hchain) = (n_k - dk.rank) - dk1.rank := by
  have hb : finrank F2 ((LinearMap.range dk1.mulVecLin).comap
      (LinearMap.ker dk.mulVecLin).subtype) = dk1.rank :=
    (Submodule.comapSubtypeEquivOfLe hchain).finrank_eq
  have hq := Submodule.finrank_quotient (R := F2)
    (M := (LinearMap.ker dk.mulVecLin))
    ((LinearMap.range dk1.mulVecLin).comap (LinearMap.ker dk.mulVecLin).subtype)
  have hk := gf2_nullity_is_count_minus_rank dk
  simpa [Homology, hb, hk] using hq

end DeepCausalityFormal.Linear.RankNullity
