/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — classical causal models as diagonal process operators.

A classical conditional table enters the quantum model as a diagonal Choi operator. Diagonal
operators commute pairwise, so every family of diagonal factors meets the Markov condition, and by
`pairwise_commute_prod_perm` its product does not depend on the order of the factors. The Kronecker
product of two diagonal operators is diagonal, so tensoring classical tables stays classical. A
classical model is therefore the special case of a quantum causal model in which the commutation
test always passes.

Rust witness: `deep_causality_quantum/tests/formalization_lean/qcm_theorem_tests.rs`.

Imports: this file adds no Mathlib import.
-/

import DeepCausalityFormal.Quantum.Markov

set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix BigOperators

variable {α β : Type*} [Fintype α] [Fintype β] [DecidableEq α] [DecidableEq β]
variable {R : Type*} [CommRing R]

/-- Diagonal operators commute.

    THEOREM_MAP: `quantum.classical_embedding` -/
theorem diagonal_commute (d₁ d₂ : α → R) : Commute (diagonal d₁) (diagonal d₂) := by
  show diagonal d₁ * diagonal d₂ = diagonal d₂ * diagonal d₁
  rw [diagonal_mul_diagonal, diagonal_mul_diagonal]
  congr 1
  funext i
  exact mul_comm _ _

/-- A family of diagonal factors commutes pairwise, which is the Markov condition. -/
theorem diagonal_pairwise_commute (l : List (α → R)) : (l.map diagonal).Pairwise Commute :=
  List.pairwise_map.2 (List.pairwise_of_forall (fun d₁ d₂ => diagonal_commute d₁ d₂))

/-- The product of diagonal factors does not depend on their order.

    THEOREM_MAP: `quantum.classical_embedding` -/
theorem diagonal_prod_perm {l l' : List (α → R)} (hp : l.Perm l') :
    (l.map diagonal).prod = (l'.map diagonal).prod :=
  pairwise_commute_prod_perm (hp.map diagonal) (diagonal_pairwise_commute l)

/-- The Kronecker product of two diagonal operators is diagonal.

    THEOREM_MAP: `quantum.classical_embedding` -/
theorem kron_diagonal (a : α → R) (b : β → R) :
    kron (diagonal a) (diagonal b) = diagonal (fun p : α × β => a p.1 * b p.2) := by
  funext i j
  obtain ⟨i₁, i₂⟩ := i
  obtain ⟨j₁, j₂⟩ := j
  by_cases h₁ : i₁ = j₁ <;> by_cases h₂ : i₂ = j₂ <;> simp [kron_apply, h₁, h₂]

end DeepCausalityFormal.Quantum
