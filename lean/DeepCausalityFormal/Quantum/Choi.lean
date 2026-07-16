/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — the Choi–Jamiołkowski foundation.

The pinned Mathlib has no Choi–Jamiołkowski / channel layer. This file builds it on the matrix
model: the Choi matrix `J(E)` of a linear channel `E : Mat_A → Mat_B` is `J(E)(i,k)(j,l) =
E(|i⟩⟨j|)(k,l)`, and the action reconstructed from a Choi matrix is `applyChoi J A (k,l) =
∑ i j, A i j · J(i,k)(j,l)`. The keystone is the **Choi–Jamiołkowski isomorphism**: reconstructing
the action from the Choi matrix of a linear channel recovers the channel, `applyChoi (choiOf E) = E`.

Rust witness: `deep_causality_quantum/tests/kernels/channel_tests.rs` (the Choi↔Kraus round-trip and
`apply_choi`/`apply_kraus` agreement).
-/

import DeepCausalityFormal.Quantum.PartialTrace
import Mathlib.LinearAlgebra.Matrix.StdBasis

set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix BigOperators

variable {α β : Type*} [Fintype α] [Fintype β] [DecidableEq α] [DecidableEq β]
variable {R : Type*} [CommRing R]

/-- The action reconstructed from a Choi matrix `J` on `H_A ⊗ H_B`:
    `applyChoi J A (k, l) = ∑ i j, A i j · J (i, k) (j, l)`. -/
def applyChoi (J : Matrix (α × β) (α × β) R) (A : Matrix α α R) : Matrix β β R :=
  fun k l => ∑ i, ∑ j, A i j * J (i, k) (j, l)

/-- The Choi matrix of a linear channel `E : Mat_A → Mat_B`:
    `J(E) (i, k) (j, l) = E(|i⟩⟨j|) (k, l)`, where `|i⟩⟨j|` is the matrix unit. -/
def choiOf (E : Matrix α α R →ₗ[R] Matrix β β R) : Matrix (α × β) (α × β) R :=
  fun p q => E (Matrix.single p.1 q.1 1) p.2 q.2

/-- `applyChoi J` is additive in the state.

    THEOREM_MAP: `quantum.choi.apply_add` -/
theorem applyChoi_add (J : Matrix (α × β) (α × β) R) (A B : Matrix α α R) :
    applyChoi J (A + B) = applyChoi J A + applyChoi J B := by
  funext k l
  simp only [applyChoi, Matrix.add_apply, add_mul]
  rw [← Finset.sum_add_distrib]
  refine Finset.sum_congr rfl ?_
  intro i _
  rw [← Finset.sum_add_distrib]

/-- `applyChoi J` commutes with scalar multiplication of the state.

    THEOREM_MAP: `quantum.choi.apply_smul` -/
theorem applyChoi_smul (J : Matrix (α × β) (α × β) R) (c : R) (A : Matrix α α R) :
    applyChoi J (c • A) = c • applyChoi J A := by
  funext k l
  simp only [applyChoi, Matrix.smul_apply, smul_eq_mul, Finset.mul_sum]
  refine Finset.sum_congr rfl (fun i _ => ?_)
  refine Finset.sum_congr rfl (fun j _ => ?_)
  exact mul_assoc _ _ _

-- The Choi–Jamiołkowski reconstruction isomorphism `applyChoi (choiOf E) = E` (that reconstructing
-- the action from a linear channel's Choi matrix recovers the channel) is the CJ keystone. Its
-- proof over Mathlib's `stdBasisMatrix` module-expansion is deferred; the correspondence is checked
-- numerically by the Rust round-trip witnesses in `channel_tests.rs`
-- (`test_choi_kraus_choi_round_trip`, `test_apply_kraus_and_apply_choi_agree`).

end DeepCausalityFormal.Quantum
