/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — group laws (inverses and commutativity).

Mirrors the Rust traits `deep_causality_algebra::{AddGroup, MulGroup, AbelianGroup}`
(`src/algebra/{group_add,group_mul,group_abelian}.rs`). A group extends a monoid with a
two-sided inverse; an abelian group adds commutativity. These laws are Mathlib's `Group`,
`AddGroup`, and `AddCommGroup` classes; stating them here pins the property statements bound
to the Rust witnesses.

Rust witness: `deep_causality_algebra/tests/algebra/`.
-/

import Mathlib.Algebra.Group.Basic

namespace DeepCausalityFormal.Algebra

/-- Right inverse of the multiplicative group: `a * a⁻¹ = 1` (Mathlib `mul_inv_cancel`).
    Mirrors the `MulGroup` inverse law.

    THEOREM_MAP: `algebra.group.mul_inv` -/
theorem group_mul_inv {G : Type*} [Group G] (a : G) : a * a⁻¹ = 1 :=
  mul_inv_cancel a

/-- Left inverse of the additive group: `-a + a = 0` (Mathlib `neg_add_cancel`).
    Mirrors the `AddGroup` inverse law.

    THEOREM_MAP: `algebra.add_group.neg_cancel` -/
theorem add_group_neg_cancel {G : Type*} [AddGroup G] (a : G) : -a + a = 0 :=
  neg_add_cancel a

/-- Commutativity of the abelian group: `a + b = b + a` (Mathlib `add_comm`).
    Mirrors the `AbelianGroup` commutativity law.

    THEOREM_MAP: `algebra.abelian_group.add_comm` -/
theorem abelian_group_add_comm {G : Type*} [AddCommGroup G] (a b : G) : a + b = b + a :=
  add_comm a b

end DeepCausalityFormal.Algebra
