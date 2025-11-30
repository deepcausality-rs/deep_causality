/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Module, One, Ring};
use std::ops::{Mul, MulAssign};

/// Represents a Unital Algebra over a `Ring`.
///
/// In abstract algebra, an algebra is a vector space—or, more generally, a
/// module—equipped with a bilinear binary operation. This trait abstracts
/// over this concept.
///
/// This trait defines a **Unital Algebra** because it requires the `One` trait,
/// which provides a multiplicative identity (`1`).
///
/// # Mathematical Definition
///
/// An algebra `A` over a commutative ring `R` is a module over `R` that is also
/// a ring itself, where the ring multiplication is R-bilinear. This implementation
/// is slightly more general, as `R` is only required to be a `Ring`, not necessarily
/// commutative.
///
/// An algebra is **unital** if it has a multiplicative identity element.
///
/// ## Structure:
/// 1. `Self` is a `Module` over a scalar `Ring` `R`. This provides vector addition
///    and scalar multiplication.
/// 2. `Self` has a binary operation `*` (multiplication) that is compatible with
///    the module structure.
/// 3. `Self` has a multiplicative identity `1` (from the `One` trait).
///
/// ## Note:
/// This trait does *not* require the algebra to be associative. For that, see
/// the `AssociativeAlgebra` trait.
pub trait Algebra<R: Ring>: Module<R> + Mul<Output = Self> + MulAssign + One {
    /// Computes the square of an element.
    ///
    /// This is a convenience method equivalent to `self * self`.
    ///
    /// # Returns
    ///
    /// The result of multiplying `self` by itself.
    ///
    /// # Example
    /// ```
    /// use deep_causality_num::{Algebra, AssociativeRing, DivisionAlgebra, Field, RealField};
    /// let x = 2.0f64;
    /// // For real numbers, sqr() is the standard square.
    /// assert_eq!(<f64 as Algebra<f64>>::sqr(&x), 4.0);
    /// ```
    fn sqr(&self) -> Self {
        self.clone() * self.clone()
    }
}

// Blanket implementation
impl<T, R> Algebra<R> for T
where
    T: Module<R> + Mul<Output = Self> + MulAssign + One,
    R: Ring,
{
}
