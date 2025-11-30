/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AssociativeRing, Field, Module};

/// An Associative Algebra is an `AssociativeRing` that is also a `Module`
/// over a `Field` (its scalar field), such that scalar multiplication
/// is compatible with the ring multiplication.
///
/// This trait serves as a semantic marker for types that explicitly represent
/// a **Unital** Associative Algebra (since `Ring` requires `One`).
///
/// ## Requirements:
/// 1. `Self` forms an `AssociativeRing`.
/// 2. `Self` forms a `Module` over a scalar `Field`.
/// 3. Compatibility of operations:
///    * `r * (a * b) = (r * a) * b = a * (r * b)` for scalar `r` and elements `a, b`.
///    * The `Module` trait's `Mul<R, Output=Self>` and `MulAssign<R>` already
///      imply the necessary compatibility with scalar multiplication.
pub trait AssociativeAlgebra<R: Field>: AssociativeRing + Module<R> {}

// Blanket implementation for any type that satisfies the bounds
impl<T, R> AssociativeAlgebra<R> for T
where
    T: AssociativeRing + Module<R>,
    R: Field,
{
}
