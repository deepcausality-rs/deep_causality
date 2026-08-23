/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Rational — ℚ as the field of fractions of ℤ.

Mirrors the Rust type `deep_causality_num_rational::Rational<T>`
(`src/rational/rational_number/`), built over `T: EuclideanDomain` because reducing a fraction to
lowest terms needs the gcd that a Euclidean domain provides.

Three groups of facts are pinned here.

The **field laws** are what ℚ has and ℤ does not: every non-zero rational inverts. This is the
`Invertible` marker in the tower, and the reason `Rational` reaches `Field` while the integers it
is built from stop at `CommutativeRing`.

The **canonical-form invariants** mirror the Rust type's private fields. Mathlib's `Rat` maintains
exactly the same normalisation the Rust `reduce` does — a positive denominator and coprime
components — so `Rat.den_pos` and `Rat.reduced` are the model of invariants 1 and 2. Those
invariants are what make equality structural on both sides: two values are equal exactly when
their stored components match, with no cross-multiplication.

**Density** is the order property that separates ℚ from ℤ: between any two distinct rationals lies
a third, so ℚ has no discrete successor. It is the reason ℚ is a sensible domain for exact
subdivision even though it is not analytically closed.

Rust witness: `deep_causality_num_rational/tests/formalization_lean/`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.Data.Rat.Cast.Defs
import Mathlib.Algebra.Field.Basic
import Mathlib.Tactic.Linarith

namespace DeepCausalityFormal.Rational

/-- Every non-zero rational has a multiplicative inverse: `q ≠ 0 → q * q⁻¹ = 1`.

    The defining field axiom, and the one ℤ cannot satisfy. Mirrors `Rational::recip` together
    with the `Invertible` marker that carries `Rational<T>` from `CommutativeRing` to `Field`.

    THEOREM_MAP: `rational.field.mul_inv` -/
theorem rational_mul_inv (q : ℚ) (h : q ≠ 0) : q * q⁻¹ = 1 :=
  mul_inv_cancel₀ h

/-- Multiplication commutes over ℚ.

    Mirrors the `Commutative` marker on `Rational<T>`.

    THEOREM_MAP: `rational.field.mul_comm` -/
theorem rational_mul_comm (a b : ℚ) : a * b = b * a :=
  mul_comm a b

/-- Multiplication is associative over ℚ.

    Mirrors the `Associative` marker on `Rational<T>`.

    THEOREM_MAP: `rational.field.mul_assoc` -/
theorem rational_mul_assoc (a b c : ℚ) : a * b * c = a * (b * c) :=
  mul_assoc a b c

/-- Left distributivity over ℚ: `a * (b + c) = a * b + a * c`.

    Mirrors the `Distributive` marker on `Rational<T>`.

    THEOREM_MAP: `rational.field.distrib` -/
theorem rational_distrib (a b c : ℚ) : a * (b + c) = a * b + a * c :=
  mul_add a b c

/-- Additive inverses exist: `a + (-a) = 0`.

    Mirrors the `AbelianGroup` marker on `Rational<T>`, and the Rust `Neg` impl that negates the
    numerator while leaving the denominator positive.

    THEOREM_MAP: `rational.abelian_group.add_neg` -/
theorem rational_add_neg (a : ℚ) : a + (-a) = 0 :=
  add_neg_cancel a

/-- Canonical form, invariant 1: the denominator is strictly positive.

    Mirrors the Rust `reduce`, which moves any sign out of the denominator and into the numerator
    by negating both components. Keeping the denominator positive is what lets `Ord` compare two
    rationals by cross-multiplication without having to track a sign flip.

    THEOREM_MAP: `rational.canonical.den_pos` -/
theorem rational_den_pos (q : ℚ) : 0 < q.den :=
  q.den_pos

/-- Canonical form, invariant 2: numerator and denominator are coprime.

    Mirrors the second step of the Rust `reduce`, which divides both components through by their
    gcd. Together with invariant 1 this makes the representation unique, and therefore makes
    equality structural rather than a cross-multiplication.

    THEOREM_MAP: `rational.canonical.coprime` -/
theorem rational_coprime (q : ℚ) : q.num.natAbs.Coprime q.den :=
  q.reduced

/-- ℚ is dense: between any two distinct rationals lies a third.

    The order property that distinguishes ℚ from ℤ — there is no successor function. Mirrors the
    Rust `density_between_any_two_rationals` witness.

    THEOREM_MAP: `rational.order.dense` -/
theorem rational_dense (a b : ℚ) (h : a < b) : a < (a + b) / 2 ∧ (a + b) / 2 < b := by
  constructor <;> linarith

end DeepCausalityFormal.Rational
