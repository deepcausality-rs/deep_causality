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
        // The non-finite components are decided before the ordering and before the ratio, because
        // the scaled form gets both wrong if they reach it.
        //
        // A `NaN` cannot be ranked: `NaN > 0` is false, so a `NaN` beside a zero would be sorted
        // into `min` and the zero-maximum guard below would return a finite **zero** — swallowing
        // it. That is worse than an unhelpful number, because a residual compared against a
        // tolerance would then report no defect at all.
        //
        // Two infinities give `∞/∞` for the ratio, so a modulus that is genuinely infinite would
        // come back `NaN`.
        //
        // Both answers here agree with what `modulus_squared` gives, which is the property that
        // keeps the two members of this trait consistent.
        if a.is_nan() || b.is_nan() {
            return T::nan();
        }
        if a.is_infinite() {
            return a;
        }
        if b.is_infinite() {
            return b;
        }
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
