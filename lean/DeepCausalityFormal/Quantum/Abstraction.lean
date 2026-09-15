/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — abstractions compose (Lorenz & Tull, arXiv:2602.16612, Proposition 17, the exact case).

An abstraction between two compositional models is, on one query, a commuting square of linear
maps on vectorised operators: `τ_out ∘ ⟦Q⟧_low = ⟦Q⟧_high ∘ τ_in`. Over the pair-indexed matrix model
every morphism of the numeric semantics is a matrix, composition is matrix multiplication, and a
square is a matrix equation. Two squares that share their middle level paste to a square for the
composite abstraction, with `τ = τ₂ ∘ τ₁` on both sides: that is Proposition 17 in the exact case,
and it is nothing but associativity of matrix multiplication applied twice.

Rust witness: `deep_causality_quantum/tests/types/abstraction/composition_tests.rs ::
test_exact_links_compose_exactly_on_the_concatenated_code`, where two links of residual zero on
the concatenated `[[4,2,2]]` code compose to a residual-zero square. The approximate law
`ε ≤ ‖τ₂‖_post · ε₁ + ‖τ₁‖_pre · ε₂` that the crate records for non-zero residuals is the crate's
own theorem by the triangle inequality on the same pasting; only the exact case is stated here.

Imports: the sibling `Choi` module already reaches `Matrix` and is in the Bazel cache roots.
-/

import DeepCausalityFormal.Quantum.Choi

set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix

variable {R : Type*} [Semiring R]

/-- **Proposition 17, exact case.** Two commuting abstraction squares paste along their middle
level. `low : ι₁ → o₁`, `mid : ι₂ → o₂` and `high : ι₃ → o₃` are the three queries as matrices on
vectorised operators; `τ₁in, τ₁out` align the low and middle levels and `τ₂in, τ₂out` the middle
and high levels. If `τ₁out * low = mid * τ₁in` and `τ₂out * mid = high * τ₂in`, then the composite
alignment `τ₂ * τ₁` makes the outer square commute:
`(τ₂out * τ₁out) * low = high * (τ₂in * τ₁in)`. -/
theorem abstraction_compose_exact
    {ι₁ ι₂ ι₃ o₁ o₂ o₃ : Type*}
    [Fintype ι₁] [Fintype ι₂] [Fintype ι₃] [Fintype o₁] [Fintype o₂] [Fintype o₃]
    (low : Matrix o₁ ι₁ R) (mid : Matrix o₂ ι₂ R) (high : Matrix o₃ ι₃ R)
    (τ₁in : Matrix ι₂ ι₁ R) (τ₁out : Matrix o₂ o₁ R)
    (τ₂in : Matrix ι₃ ι₂ R) (τ₂out : Matrix o₃ o₂ R)
    (h₁ : τ₁out * low = mid * τ₁in) (h₂ : τ₂out * mid = high * τ₂in) :
    (τ₂out * τ₁out) * low = high * (τ₂in * τ₁in) := by
  calc (τ₂out * τ₁out) * low = τ₂out * (τ₁out * low) := Matrix.mul_assoc _ _ _
    _ = τ₂out * (mid * τ₁in) := by rw [h₁]
    _ = (τ₂out * mid) * τ₁in := (Matrix.mul_assoc _ _ _).symm
    _ = (high * τ₂in) * τ₁in := by rw [h₂]
    _ = high * (τ₂in * τ₁in) := Matrix.mul_assoc _ _ _

/-- The exact case as the crate's report states it: both link residuals zero, so the composite's
residual is zero. Stated on the defects `τ_out * low − high * τ_in`, which is how the Rust check
measures a square, over a ring. -/
theorem abstraction_compose_exact_defect
    {R : Type*} [Ring R]
    {ι₁ ι₂ ι₃ o₁ o₂ o₃ : Type*}
    [Fintype ι₁] [Fintype ι₂] [Fintype ι₃] [Fintype o₁] [Fintype o₂] [Fintype o₃]
    (low : Matrix o₁ ι₁ R) (mid : Matrix o₂ ι₂ R) (high : Matrix o₃ ι₃ R)
    (τ₁in : Matrix ι₂ ι₁ R) (τ₁out : Matrix o₂ o₁ R)
    (τ₂in : Matrix ι₃ ι₂ R) (τ₂out : Matrix o₃ o₂ R)
    (h₁ : τ₁out * low - mid * τ₁in = 0) (h₂ : τ₂out * mid - high * τ₂in = 0) :
    (τ₂out * τ₁out) * low - high * (τ₂in * τ₁in) = 0 := by
  have e₁ : τ₁out * low = mid * τ₁in := sub_eq_zero.mp h₁
  have e₂ : τ₂out * mid = high * τ₂in := sub_eq_zero.mp h₂
  exact sub_eq_zero.mpr (abstraction_compose_exact low mid high τ₁in τ₁out τ₂in τ₂out e₁ e₂)

end DeepCausalityFormal.Quantum
