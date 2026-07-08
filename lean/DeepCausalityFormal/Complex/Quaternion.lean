/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Quaternion — division-ring, norm, conjugation, and non-commutativity laws.

Mirrors the Rust `Quaternion` type of `deep_causality_num_complex` (see
`deep_causality_num_complex/src/quaternion/`). The laws are stated on Mathlib's `ℍ[ℝ]`
(`Quaternion ℝ`), the canonical model: a non-commutative division ring with an antihomomorphic
conjugation and a multiplicative squared norm. The `quaternion_noncomm` theorem pins the defining
non-commutativity via the Hamilton relations `i·j = k` and `j·i = -k`. Octonions are intentionally
out of scope (non-associative and absent from Mathlib).

Rust witness: `deep_causality_num_complex/tests/complex/`.
-/

import Mathlib.Algebra.Quaternion
import Mathlib.Analysis.Quaternion

open scoped Quaternion

namespace DeepCausalityFormal.Complex

/-- Division-ring inverse: a nonzero quaternion times its inverse is one, `q * q⁻¹ = 1`.
    Mirrors the `Quaternion` reciprocal in `deep_causality_num_complex`.

    THEOREM_MAP: `quaternion.division_ring.mul_inv` -/
theorem quaternion_mul_inv (q : ℍ[ℝ]) (h : q ≠ 0) : q * q⁻¹ = 1 :=
  mul_inv_cancel₀ h

/-- The squared norm is multiplicative: `normSq (q * p) = normSq q * normSq p`.
    Mirrors the squared-modulus of the `Quaternion` type in `deep_causality_num_complex`.

    THEOREM_MAP: `quaternion.norm_sq.mul` -/
theorem quaternion_normSq_mul (q p : ℍ[ℝ]) :
    Quaternion.normSq (q * p) = Quaternion.normSq q * Quaternion.normSq p :=
  map_mul Quaternion.normSq q p

/-- Conjugation is an antihomomorphism: `star (q * p) = star p * star q` (order reverses).
    Mirrors the `Quaternion` conjugation of `deep_causality_num_complex`.

    THEOREM_MAP: `quaternion.conj.mul` -/
theorem quaternion_conj_mul (q p : ℍ[ℝ]) : star (q * p) = star p * star q :=
  star_mul q p

/-- Multiplication is not commutative: the Hamilton units `i` and `j` satisfy `i·j = k` but
    `j·i = -k`, so `i * j ≠ j * i`. Mirrors the non-commutative `Quaternion` product in
    `deep_causality_num_complex`.

    THEOREM_MAP: `quaternion.noncomm` -/
theorem quaternion_noncomm :
    (⟨0, 1, 0, 0⟩ : ℍ[ℝ]) * ⟨0, 0, 1, 0⟩ ≠ (⟨0, 0, 1, 0⟩ : ℍ[ℝ]) * ⟨0, 1, 0, 0⟩ := by
  intro h
  have hk : ((⟨0, 1, 0, 0⟩ : ℍ[ℝ]) * ⟨0, 0, 1, 0⟩).imK
      = ((⟨0, 0, 1, 0⟩ : ℍ[ℝ]) * ⟨0, 1, 0, 0⟩).imK := by rw [h]
  simp only [QuaternionAlgebra.mk_mul_mk] at hk
  norm_num at hk

end DeepCausalityFormal.Complex
