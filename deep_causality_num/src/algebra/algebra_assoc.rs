/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Algebra, AssociativeRing, Field, Module, Ring};

/// A marker trait for an **Associative Algebra**.
///
/// This trait identifies an `Algebra` where the multiplication operation is
/// associative. Since the `Ring` trait (required by `Algebra`) already enforces
/// associativity via `MulMonoid`, this trait serves as a semantic marker to
/// distinguish from non-associative algebras like Octonions.
///
/// # Mathematical Definition
///
/// An associative algebra `A` is an algebra that is also an `AssociativeRing`.
/// This means it satisfies the law:
///
/// `(x * y) * z = x * (y * z)` for all `x, y, z` in `A`.
///
/// ## Examples
///
/// - **Associative:** Real numbers, Complex numbers, Quaternions.
/// - **Non-Associative:** Octonions.
pub trait AssociativeAlgebra<R: Ring>: Algebra<R> + Ring {}

// Blanket implementation for any type that satisfies the bounds
impl<T, R> AssociativeAlgebra<R> for T
where
    T: AssociativeRing + Module<R>,
    R: Field,
{
}
