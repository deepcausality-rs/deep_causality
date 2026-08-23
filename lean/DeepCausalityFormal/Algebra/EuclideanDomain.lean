/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — Euclidean-domain laws (division with remainder, and the gcd it generates).

Mirrors the Rust trait `deep_causality_algebra::EuclideanDomain`
(`src/algebra/domain_euclidean.rs`), which admits ℤ into the tower at the level that carries exact
integer arithmetic. The trait supplies `euclidean_fn` (φ = |·|), `div_euclid`, `rem_euclid`, and a
default `gcd`/`lcm` pair computed by the Euclidean algorithm.

The reconstruction law `b * (a / b) + a % b = a` is already pinned by `num.integer.euclidean` in
`Num/Integer.lean`. What this file adds is the part that makes the division *Euclidean* rather
than merely exact: the remainder is non-negative and strictly smaller than φ of the divisor. That
second bound is the termination argument for the Euclidean algorithm — φ strictly decreases on
each step and φ is ℕ-valued, so the recursion is well founded.

Also stated here is the negative fact that keeps ℤ out of `Field`: 2 has no multiplicative inverse
over ℤ. That is why `Invertible` (see `Algebra/Invertible.lean`) is withheld from the integers, and
so why the tower stops ℤ at `CommutativeRing`.

Rust witness: `deep_causality_algebra/tests/formalization_lean/`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.Algebra.Order.Group.Int
import Mathlib.Algebra.Order.Group.Unbundled.Int

namespace DeepCausalityFormal.Algebra

/-- The Euclidean remainder is non-negative: `b ≠ 0 → 0 ≤ a % b`.

    Mirrors the Rust `EuclideanDomain::rem_euclid`, which — unlike the `%` operator — is
    documented to return a non-negative value regardless of the sign of either operand.

    THEOREM_MAP: `algebra.euclidean.remainder_nonneg` -/
theorem euclidean_remainder_nonneg (a b : ℤ) (h : b ≠ 0) : 0 ≤ a % b :=
  Int.emod_nonneg a h

/-- The Euclidean remainder strictly decreases φ: `b ≠ 0 → a % b < |b|`.

    This is the termination argument, and it is stated for every non-zero divisor rather than
    only for a positive one. The first step of the Euclidean algorithm divides by the caller's
    own argument, which may be negative — `gcd(a, -12)` is an ordinary call — and only from the
    second step on is the divisor a previous remainder and hence non-negative by
    `euclidean_remainder_nonneg`. A bound assuming `0 < b` would therefore leave the entry to
    the recursion uncovered. Bounding by φ(b) = |b| covers both, and since φ is ℕ-valued and
    strictly decreasing the recursion in `EuclideanDomain::gcd` is well founded and terminates
    in finitely many steps rather than merely being well typed.

    THEOREM_MAP: `algebra.euclidean.remainder_lt_divisor` -/
theorem euclidean_remainder_lt_divisor (a b : ℤ) (h : b ≠ 0) : a % b < |b| :=
  Int.emod_lt_abs a h

/-- The gcd divides its left argument.

    THEOREM_MAP: `algebra.euclidean.gcd_dvd_left` -/
theorem euclidean_gcd_dvd_left (a b : ℤ) : (Int.gcd a b : ℤ) ∣ a :=
  Int.gcd_dvd_left a b

/-- The gcd divides its right argument.

    Together with `euclidean_gcd_dvd_left` this is the "common divisor" half of the specification
    of `EuclideanDomain::gcd`.

    THEOREM_MAP: `algebra.euclidean.gcd_dvd_right` -/
theorem euclidean_gcd_dvd_right (a b : ℤ) : (Int.gcd a b : ℤ) ∣ b :=
  Int.gcd_dvd_right a b

/-- The gcd is non-negative, whatever the signs of its arguments.

    Mirrors the Rust behaviour: because the algorithm iterates `rem_euclid`, which is itself
    non-negative, `gcd(-48, 18)` is `6` rather than `-6`.

    THEOREM_MAP: `algebra.euclidean.gcd_nonneg` -/
theorem euclidean_gcd_nonneg (a b : ℤ) : 0 ≤ (Int.gcd a b : ℤ) :=
  Int.natCast_nonneg _

/-- The base case of the algorithm: `gcd(a, 0) = |a|`.

    This is where the recursion stops, and it is why `gcd` returns the absolute value rather than
    the argument itself.

    THEOREM_MAP: `algebra.euclidean.gcd_zero_right` -/
theorem euclidean_gcd_zero_right (a : ℤ) : Int.gcd a 0 = a.natAbs :=
  Int.gcd_zero_right a

/-- ℤ is **not** a field: `2` has no multiplicative inverse over the integers.

    This is the load-bearing negative fact for the tower. Integer `/` is a truncating quotient,
    not an inverse — `1 / 5 = 0` — so admitting the integers to `CommutativeRing` must not carry
    them on to `Field`. The `Invertible` marker is what withholds that step; this theorem is the
    reason it must.

    THEOREM_MAP: `algebra.euclidean.integers_not_field` -/
theorem euclidean_integers_not_field : ¬ ∃ x : ℤ, 2 * x = 1 := by
  omega

end DeepCausalityFormal.Algebra
