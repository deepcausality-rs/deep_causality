/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{AssociativeAlgebra, Field, MulGroup};

/// A Division Algebra is an `AssociativeAlgebra` where every non-zero element
/// has a multiplicative inverse.
///
/// This implies that division by any non-zero element is well-defined.
///
/// ## Requirements:
/// 1. `Self` forms an `AssociativeAlgebra` over a `Field` (its scalar field).
/// 2. `Self` forms a `MulGroup`, which guarantees a multiplicative inverse
///    for non-zero elements.
/// 3. Division (`/`) operation is available and consistent with the inverse.
pub trait DivisionAlgebra<R: Field>: AssociativeAlgebra<R> + MulGroup {}

// Blanket implementation for any type that satisfies the bounds
impl<T, R> DivisionAlgebra<R> for T
where
    T: AssociativeAlgebra<R> + MulGroup,
    R: Field,
{
}
