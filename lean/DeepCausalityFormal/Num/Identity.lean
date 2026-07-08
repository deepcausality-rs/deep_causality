/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — identity laws for the numeric core.

Mirrors the Rust marker traits `deep_causality_num::{Zero, One}` (`src/identity/`), which pin the
two-sided identity elements of the additive and multiplicative monoids. The laws are near-inherited
from Mathlib's `Monoid`/`AddMonoid` classes; stating them here binds each Rust marker to a pinned
property.

Rust witness: `deep_causality_num/tests/identity/`.
-/

import Mathlib.Algebra.Group.Defs

namespace DeepCausalityFormal.Num

/-- Two-sided multiplicative identity: `1 * a = a` and `a * 1 = a`.
    The identity element `1` corresponds to the Rust `One` marker trait.

    THEOREM_MAP: `num.one.identity` -/
theorem one_is_identity {M : Type*} [Monoid M] (a : M) :
    1 * a = a ∧ a * 1 = a :=
  ⟨one_mul a, mul_one a⟩

/-- Two-sided additive identity: `0 + a = a` and `a + 0 = a`.
    The identity element `0` corresponds to the Rust `Zero` marker trait.

    THEOREM_MAP: `num.zero.identity` -/
theorem zero_is_identity {M : Type*} [AddMonoid M] (a : M) :
    0 + a = a ∧ a + 0 = a :=
  ⟨zero_add a, add_zero a⟩

end DeepCausalityFormal.Num
