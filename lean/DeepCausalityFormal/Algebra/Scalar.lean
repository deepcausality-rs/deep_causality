/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — conjugation and norm laws for scalars.

Mirrors the Rust traits `deep_causality_algebra::{ConjugateScalar, Normed, NormedScalar}`
(`src/algebra/{scalar_conjugate,normed,scalar_normed}.rs`). Conjugation is an involutive,
order-reversing ring anti-homomorphism (Mathlib's `StarRing`), while the norm is a non-negative,
multiplicative absolute value (Mathlib's `SeminormedAddGroup` / `NormedField`). Stating these laws
here pins the property statements bound to the Rust witnesses.

Rust witness: `deep_causality_unified_math/deep_causality_algebra/tests/algebra/`
(`scalar_conjugate_tests.rs`, `scalar_normed_tests.rs`).

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

import Mathlib.Algebra.Star.Basic
import Mathlib.Analysis.Normed.Field.Basic

namespace DeepCausalityFormal.Algebra

/-- Conjugation is involutive: `star (star a) = a` (Mathlib `star_star`).
    Mirrors the `ConjugateScalar` involution law.

    THEOREM_MAP: `algebra.conjugate.star_star` -/
theorem conjugate_star_star {R : Type*} [NonUnitalNonAssocSemiring R] [StarRing R] (a : R) :
    star (star a) = a :=
  star_star a

/-- Conjugation reverses a product: `star (a * b) = star b * star a` (Mathlib `star_mul`).
    Mirrors the `ConjugateScalar` anti-homomorphism law.

    THEOREM_MAP: `algebra.conjugate.star_mul` -/
theorem conjugate_star_mul {R : Type*} [NonUnitalNonAssocSemiring R] [StarRing R] (a b : R) :
    star (a * b) = star b * star a :=
  star_mul a b

/-- Conjugation is additive: `star (a + b) = star a + star b` (Mathlib `star_add`).
    Mirrors the `ConjugateScalar` additivity law.

    THEOREM_MAP: `algebra.conjugate.star_add` -/
theorem conjugate_star_add {R : Type*} [NonUnitalNonAssocSemiring R] [StarRing R] (a b : R) :
    star (a + b) = star a + star b :=
  star_add a b

/-- The norm is multiplicative: `‖a * b‖ = ‖a‖ * ‖b‖` (Mathlib `norm_mul`).
    Mirrors the `NormedScalar` multiplicativity law.

    THEOREM_MAP: `algebra.normed.norm_mul` -/
theorem normed_norm_mul {K : Type*} [NormedField K] (a b : K) :
    ‖a * b‖ = ‖a‖ * ‖b‖ :=
  norm_mul a b

/-- The norm is non-negative: `0 ≤ ‖a‖` (Mathlib `norm_nonneg`).
    Mirrors the `Normed` non-negativity law.

    THEOREM_MAP: `algebra.normed.norm_nonneg` -/
theorem normed_norm_nonneg {E : Type*} [SeminormedAddGroup E] (a : E) :
    0 ≤ ‖a‖ :=
  norm_nonneg a

end DeepCausalityFormal.Algebra
