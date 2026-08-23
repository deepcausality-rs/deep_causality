/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Semiring;
use crate::algebra::commutative::Commutative;

/// A marker trait for a **Commutative Semiring**.
///
/// A commutative semiring is a [`Semiring`] whose multiplication commutes. It is exactly the
/// structure of the natural numbers ℕ, and it is where the unsigned integer types stop in this
/// tower.
///
/// # Mathematical Definition
///
/// A semiring `(R, +, *)` is commutative if it satisfies the additional law:
///
/// 1.  **Commutativity of Multiplication:** `a * b = b * a` for all `a, b` in `R`.
///
/// ## Note on Implementation
///
/// This is a **marker trait** and has no methods. Its purpose is to signal at the type level that
/// the commutativity law holds. The compiler cannot verify this law, so implementing it is a
/// promise by the developer.
///
/// ## Where this sits relative to `CommutativeRing`
///
/// `CommutativeSemiring` is `CommutativeRing` minus additive inverses. ℕ satisfies the first and
/// not the second; ℤ satisfies both. The step between them is `Neg`, which is the witness for
/// `-a` and the bound that [`AbelianGroup`](crate::AbelianGroup) requires — so
/// `assert_commutative_semiring::<u64>()` compiles while `assert_commutative_ring::<u64>()` does
/// not.
pub trait CommutativeSemiring: Semiring + Commutative {}
impl<T> CommutativeSemiring for T where T: Semiring + Commutative {}
