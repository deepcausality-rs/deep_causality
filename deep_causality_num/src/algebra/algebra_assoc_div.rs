/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AssociativeAlgebra, DivisionAlgebra, Field};

/// An Associative Division Algebra is a Division Algebra that is also associative.
///
/// Examples: Real numbers, Complex numbers, Quaternions.
/// Counter-example: Octonions (non-associative).
pub trait AssociativeDivisionAlgebra<R: Field>: DivisionAlgebra<R> + AssociativeAlgebra<R> {}

// Blanket implementation for AssociativeDivisionAlgebra
impl<T, R> AssociativeDivisionAlgebra<R> for T
where
    T: DivisionAlgebra<R> + AssociativeAlgebra<R>,
    R: Field,
{
}
