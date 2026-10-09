/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — no influence from A to D.

Lorenz and Barrett (Quantum 5, 511, 2021; arXiv:2001.07774), Definition 1: for a unitary
`U : A ⊗ B → C ⊗ D`, A has no influence on D when the marginal on D factors through the partial trace
over A, `Tr_C ∘ U = M ∘ Tr_A` for some channel `M : B → D`. This file states it for any linear map
on the pair-indexed model and proves the operational reading: the marginal factors through `Tr_A`
exactly when the D marginal of every product input depends on the A part only through its trace.

The forward direction is `Tr_A (X ⊗ Y) = Tr(X) • Y`. The converse fixes a unit-trace `e`, defines
`M Y := Tr_C (U (e ⊗ Y))`, proves agreement on products, and extends to every input because each
matrix unit on the pair index is a Kronecker product of matrix units.

Rust witness: `deep_causality_quantum/tests/formalization_lean/qcm_theorem_tests.rs`.

Imports: this file adds no Mathlib import.
-/

import DeepCausalityFormal.Quantum.PartialTrace

set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix BigOperators

variable {α β γ δ : Type*}
variable [Fintype α] [Fintype β] [Fintype γ] [Fintype δ]
variable [DecidableEq α] [DecidableEq β] [DecidableEq γ] [DecidableEq δ]
variable {R : Type*} [CommRing R]

/-- The partial trace over the left (`A`) factor: `(Tr_A M) k l = ∑ i, M (i, k) (i, l)`. -/
def partialTraceLeft (M : Matrix (α × β) (α × β) R) : Matrix β β R :=
  fun k l => ∑ i, M (i, k) (i, l)

theorem partialTraceLeft_add (M N : Matrix (α × β) (α × β) R) :
    partialTraceLeft (M + N) = partialTraceLeft M + partialTraceLeft N := by
  funext k l
  simp only [partialTraceLeft, Matrix.add_apply, Finset.sum_add_distrib]

theorem partialTraceLeft_smul (c : R) (M : Matrix (α × β) (α × β) R) :
    partialTraceLeft (c • M) = c • partialTraceLeft M := by
  funext k l
  simp only [partialTraceLeft, Matrix.smul_apply, smul_eq_mul, Finset.mul_sum]

/-- `Tr_A (X ⊗ Y) = Tr(X) • Y`. -/
theorem partialTraceLeft_kron (X : Matrix α α R) (Y : Matrix β β R) :
    partialTraceLeft (kron X Y) = Matrix.trace X • Y := by
  funext k l
  simp only [partialTraceLeft, kron_apply, Matrix.trace, Matrix.diag, Matrix.smul_apply,
    smul_eq_mul, Finset.sum_mul]

/-- The partial trace over `A` as a linear map. -/
def partialTraceLeftₗ : Matrix (α × β) (α × β) R →ₗ[R] Matrix β β R where
  toFun := partialTraceLeft
  map_add' := partialTraceLeft_add
  map_smul' := partialTraceLeft_smul

theorem kron_add_right (X : Matrix α α R) (Y Y' : Matrix β β R) :
    kron X (Y + Y') = kron X Y + kron X Y' := by
  funext i j
  simp only [kron_apply, Matrix.add_apply, mul_add]

theorem kron_smul_right (X : Matrix α α R) (c : R) (Y : Matrix β β R) :
    kron X (c • Y) = c • kron X Y := by
  funext i j
  simp only [kron_apply, Matrix.smul_apply, smul_eq_mul]
  ring

theorem kron_smul_left (c : R) (X : Matrix α α R) (Y : Matrix β β R) :
    kron (c • X) Y = c • kron X Y := by
  funext i j
  simp only [kron_apply, Matrix.smul_apply, smul_eq_mul]
  ring

/-- `Y ↦ X ⊗ Y` as a linear map. -/
def kronLeftₗ (X : Matrix α α R) : Matrix β β R →ₗ[R] Matrix (α × β) (α × β) R where
  toFun := kron X
  map_add' := kron_add_right X
  map_smul' := fun c Y => kron_smul_right X c Y

/-- A matrix unit on the pair index is the Kronecker product of two matrix units. -/
theorem single_pair_eq_kron (i j : α) (k l : β) :
    Matrix.single (i, k) (j, l) (1 : R) = kron (Matrix.single i j 1) (Matrix.single k l 1) := by
  funext a b
  obtain ⟨a₁, a₂⟩ := a
  obtain ⟨b₁, b₂⟩ := b
  simp only [kron_apply, Matrix.single_apply, Prod.mk.injEq]
  by_cases h₁ : i = a₁ <;> by_cases h₂ : j = b₁ <;> by_cases h₃ : k = a₂ <;>
    by_cases h₄ : l = b₂ <;> simp [h₁, h₂, h₃, h₄]

/-- A has no influence on D under `U`: the marginal on D factors through the partial trace over A. -/
def NoInfluence (U : Matrix (α × β) (α × β) R →ₗ[R] Matrix (γ × δ) (γ × δ) R) : Prop :=
  ∃ M : Matrix β β R →ₗ[R] Matrix δ δ R, ∀ ρ, partialTraceLeft (U ρ) = M (partialTraceLeft ρ)

/-- No influence from A to D holds exactly when the D marginal of every product input depends on
    the A part only through its trace.

    THEOREM_MAP: `quantum.no_influence` -/
theorem noInfluence_iff [Nonempty α]
    (U : Matrix (α × β) (α × β) R →ₗ[R] Matrix (γ × δ) (γ × δ) R) :
    NoInfluence U ↔
      ∀ (X X' : Matrix α α R) (Y : Matrix β β R), Matrix.trace X = Matrix.trace X' →
        partialTraceLeft (U (kron X Y)) = partialTraceLeft (U (kron X' Y)) := by
  constructor
  · rintro ⟨M, hM⟩ X X' Y h
    rw [hM, hM, partialTraceLeft_kron, partialTraceLeft_kron, h]
  · intro h
    obtain ⟨i₀⟩ := ‹Nonempty α›
    let e : Matrix α α R := Matrix.single i₀ i₀ 1
    have he : Matrix.trace e = 1 := Matrix.trace_single_eq_same i₀ (1 : R)
    let M : Matrix β β R →ₗ[R] Matrix δ δ R := partialTraceLeftₗ.comp (U.comp (kronLeftₗ e))
    have hM : ∀ Y, M Y = partialTraceLeft (U (kron e Y)) := fun _ => rfl
    have hprod : ∀ (X : Matrix α α R) (Y : Matrix β β R),
        partialTraceLeft (U (kron X Y)) = M (partialTraceLeft (kron X Y)) := by
      intro X Y
      have htr : Matrix.trace X = Matrix.trace (Matrix.trace X • e) := by
        rw [Matrix.trace_smul, he, smul_eq_mul, mul_one]
      rw [partialTraceLeft_kron, map_smul, hM, h X (Matrix.trace X • e) Y htr, kron_smul_left,
        map_smul, partialTraceLeft_smul]
    refine ⟨M, fun ρ => ?_⟩
    have hL : (partialTraceLeftₗ.comp U) ρ = (M.comp partialTraceLeftₗ) ρ := by
      rw [Matrix.matrix_eq_sum_single ρ]
      simp only [map_sum]
      refine Finset.sum_congr rfl (fun p _ => Finset.sum_congr rfl (fun q _ => ?_))
      have hs : Matrix.single p q (ρ p q) = ρ p q • Matrix.single p q (1 : R) := by
        rw [Matrix.smul_single, smul_eq_mul, mul_one]
      rw [hs, map_smul, map_smul]
      congr 1
      obtain ⟨i, k⟩ := p
      obtain ⟨j, l⟩ := q
      rw [single_pair_eq_kron]
      exact hprod _ _
    exact hL

end DeepCausalityFormal.Quantum
