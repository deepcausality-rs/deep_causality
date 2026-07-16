/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — field laws (multiplicative inverses and the ordered-field sign rule).

Mirrors the Rust traits `deep_causality_algebra::{Field, RealField}`
(`src/algebra/{field,field_real}.rs`). A field is a commutative ring in which every non-zero
element has a multiplicative inverse; a real field adds a compatible linear order. The inverse
laws are Mathlib's `mul_inv_cancel₀` / `inv_mul_cancel₀`, and positivity of a product of positives
is `mul_pos`; stating them here pins the property statements bound to the Rust witnesses.

Rust witness: `deep_causality_algebra/tests/algebra/`.
-/

import Mathlib.Algebra.Field.Basic
import Mathlib.Algebra.Order.Field.Defs
import Mathlib.Algebra.Order.Ring.Defs

namespace DeepCausalityFormal.Algebra

/-- Right inverse away from zero: `a ≠ 0 → a * a⁻¹ = 1` (Mathlib `mul_inv_cancel₀`).
    Mirrors the `Field` division law.

    THEOREM_MAP: `algebra.field.mul_inv_cancel` -/
theorem field_mul_inv_cancel {F : Type*} [Field F] (a : F) (h : a ≠ 0) :
    a * a⁻¹ = 1 :=
  mul_inv_cancel₀ h

/-- Left inverse away from zero: `a ≠ 0 → a⁻¹ * a = 1` (Mathlib `inv_mul_cancel₀`).
    Mirrors the `Field` division law.

    THEOREM_MAP: `algebra.field.inv_mul_cancel` -/
theorem field_inv_mul_cancel {F : Type*} [Field F] (a : F) (h : a ≠ 0) :
    a⁻¹ * a = 1 :=
  inv_mul_cancel₀ h

/-- Product of positives is positive: `0 < a → 0 < b → 0 < a * b` (Mathlib `mul_pos`).
    Mirrors the ordered `RealField` sign rule.

    THEOREM_MAP: `algebra.real_field.mul_pos` -/
theorem real_field_mul_pos {F : Type*} [Field F] [LinearOrder F] [IsStrictOrderedRing F]
    {a b : F} (ha : 0 < a) (hb : 0 < b) : 0 < a * b :=
  mul_pos ha hb

end DeepCausalityFormal.Algebra
