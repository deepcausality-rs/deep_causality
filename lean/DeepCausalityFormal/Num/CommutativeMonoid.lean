/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — commutative-monoid and bounded-semilattice laws.

Mirrors the Rust traits `deep_causality_num::{CommutativeMonoid, Idempotent, BoundedSemilattice}`
(`src/algebra/{commutative_monoid,idempotent,bounded_semilattice}.rs`). Commutativity is stated
abstractly over Mathlib's `CommMonoid`; the bounded-semilattice laws (idempotence, associativity,
commutativity of the meet) are proved concretely on the boolean `∧` — the `Conjunction`/`Disjunction`
`AggregateLogic` reducers — which needs no `Mathlib.Order.*` (its olean cache is unavailable in this
environment) and matches the Rust `bool`-based instances exactly. `Count` is a commutative monoid
that is deliberately not idempotent (`Count(1).combine(Count(1)) = Count(2) ≠ Count(1)`).

Rust witness: `deep_causality_num/tests/algebra/commutative_semilattice_tests.rs`.
-/

import Mathlib.Algebra.Group.Defs

namespace DeepCausalityFormal.Num

/-- Commutativity: `x.combine(y) = y.combine(x)` (Mathlib `mul_comm`).

    THEOREM_MAP: `num.commutative_monoid.comm` -/
theorem commutative_monoid_comm {M : Type*} [CommMonoid M] (x y : M) :
    x * y = y * x :=
  mul_comm x y

/-- Idempotence of the boolean ∧-semilattice (`Conjunction`): `x.combine(x) = x`.

    THEOREM_MAP: `num.semilattice.idempotent` -/
theorem semilattice_idempotent (x : Bool) : (x && x) = x := by
  cases x <;> rfl

/-- Associativity of the boolean ∧-semilattice.

    THEOREM_MAP: `num.semilattice.assoc` -/
theorem semilattice_assoc (x y z : Bool) : ((x && y) && z) = (x && (y && z)) := by
  cases x <;> cases y <;> cases z <;> rfl

/-- Commutativity of the boolean ∧-semilattice.

    THEOREM_MAP: `num.semilattice.comm` -/
theorem semilattice_comm (x y : Bool) : (x && y) = (y && x) := by
  cases x <;> cases y <;> rfl

end DeepCausalityFormal.Num
