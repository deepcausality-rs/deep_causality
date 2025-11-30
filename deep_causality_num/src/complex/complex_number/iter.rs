/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, RealField};
use std::iter::{Product, Sum};

// Implement Sum trait
impl<T: RealField> Sum for Complex<T> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(T::zero(), T::zero()), |acc, x| acc + x)
    }
}

// Implement Product trait
impl<T: RealField> Product for Complex<T> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::new(T::one(), T::zero()), |acc, x| acc * x)
    }
}
