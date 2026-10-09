/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — the commutation facts behind the Markov condition.

Lorenz (Synthese 200:424, 2022), Definition 3.3, calls a process operator Markov for a graph when
it factorizes into Choi operators that commute pairwise. The definition needs two facts to be usable
as a check. Factors that act on disjoint Hilbert legs commute with no condition, so only factors that
share a leg need testing; this is the freeze check's pair selection. And a product of pairwise
commuting factors does not depend on the order of the factors, so the factorization fixes one
operator. Both are proved here on the pair-indexed model, through the mixed-product law
`(A ⊗ B)(C ⊗ D) = (AC) ⊗ (BD)` for the local `kron`.

Rust witness: `deep_causality_quantum/tests/formalization_lean/qcm_theorem_tests.rs`.

Imports: this file adds no Mathlib import. `List.Perm.prod_eq'` is reachable through
`PartialTrace.lean`'s `Mathlib.LinearAlgebra.Matrix.Trace`.
-/

import DeepCausalityFormal.Quantum.PartialTrace

set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix BigOperators

variable {α β : Type*} [Fintype α] [Fintype β] [DecidableEq α] [DecidableEq β]
variable {R : Type*} [CommRing R]

/-- The mixed-product law for the Kronecker product: `(A ⊗ B)(C ⊗ D) = (AC) ⊗ (BD)`. -/
theorem kron_mul_kron (A C : Matrix α α R) (B D : Matrix β β R) :
    kron A B * kron C D = kron (A * C) (B * D) := by
  funext i j
  simp only [Matrix.mul_apply, kron_apply]
  rw [Fintype.sum_prod_type, Finset.sum_mul_sum]
  refine Finset.sum_congr rfl (fun a _ => Finset.sum_congr rfl (fun b _ => ?_))
  ring

/-- Factors on disjoint legs commute: `(A ⊗ 1)(1 ⊗ B) = (1 ⊗ B)(A ⊗ 1)`.

    THEOREM_MAP: `quantum.markov_commutativity` -/
theorem kron_one_commute_one_kron (A : Matrix α α R) (B : Matrix β β R) :
    kron A 1 * kron 1 B = kron 1 B * kron A 1 := by
  rw [kron_mul_kron, kron_mul_kron, Matrix.mul_one, Matrix.one_mul, Matrix.one_mul,
    Matrix.mul_one]

/-- The product of a pairwise-commuting family does not depend on the order of the factors.

    THEOREM_MAP: `quantum.markov_commutativity` -/
theorem pairwise_commute_prod_perm {M : Type*} [Monoid M] {l l' : List M}
    (hp : l.Perm l') (hc : l.Pairwise Commute) : l.prod = l'.prod :=
  hp.prod_eq' hc

end DeepCausalityFormal.Quantum
