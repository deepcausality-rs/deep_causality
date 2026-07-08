/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Laws of the `deep_causality_num_dual` crate's `Dual` number type.

A dual number `a + bε` carries a value `a` and a derivative `b`, where `ε` is a
formal symbol satisfying `ε² = 0`. The crate uses this for forward-mode automatic
differentiation: a single multiplication propagates both the value and its
derivative via the product rule. These theorems reuse Mathlib's `DualNumber R`,
defined as `TrivSqZeroExt R R`, and bind its laws to the Rust witnesses.

The real component `TrivSqZeroExt.fst` mirrors `Dual::value()`; the dual component
`TrivSqZeroExt.snd` mirrors `Dual::derivative()`.

Rust witness: `deep_causality_num_dual/tests/dual/dual_number/`.
-/

import Mathlib.Algebra.DualNumber

open scoped DualNumber
open TrivSqZeroExt

namespace DeepCausalityFormal.Dual

/-- Multiplication of dual numbers commutes: `a * b = b * a`. `DualNumber R` is a
    `CommRing` whenever `R` is, so the value and derivative of a product do not
    depend on operand order. Mirrors `Dual::mul` on a commutative scalar.

    THEOREM_MAP: `dual.comm_ring.mul_comm` -/
theorem dual_comm_ring_mul_comm {R : Type*} [CommRing R] (a b : DualNumber R) :
    a * b = b * a :=
  mul_comm a b

/-- The dual unit squares to zero: `ε * ε = 0`. This is the defining relation of
    the dual numbers and the reason first-order derivatives compose linearly.
    Discharged by Mathlib's `DualNumber.eps_mul_eps`.

    THEOREM_MAP: `dual.eps_sq_zero` -/
theorem dual_eps_sq_zero {R : Type*} [CommRing R] :
    (DualNumber.eps : DualNumber R) * DualNumber.eps = 0 :=
  DualNumber.eps_mul_eps

/-- The real projection is additive: `fst (a + b) = fst a + fst b`. The value of a
    sum is the sum of the values, so `Dual::value()` is a homomorphism for `+`.
    Discharged by `TrivSqZeroExt.fst_add`.

    THEOREM_MAP: `dual.real_projection.add` -/
theorem dual_fst_is_ring_hom_add {R : Type*} [CommRing R] (a b : DualNumber R) :
    TrivSqZeroExt.fst (a + b) = TrivSqZeroExt.fst a + TrivSqZeroExt.fst b :=
  TrivSqZeroExt.fst_add a b

/-- The real projection is multiplicative: `fst (a * b) = fst a * fst b`. The value
    of a product is the product of the values; the derivative rides in the dual
    component and never disturbs it. Discharged by `TrivSqZeroExt.fst_mul`.

    THEOREM_MAP: `dual.real_projection.mul` -/
theorem dual_fst_mul {R : Type*} [CommRing R] (a b : DualNumber R) :
    TrivSqZeroExt.fst (a * b) = TrivSqZeroExt.fst a * TrivSqZeroExt.fst b :=
  TrivSqZeroExt.fst_mul a b

/-- The product rule (Leibniz law), the heart of forward-mode AD:
    `snd (a * b) = fst a * snd b + snd a * fst b`. The derivative of a product is
    `value(a)·deriv(b) + deriv(a)·value(b)`, matching how `Dual::mul` combines the
    dual components. Discharged by `DualNumber.snd_mul`, whose right-hand side is
    stated in exactly this order.

    THEOREM_MAP: `dual.leibniz.product_rule` -/
theorem dual_leibniz {R : Type*} [CommRing R] (a b : DualNumber R) :
    TrivSqZeroExt.snd (a * b)
      = TrivSqZeroExt.fst a * TrivSqZeroExt.snd b
        + TrivSqZeroExt.snd a * TrivSqZeroExt.fst b :=
  DualNumber.snd_mul a b

/-- `ε` is a nonzero zero-divisor: `ε ≠ 0` and `ε * ε = 0`. This witnesses that
    `DualNumber R` is not a field nor an integral domain over a nontrivial `R`.
    The `≠ 0` half follows because `snd ε = 1`, so `ε = 0` would force `1 = 0`.

    THEOREM_MAP: `dual.not_field.zero_divisor` -/
theorem dual_not_field_zero_divisor {R : Type*} [CommRing R] [Nontrivial R] :
    (DualNumber.eps : DualNumber R) ≠ 0 ∧
      (DualNumber.eps : DualNumber R) * DualNumber.eps = 0 := by
  refine ⟨?_, DualNumber.eps_mul_eps⟩
  intro h
  have : (1 : R) = 0 := by
    have hsnd : TrivSqZeroExt.snd (DualNumber.eps : DualNumber R) = 0 := by
      rw [h]; exact TrivSqZeroExt.snd_zero
    rwa [DualNumber.snd_eps] at hsnd
  exact one_ne_zero this

end DeepCausalityFormal.Dual
