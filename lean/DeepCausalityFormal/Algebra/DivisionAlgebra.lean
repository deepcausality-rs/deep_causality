/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — division-algebra inverse law.

Mirrors the Rust trait `deep_causality_algebra::DivisionAlgebra`
(`src/algebra/algebra_div.rs`). A division algebra is an algebra in which every non-zero element has
a multiplicative inverse; the defining law is exactly the non-zero cancellation of Mathlib's
`DivisionRing`. Stating it here pins the property statement bound to the Rust witness.

Rust witness: `deep_causality_algebra/tests/algebra/`.
-/

import Mathlib.Algebra.Field.Basic

namespace DeepCausalityFormal.Algebra

/-- Right inverse away from zero in a division ring: `a ≠ 0 → a * a⁻¹ = 1`
    (Mathlib `mul_inv_cancel₀`). Mirrors the `DivisionAlgebra` inverse law.

    THEOREM_MAP: `algebra.division_algebra.mul_inv` -/
theorem division_algebra_mul_inv {D : Type*} [DivisionRing D] (a : D) (h : a ≠ 0) :
    a * a⁻¹ = 1 :=
  mul_inv_cancel₀ h

end DeepCausalityFormal.Algebra
