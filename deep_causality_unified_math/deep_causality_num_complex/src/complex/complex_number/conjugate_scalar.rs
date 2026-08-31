/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Complex;
use deep_causality_algebra::{ComplexField, ConjugateScalar, RealField};
use deep_causality_num::FromPrimitive;

/// Complex scalars carry a genuine conjugation `a − bi` and a real modulus `re² + im²`; magnitudes
/// and singular values live in the underlying real type `T`.
impl<T: RealField + FromPrimitive> ConjugateScalar for Complex<T> {
    type Real = T;
    #[inline]
    fn conjugate(&self) -> Self {
        ComplexField::conjugate(self)
    }
    #[inline]
    fn modulus_squared(&self) -> T {
        ComplexField::norm_sqr(self)
    }
    /// Delegates to the [`Normed`](deep_causality_algebra::Normed) impl, which computes `|z|` by
    /// the scaled form and so does not overflow for a component near `T::MAX` or flush to zero for
    /// one near `T::MIN_POSITIVE`.
    /// One body for the two traits.
    #[inline]
    fn modulus(&self) -> T {
        deep_causality_algebra::Normed::modulus(self)
    }
    #[inline]
    fn real_part(&self) -> T {
        self.re()
    }
    #[inline]
    fn from_real(re: T) -> Self {
        Complex::new(re, T::zero())
    }
}
