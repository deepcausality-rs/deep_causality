/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Algebra — module and algebra laws (scalar action and its interaction with the ring product).

Mirrors the Rust traits `deep_causality_algebra::{Module, Algebra}`
(`src/algebra/{module,algebra_base}.rs`). A module is an abelian group carrying a compatible scalar
action; an algebra additionally makes that action commute with the ring multiplication. The bilinearity
and unit laws are Mathlib's `smul_add` / `add_smul` / `one_smul` / `mul_smul`, and the scalar-tower and
commutation laws are `smul_mul_assoc` / `mul_smul_comm`; stating them here pins the property statements
bound to the Rust witnesses.

Rust witness: `deep_causality_algebra/tests/algebra/`.
-/

import Mathlib.Algebra.Module.Basic
import Mathlib.Algebra.Algebra.Basic

namespace DeepCausalityFormal.Algebra

/-- Scalar distributes over vector addition: `r • (x + y) = r • x + r • y` (Mathlib `smul_add`).
    Mirrors the `Module` bilinearity law.

    THEOREM_MAP: `algebra.module.smul_add` -/
theorem module_smul_add {R M : Type*} [Semiring R] [AddCommMonoid M] [Module R M]
    (r : R) (x y : M) : r • (x + y) = r • x + r • y :=
  smul_add r x y

/-- Scalar addition distributes over the action: `(r + s) • x = r • x + s • x` (Mathlib `add_smul`).
    Mirrors the `Module` bilinearity law.

    THEOREM_MAP: `algebra.module.add_smul` -/
theorem module_add_smul {R M : Type*} [Semiring R] [AddCommMonoid M] [Module R M]
    (r s : R) (x : M) : (r + s) • x = r • x + s • x :=
  add_smul r s x

/-- Unit scalar acts trivially: `(1 : R) • x = x` (Mathlib `one_smul`).
    Mirrors the `Module` identity law.

    THEOREM_MAP: `algebra.module.one_smul` -/
theorem module_one_smul {R M : Type*} [Semiring R] [AddCommMonoid M] [Module R M]
    (x : M) : (1 : R) • x = x :=
  one_smul R x

/-- Scalar multiplication is compatible with the action: `(r * s) • x = r • (s • x)`
    (Mathlib `mul_smul`). Mirrors the `Module` compatibility law.

    THEOREM_MAP: `algebra.module.mul_smul` -/
theorem module_mul_smul {R M : Type*} [Semiring R] [AddCommMonoid M] [Module R M]
    (r s : R) (x : M) : (r * s) • x = r • (s • x) :=
  mul_smul r s x

/-- Scalar passes through the left factor: `r • (a * b) = (r • a) * b` (Mathlib `smul_mul_assoc`).
    Mirrors the `Algebra` scalar-tower law.

    THEOREM_MAP: `algebra.algebra.smul_mul_assoc` -/
theorem algebra_smul_mul_assoc {R A : Type*} [CommSemiring R] [Semiring A] [Algebra R A]
    (r : R) (a b : A) : r • (a * b) = (r • a) * b :=
  (smul_mul_assoc r a b).symm

/-- Scalar passes through the right factor: `r • (a * b) = a * (r • b)` (Mathlib `mul_smul_comm`).
    Mirrors the `Algebra` scalar-commutation law.

    THEOREM_MAP: `algebra.algebra.mul_smul_comm` -/
theorem algebra_mul_smul_comm {R A : Type*} [CommSemiring R] [Semiring A] [Algebra R A]
    (r : R) (a b : A) : r • (a * b) = a * (r • b) :=
  (mul_smul_comm r a b).symm

end DeepCausalityFormal.Algebra
