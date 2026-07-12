/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quantum — the partial-trace foundation.

The pinned Mathlib (v4.15.0) has `Matrix`, `trace`, and the Kronecker product, but **no partial
trace**. Partial trace is the load-bearing operation of every quantum theorem in scope
(`openspec/changes/add-quantum-crate`), so this file builds it from first principles on the
pair-indexed matrix model of a bipartite operator on `H_A ⊗ H_B` and proves its defining lemma
library: linearity, the product identity `Tr_B(X ⊗ Y) = Tr(Y) • X`, and the bimodule law
`Tr_B((Z ⊗ 1_B) · M) = Z · Tr_B(M)` (the Q-PTP boundary identity).

The Kronecker product is defined locally (`kron`) rather than imported, matching
`Matrix.kroneckerMap (· * ·)`, to keep the module's dependency footprint minimal.

Rust witness: `deep_causality_quantum/tests/kernels/operator_linalg_tests.rs` (the `partial_trace`
property tests) and the migrated operator layer.
-/

import Mathlib.LinearAlgebra.Matrix.Trace
import Mathlib.Data.Matrix.Mul

-- The pair-indexed matrix model leaves some section instances unused per-lemma; this is
-- the standard Mathlib idiom for that situation and keeps the proofs uncluttered.
set_option linter.unusedSectionVars false

namespace DeepCausalityFormal.Quantum

open Matrix BigOperators

variable {α β : Type*} [Fintype α] [Fintype β] [DecidableEq α] [DecidableEq β]
variable {R : Type*} [CommRing R]

/-- The Kronecker (tensor) product of two square matrices on the pair index, matching
    `Matrix.kroneckerMap (· * ·)`: `(X ⊗ Y) (i, k) (j, l) = X i j * Y k l`. -/
def kron (X : Matrix α α R) (Y : Matrix β β R) : Matrix (α × β) (α × β) R :=
  fun i j => X i.1 j.1 * Y i.2 j.2

@[simp]
theorem kron_apply (X : Matrix α α R) (Y : Matrix β β R) (i j : α × β) :
    kron X Y i j = X i.1 j.1 * Y i.2 j.2 := rfl

/-- The partial trace over the right (`B`) factor of a bipartite operator on `H_A ⊗ H_B`,
    represented as a matrix indexed by pairs: `(Tr_B M) i j = ∑ k, M (i, k) (j, k)`. -/
def partialTraceRight (M : Matrix (α × β) (α × β) R) : Matrix α α R :=
  fun i j => ∑ k, M (i, k) (j, k)

@[simp]
theorem partialTraceRight_apply (M : Matrix (α × β) (α × β) R) (i j : α) :
    partialTraceRight M i j = ∑ k, M (i, k) (j, k) := rfl

/-- Partial trace is additive.

    THEOREM_MAP: `quantum.partial_trace.add` -/
theorem partialTraceRight_add (M N : Matrix (α × β) (α × β) R) :
    partialTraceRight (M + N) = partialTraceRight M + partialTraceRight N := by
  funext i j
  simp [partialTraceRight, Matrix.add_apply, Finset.sum_add_distrib]

/-- Partial trace commutes with scalar multiplication.

    THEOREM_MAP: `quantum.partial_trace.smul` -/
theorem partialTraceRight_smul (c : R) (M : Matrix (α × β) (α × β) R) :
    partialTraceRight (c • M) = c • partialTraceRight M := by
  funext i j
  simp [partialTraceRight, Matrix.smul_apply, Finset.mul_sum]

/-- The product identity: the partial trace of a Kronecker product factorizes,
    `Tr_B(X ⊗ Y) = Tr(Y) • X`.

    THEOREM_MAP: `quantum.partial_trace.kronecker` -/
theorem partialTraceRight_kron (X : Matrix α α R) (Y : Matrix β β R) :
    partialTraceRight (kron X Y) = (trace Y) • X := by
  funext i j
  simp only [partialTraceRight, kron_apply, Matrix.smul_apply, smul_eq_mul, Matrix.trace,
    Matrix.diag_apply, Finset.sum_apply]
  rw [Finset.sum_mul]
  refine Finset.sum_congr rfl ?_
  intro k _
  exact mul_comm _ _

/-- The bimodule (boundary) law — the Q-PTP hypothesis: when the exterior neighbour acts only on
    the traced-out interior as `Z ⊗ 1_B`, the partial trace intertwines with left multiplication,
    `Tr_B((Z ⊗ 1_B) · M) = Z · Tr_B(M)`.

    THEOREM_MAP: `quantum.partial_trace.bimodule` -/
theorem partialTraceRight_bimodule (Z : Matrix α α R) (M : Matrix (α × β) (α × β) R) :
    partialTraceRight ((kron Z (1 : Matrix β β R)) * M) = Z * partialTraceRight M := by
  funext i j
  simp only [partialTraceRight, Matrix.mul_apply, Fintype.sum_prod_type, kron_apply,
    Matrix.one_apply]
  -- LHS: ∑ k, ∑ i', ∑ k', Z i i' * (if k = k' then 1 else 0) * M (i', k') (j, k)
  -- Swap the outer k and i' sums, collapse the `k = k'` delta.
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl ?_
  intro i' _
  rw [Finset.mul_sum]
  refine Finset.sum_congr rfl ?_
  intro k _
  rw [Finset.sum_eq_single k]
  · simp [mul_comm, mul_assoc]
  · intro k' _ hk'
    simp [Ne.symm hk']
  · intro h
    exact absurd (Finset.mem_univ k) h

/-- The right bimodule law: `Tr_B(M · (Z ⊗ 1_B)) = Tr_B(M) · Z`.

    THEOREM_MAP: `quantum.partial_trace.bimodule_right` -/
theorem partialTraceRight_bimodule_right (Z : Matrix α α R) (M : Matrix (α × β) (α × β) R) :
    partialTraceRight (M * (kron Z (1 : Matrix β β R))) = partialTraceRight M * Z := by
  funext i j
  simp only [partialTraceRight, Matrix.mul_apply, Fintype.sum_prod_type, kron_apply,
    Matrix.one_apply]
  rw [Finset.sum_comm]
  refine Finset.sum_congr rfl ?_
  intro j' _
  rw [Finset.sum_mul]
  refine Finset.sum_congr rfl ?_
  intro k _
  rw [Finset.sum_eq_single k]
  · simp [mul_assoc]
  · intro k' _ hk'
    simp [hk']
  · intro h
    exact absurd (Finset.mem_univ k) h

/-- **Boundary preservation (Q-PTP).** When a boundary operator `Z ⊗ 1_B` — one that acts trivially
    on the traced-out interior — commutes with `M`, its `A`-part `Z` commutes with the partial trace
    `Tr_B(M)`. This is the *conditional* preservation the roadmap promised: partial trace sends the
    commutator to the commutator **under the boundary hypothesis**, via the two bimodule laws.

    THEOREM_MAP: `quantum.partial_trace_preservation_boundary` -/
theorem partial_trace_preservation_boundary (Z : Matrix α α R)
    (M : Matrix (α × β) (α × β) R)
    (h : (kron Z (1 : Matrix β β R)) * M = M * (kron Z (1 : Matrix β β R))) :
    Z * partialTraceRight M = partialTraceRight M * Z := by
  calc Z * partialTraceRight M
      = partialTraceRight ((kron Z 1) * M) := (partialTraceRight_bimodule Z M).symm
    _ = partialTraceRight (M * (kron Z 1)) := by rw [h]
    _ = partialTraceRight M * Z := partialTraceRight_bimodule_right Z M

end DeepCausalityFormal.Quantum
