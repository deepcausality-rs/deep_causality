/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Exemplar 1 — foundational monoid laws (Num layer).

Mirrors the Rust trait `deep_causality_num::AddMonoid`, a blanket impl over
`Add + AddAssign + Zero + Clone` (see `deep_causality_num/src/algebra/monoid.rs`).
These laws are near-inherited from Mathlib's `AddMonoid` class; the value of stating them
here is that they become the pinned property statements bound to a Rust witness.

Rust witness: `deep_causality_algebra/tests/algebra/monoid_tests.rs`.
-/

import Mathlib.Algebra.Group.Defs

namespace DeepCausalityFormal.Algebra

/-- Associativity of the additive monoid operation: `(a + b) + c = a + (b + c)`.

    THEOREM_MAP: `algebra.add_monoid.assoc` -/
theorem add_monoid_assoc {M : Type*} [AddMonoid M] (a b c : M) :
    (a + b) + c = a + (b + c) :=
  add_assoc a b c

/-- Two-sided identity of the additive monoid: `a + 0 = a` and `0 + a = a`.
    The identity element `0` corresponds to the Rust `Zero` trait.

    THEOREM_MAP: `algebra.add_monoid.identity` -/
theorem add_monoid_identity {M : Type*} [AddMonoid M] (a : M) :
    a + 0 = a ∧ 0 + a = a :=
  ⟨add_zero a, zero_add a⟩

end DeepCausalityFormal.Algebra
