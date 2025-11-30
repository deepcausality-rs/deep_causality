/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Module, Ring};
use std::ops::{Mul, MulAssign};

/// An Algebra is a Module equipped with a bilinear binary product.
///
/// It generalizes the concept of an algebra over a field (or ring).
///
/// ## Structure:
/// 1. `Self` is a Module over a scalar Ring `R`.
/// 2. `Self` is equipped with a binary operation `*` (multiplication).
/// 3. The multiplication is bilinear (distributes over addition and is compatible with scalars).
///
/// ## Note:
/// This trait does *not* require the algebra to be associative or unital.
/// * For associative algebras, see `AssociativeAlgebra`.
/// * For unital algebras, the type would typically also implement `One` (or `MulMonoid` if associative).
pub trait Algebra<R: Ring>: Module<R> + Mul<Output = Self> + MulAssign {}

// Blanket implementation
impl<T, R> Algebra<R> for T
where
    T: Module<R> + Mul<Output = Self> + MulAssign,
    R: Ring,
{
}
