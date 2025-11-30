/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Algebra, Field, One, Zero};
use std::ops::{Div, DivAssign};

/// A Division Algebra is an `Algebra` where every non-zero element
/// has a multiplicative inverse.
///
/// This implies that division by any non-zero element is well-defined.
/// It does *not* require associativity (e.g., Octonions).
///
/// ## Requirements:
/// 1. `Self` forms an `Algebra` over a `Field`.
/// 2. `Self` has a multiplicative identity (`One`).
/// 3. Every non-zero element has an inverse.
pub trait DivisionAlgebra<R: Field>:
    Algebra<R> + Div<Output = Self> + DivAssign + One + Zero + Clone
{
    /// Returns the multiplicative inverse of `self`.
    fn inverse(&self) -> Self;
}
