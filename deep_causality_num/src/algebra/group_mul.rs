/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::MulMonoid;
use core::ops::{Div, DivAssign};

/// A Multiplicative Group is a Monoid where every element (except zero in Fields)
/// has a multiplicative inverse (1/a).
///
/// Laws:
/// 1. Inverse: a * (1/a) = 1
pub trait MulGroup: MulMonoid + Div<Output = Self> + DivAssign {
    /// Returns the multiplicative inverse of `self`.
    ///
    /// For types that can have a zero element (like floating-point numbers),
    /// this method should handle the inverse of zero appropriately (e.g., return NaN or infinity).
    fn inverse(&self) -> Self;
}

impl MulGroup for f32 {
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}

impl MulGroup for f64 {
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}
