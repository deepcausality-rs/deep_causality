/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Dual;
use deep_causality_algebra::{ConjugateScalar, Scalar};

/// `Dual` (forward-mode AD) is an ordered analytic extension of its real base: its conjugate is the
/// identity and its modulus carries the derivative (the real type is `Dual` itself, so singular
/// values differentiate).
impl<T: Scalar> ConjugateScalar for Dual<T> {
    type Real = Dual<T>;
    #[inline]
    fn conjugate(&self) -> Self {
        *self
    }
    #[inline]
    fn modulus_squared(&self) -> Dual<T> {
        *self * *self
    }
    /// `|x|`, without forming `x²`. `Real::abs` for a dual carries the derivative — `d/dx |x|` is
    /// `sign(x)` — so this differentiates as the modulus should, and it does not overflow for a
    /// real part near `T::MAX` the way squaring then rooting does.
    #[inline]
    fn modulus(&self) -> Dual<T> {
        deep_causality_algebra::Real::abs(*self)
    }
    #[inline]
    fn real_part(&self) -> Dual<T> {
        *self
    }
    #[inline]
    fn from_real(re: Dual<T>) -> Self {
        re
    }
}
