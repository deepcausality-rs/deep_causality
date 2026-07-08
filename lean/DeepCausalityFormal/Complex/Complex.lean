/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Complex — field, conjugation, and norm laws.

Mirrors the Rust `Complex` type of `deep_causality_num_complex` (see
`deep_causality_num_complex/src/complex/`). The laws are stated on Mathlib's `ℂ`, which is the
canonical model of the same algebra: a field with an involutive, multiplicative conjugation whose
squared norm is multiplicative. Octonions are intentionally out of scope (they are non-associative
and absent from Mathlib).

Rust witness: `deep_causality_num_complex/tests/complex/`.
-/

import Mathlib.Analysis.Complex.Basic

namespace DeepCausalityFormal.Complex

/-- Field inverse: a nonzero complex number times its inverse is one, `z * z⁻¹ = 1`.
    Mirrors the `Complex` division-ring reciprocal in `deep_causality_num_complex`.

    THEOREM_MAP: `complex.field.mul_inv` -/
theorem complex_field_mul_inv (z : ℂ) (h : z ≠ 0) : z * z⁻¹ = 1 :=
  mul_inv_cancel₀ h

/-- Conjugation is involutive: conjugating twice returns the original, `conj (conj z) = z`.
    Mirrors the `Complex` conjugation of `deep_causality_num_complex`.

    THEOREM_MAP: `complex.conj.involutive` -/
theorem complex_conj_involutive (z : ℂ) :
    (starRingEnd ℂ) ((starRingEnd ℂ) z) = z :=
  Complex.conj_conj z

/-- Conjugation is multiplicative: `conj (z * w) = conj z * conj w`.
    Mirrors the `Complex` conjugation of `deep_causality_num_complex`.

    THEOREM_MAP: `complex.conj.mul` -/
theorem complex_conj_mul (z w : ℂ) :
    (starRingEnd ℂ) (z * w) = (starRingEnd ℂ) z * (starRingEnd ℂ) w :=
  map_mul (starRingEnd ℂ) z w

/-- The squared norm is multiplicative: `normSq (z * w) = normSq z * normSq w`.
    Mirrors the squared-modulus of the `Complex` type in `deep_causality_num_complex`.

    THEOREM_MAP: `complex.norm_sq.mul` -/
theorem complex_normSq_mul (z w : ℂ) :
    Complex.normSq (z * w) = Complex.normSq z * Complex.normSq w :=
  Complex.normSq_mul z w

/-- The norm is multiplicative: `‖z * w‖ = ‖z‖ * ‖w‖`.
    Mirrors the modulus of the `Complex` type in `deep_causality_num_complex`.

    THEOREM_MAP: `complex.norm.mul` -/
theorem complex_norm_mul (z w : ℂ) : ‖z * w‖ = ‖z‖ * ‖w‖ :=
  norm_mul z w

end DeepCausalityFormal.Complex
