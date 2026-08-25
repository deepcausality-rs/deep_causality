/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Algebra, Ring};

/// A marker trait for an **Associative Algebra**.
///
/// This trait identifies an `Algebra` where the multiplication operation is associative.
///
/// The refinement is real rather than cosmetic: [`Algebra`](crate::Algebra) deliberately omits the
/// associativity bound so that non-associative algebras — `Octonion<T>` — can implement it, while
/// [`Ring`](crate::Ring) requires associativity through `MulMonoid`. Requiring both is therefore
/// exactly "an algebra whose multiplication associates".
///
/// It is distinct from non-associative algebras like Octonions.
///
/// # Mathematical Definition
///
/// An associative algebra `A` is an algebra that is also a `Ring`.
/// This means it satisfies the law:
///
/// `(x * y) * z = x * (y * z)` for all `x, y, z` in `A`.
///
/// ## Examples
///
/// - **Associative:** Real numbers, Complex numbers, Quaternions.
/// - **Non-Associative:** Octonions.
pub trait AssociativeAlgebra<R: Ring>: Algebra<R> + Ring {}

// Blanket implementation
impl<T, R> AssociativeAlgebra<R> for T
where
    T: Algebra<R> + Ring, // Ring implies Associative
    R: Ring,
{
}
