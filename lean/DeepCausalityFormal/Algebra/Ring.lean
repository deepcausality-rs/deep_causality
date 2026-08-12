/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — ring laws (distributivity, associativity, commutativity).

Mirrors the Rust traits `deep_causality_algebra::{Ring, AssociativeRing, CommutativeRing, Distributive}`
(`src/algebra/{ring,ring_associative,ring_commutative,distributive}.rs`). A ring couples an abelian
group under addition with an associative multiplicative monoid, tied together by the two distributive
laws; a commutative ring adds multiplicative commutativity. These are Mathlib's `Ring` and `CommRing`
classes; stating them here pins the property statements bound to the Rust witnesses.

Rust witness: `deep_causality_algebra/tests/algebra/`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.Algebra.Ring.Basic

namespace DeepCausalityFormal.Algebra

/-- Left distributivity: `a * (b + c) = a * b + a * c` (Mathlib `left_distrib`).
    Mirrors the `Distributive` marker on `Ring`.

    THEOREM_MAP: `algebra.ring.left_distrib` -/
theorem ring_left_distrib {R : Type*} [Ring R] (a b c : R) :
    a * (b + c) = a * b + a * c :=
  left_distrib a b c

/-- Right distributivity: `(a + b) * c = a * c + b * c` (Mathlib `right_distrib`).
    Mirrors the `Distributive` marker on `Ring`.

    THEOREM_MAP: `algebra.ring.right_distrib` -/
theorem ring_right_distrib {R : Type*} [Ring R] (a b c : R) :
    (a + b) * c = a * c + b * c :=
  right_distrib a b c

/-- Associativity of ring multiplication: `(a * b) * c = a * (b * c)` (Mathlib `mul_assoc`).
    Mirrors the `AssociativeRing` law.

    THEOREM_MAP: `algebra.ring.mul_assoc` -/
theorem ring_mul_assoc {R : Type*} [Ring R] (a b c : R) :
    (a * b) * c = a * (b * c) :=
  mul_assoc a b c

/-- Commutativity of ring multiplication: `a * b = b * a` (Mathlib `mul_comm`).
    Mirrors the `CommutativeRing` law.

    THEOREM_MAP: `algebra.commutative_ring.mul_comm` -/
theorem commutative_ring_mul_comm {R : Type*} [CommRing R] (a b : R) :
    a * b = b * a :=
  mul_comm a b

end DeepCausalityFormal.Algebra
