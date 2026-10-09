/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — the orthomodular lattice of subspaces.

The Rust `Projection` verdict carrier (`deep_causality_quantum/src/types/verdict/projection.rs`)
computes meet, join and complement on the ranges of projections: the lattice of subspaces of a
finite-dimensional complex Hilbert space, the quantum logic of Birkhoff and von Neumann (Ann. of
Math. 37, 1936). This file proves that lattice orthomodular and not distributive. The lattice is
bounded, the orthocomplement `ᗮ` is an antitone involution with `K ⊓ Kᗮ = ⊥` and `K ⊔ Kᗮ = ⊤`, and
`K₁ ≤ K₂` implies `K₁ ⊔ (K₁ᗮ ⊓ K₂) = K₂`. Distributivity fails on the lines through any two
linearly independent vectors `u`, `v` and through `u + v`; with `|0⟩` and `|1⟩` in `ℂ²` these are
the lines of `|0⟩`, `|1⟩` and `|+⟩`, the triple the Rust carrier documents.

The statements are on subspaces; the Rust carrier stores a projection and works on its range.

Rust witness: `deep_causality_quantum/tests/formalization_lean/orthomodular_tests.rs`.

Imports: `Projection.Submodule` supplies the orthogonal complement and its laws, and adds 169
modules to the Mathlib closure. `PiL2` (`EuclideanSpace`) would add about 6,300 more, so the
distributivity witness is stated for any two linearly independent vectors instead of in `ℂ²`.
Mirror any change into `cache_roots` in `//MODULE.bazel`.
-/

import Mathlib.Analysis.InnerProductSpace.Projection.Submodule

namespace DeepCausalityFormal.Quantum

open Submodule

section Laws

variable {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E] [FiniteDimensional ℂ E]

/-- The lattice of subspaces is bounded, the orthocomplement is an antitone involution that meets
    each subspace in `⊥` and joins it to `⊤`, and the orthomodular law holds.

    THEOREM_MAP: `quantum.verdict.orthomodular` -/
theorem subspace_lattice_orthomodular :
    (∀ K : Submodule ℂ E, ⊥ ≤ K ∧ K ≤ ⊤) ∧
    (∀ K : Submodule ℂ E, Kᗮᗮ = K) ∧
    (∀ K₁ K₂ : Submodule ℂ E, K₁ ≤ K₂ → K₂ᗮ ≤ K₁ᗮ) ∧
    (∀ K : Submodule ℂ E, K ⊓ Kᗮ = ⊥ ∧ K ⊔ Kᗮ = ⊤) ∧
    (∀ K₁ K₂ : Submodule ℂ E, K₁ ≤ K₂ → K₁ ⊔ (K₁ᗮ ⊓ K₂) = K₂) :=
  ⟨fun _ => ⟨bot_le, le_top⟩,
   fun K => K.orthogonal_orthogonal,
   fun _ _ h => Submodule.orthogonal_le h,
   fun K => ⟨K.inf_orthogonal_eq_bot, Submodule.sup_orthogonal_of_hasOrthogonalProjection⟩,
   fun _ _ h => Submodule.sup_orthogonal_inf_of_hasOrthogonalProjection h⟩

end Laws

section NotDistributive

variable {E : Type*} [NormedAddCommGroup E] [InnerProductSpace ℂ E]

/-- The lattice of subspaces is not distributive. For linearly independent `u` and `v`, such as
    `|0⟩` and `|1⟩` in `ℂ²`, take the lines through `u`, `v` and `u + v` (the line of `|+⟩`):
    `K_u ⊓ (K_v ⊔ K_{u+v})` contains `u`, while `(K_u ⊓ K_v) ⊔ (K_u ⊓ K_{u+v}) = ⊥`.

    THEOREM_MAP: `quantum.verdict.orthomodular` -/
theorem subspace_lattice_not_distributive {u v : E}
    (hind : ∀ s t : ℂ, s • u + t • v = 0 → s = 0 ∧ t = 0) :
    (ℂ ∙ u) ⊓ ((ℂ ∙ v) ⊔ (ℂ ∙ (u + v))) ≠
      ((ℂ ∙ u) ⊓ (ℂ ∙ v)) ⊔ ((ℂ ∙ u) ⊓ (ℂ ∙ (u + v))) := by
  have huv : (ℂ ∙ u) ⊓ (ℂ ∙ v) = ⊥ := by
    rw [Submodule.eq_bot_iff]
    rintro x ⟨hxu, hxv⟩
    obtain ⟨a, rfl⟩ := mem_span_singleton.1 hxu
    obtain ⟨b, hb⟩ := mem_span_singleton.1 hxv
    have h0 : a • u + (-b) • v = 0 := by rw [neg_smul, hb, add_neg_cancel]
    rw [(hind a (-b) h0).1, zero_smul]
  have hup : (ℂ ∙ u) ⊓ (ℂ ∙ (u + v)) = ⊥ := by
    rw [Submodule.eq_bot_iff]
    rintro x ⟨hxu, hxp⟩
    obtain ⟨a, rfl⟩ := mem_span_singleton.1 hxu
    obtain ⟨b, hb⟩ := mem_span_singleton.1 hxp
    have h0 : (a - b) • u + (-b) • v = 0 := by
      rw [sub_smul, neg_smul, ← hb, smul_add]
      abel
    have hb0 : b = 0 := neg_eq_zero.1 (hind _ _ h0).2
    have ha : a = b := sub_eq_zero.1 (hind _ _ h0).1
    rw [ha, hb0, zero_smul]
  have hu0 : u ≠ 0 := by
    intro hu
    have := (hind 1 0 (by rw [hu, smul_zero, zero_smul, add_zero])).1
    exact one_ne_zero this
  have hmem : u ∈ (ℂ ∙ u) ⊓ ((ℂ ∙ v) ⊔ (ℂ ∙ (u + v))) :=
    ⟨mem_span_singleton_self _,
      Submodule.mem_sup.2
        ⟨(-1 : ℂ) • v, mem_span_singleton.2 ⟨-1, rfl⟩, u + v, mem_span_singleton_self _, by
          rw [neg_one_smul]
          abel⟩⟩
  rw [huv, hup, sup_bot_eq]
  intro h
  rw [h, Submodule.mem_bot] at hmem
  exact hu0 hmem

end NotDistributive

end DeepCausalityFormal.Quantum
