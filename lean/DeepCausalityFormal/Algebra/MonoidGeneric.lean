/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — the carrier-and-operation-generic monoid laws.

Mirrors the Rust trait `deep_causality_num::Monoid` (`fn empty() -> Self`, `fn combine(self, Self) -> Self`,
`src/algebra/monoid_generic.rs`) — a monoid decoupled from `Zero`/`One`/`Add`/`Mul`. The abstract laws
are exactly Mathlib's `Monoid` class (multiplicative notation: `1` = `empty`, `*` = `combine`); stating
them here pins the property statements bound to the Rust witnesses.

Rust witness: `deep_causality_algebra/tests/algebra/monoid_generic_tests.rs`.
-/

import Mathlib.Algebra.Group.Defs

namespace DeepCausalityFormal.Algebra

/-- Left identity: `empty().combine(x) = x` (Mathlib `one_mul`).

    THEOREM_MAP: `algebra.monoid.left_id` -/
theorem monoid_left_id {M : Type*} [Monoid M] (x : M) : 1 * x = x :=
  one_mul x

/-- Right identity: `x.combine(empty()) = x` (Mathlib `mul_one`).

    THEOREM_MAP: `algebra.monoid.right_id` -/
theorem monoid_right_id {M : Type*} [Monoid M] (x : M) : x * 1 = x :=
  mul_one x

/-- Associativity: `x.combine(y).combine(z) = x.combine(y.combine(z))` (Mathlib `mul_assoc`).

    THEOREM_MAP: `algebra.monoid.assoc` -/
theorem monoid_assoc {M : Type*} [Monoid M] (x y z : M) :
    (x * y) * z = x * (y * z) :=
  mul_assoc x y z

end DeepCausalityFormal.Algebra
