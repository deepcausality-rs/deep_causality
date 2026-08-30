/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Complex;
use deep_causality_algebra::{Normed, RealField};

/// A complex scalar carries the real modulus `|z|² = re² + im²` and scales component-wise.
impl<T: RealField> Normed for Complex<T> {
    type Real = T;

    #[inline]
    fn modulus_squared(&self) -> T {
        (self.re * self.re) + (self.im * self.im)
    }

    /// `|z|`, by the scaled form rather than `sqrt(re² + im²)`.
    ///
    /// Factoring the larger component out gives `max · sqrt(1 + (min/max)²)`, where the ratio is
    /// in `[0, 1]` and the square cannot overflow. The direct form squares `re` and `im`, so a
    /// complex with a component near `T::MAX` returns infinity for a modulus that is
    /// representable, and one with components near `T::MIN_POSITIVE` returns zero.
    ///
    /// Written with `Real`'s own operations — `abs`, `sqrt`, comparison and division — because
    /// `RealField` does not imply `Float` and so carries no `hypot`. `Float106` reaches
    /// `RealField` through the `impl<T: Float> RealField for T` blanket and would be excluded by
    /// a `Float` bound here.
    #[inline]
    fn modulus(&self) -> T {
        let (a, b) = (self.re.abs(), self.im.abs());
        let (max, min) = if a > b { (a, b) } else { (b, a) };
        if max == T::zero() {
            return T::zero();
        }
        let ratio = min / max;
        max * (T::one() + ratio * ratio).sqrt()
    }

    #[inline]
    fn scale_by_real(&self, s: T) -> Self {
        Complex::new(self.re * s, self.im * s)
    }
}
