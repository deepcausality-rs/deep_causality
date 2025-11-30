/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AbelianGroup, Ring};
use core::ops::{Mul, MulAssign};

/// A Module is a generalization of a Vector Space.
/// It consists of an Abelian Group (the vectors) and a Ring (the scalars).
///
/// R: The Scalar Ring (e.g., f64, Complex)
pub trait Module<R: Ring>: AbelianGroup + Mul<R, Output = Self> + MulAssign<R> {
    // Vectors can be scaled by scalars from the ring R
    fn scale(&self, scalar: R) -> Self {
        // 1. We must clone because `Mul` usually consumes `self` (value semantics),
        //    but `scale` takes `&self` (reference semantics).
        // 2. We know V is Clone because AbelianGroup -> AddGroup -> AddMonoid -> Clone.
        self.clone() * scalar
    }

    // In-place scaling without allocation (for heavy types like large Tensors), you typically rely on the MulAssign
    fn scale_mut(&mut self, scalar: R) {
        *self *= scalar; // Uses MulAssign
    }
}

// Blanket implementation for any type that satisfies the bounds
impl<V, R> Module<R> for V
where
    V: AbelianGroup + Mul<R, Output = V> + MulAssign<R>,
    R: Ring,
{
}
