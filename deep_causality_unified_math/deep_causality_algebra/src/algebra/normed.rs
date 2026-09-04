/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::RealField;

/// A value with a real modulus and real scaling: the bridge that lets generic code treat a real
/// (`f64`) and a complex (`Complex<f64>`) uniformly for norm work.
///
/// Unlike [`DivisionAlgebra`](crate::DivisionAlgebra), which *parameterizes* the real scalar
/// (`DivisionAlgebra<R>`), `Normed` makes the real an **associated** type. A generic carrier can
/// then read `T::Real` without threading a second type parameter — which is what lets a downstream
/// norm expose a single `type Output = T::Real` and be written as one blanket implementation
/// instead of one impl per concrete scalar.
pub trait Normed {
    /// The underlying real type (`f64` for both `f64` and `Complex<f64>`).
    type Real: RealField;

    /// The squared modulus as a real: `x → x²` for reals, `z → re² + im²` for complex.
    fn modulus_squared(&self) -> Self::Real;

    /// The modulus as a real: `|x|` for reals, `|z|` for complex.
    ///
    /// # Why this is a member and not `modulus_squared().sqrt()`
    ///
    /// That expression is what the default does, and it overflows where the answer does not.
    /// `|1e308|` is representable in `f64` and `(1e308)²` is not, so the square reaches infinity
    /// and the square root stays there. The same holds at the bottom: for `1e-200` the square
    /// underflows to zero and the modulus comes back as zero.
    ///
    /// Every implementor here overrides it with a form that does not go through the square, so
    /// the default is a fallback for a carrier that has no better route rather than the path
    /// anything in this workspace takes.
    ///
    /// Use [`modulus_squared`](Self::modulus_squared) when the square is what you want — a
    /// comparison of magnitudes, or a sum that is going to be square-rooted once at the end.
    /// Use this when you want the magnitude itself.
    #[inline]
    fn modulus(&self) -> Self::Real {
        use crate::Real;
        self.modulus_squared().sqrt()
    }

    /// Scale by a real.
    fn scale_by_real(&self, s: Self::Real) -> Self;
}

/// A real field element is its own real type; its squared modulus is `x²` and scaling is plain
/// multiplication. Bounding on [`RealField`](RealField) covers every primitive float
/// (`f32` / `f64` / `BFloat16` / `Float106`, via the `impl<T: Float> RealField for T` tower) in one blanket — no
/// per-type impls, no macro. `Complex` is unordered, hence not a `RealField`, so this does not
/// overlap the `Complex<T>` impl below; `num` can prove that disjointness because it owns
/// `RealField` and `Complex` together (a downstream crate could not).
impl<T: RealField> Normed for T {
    type Real = T;

    #[inline]
    fn modulus_squared(&self) -> T {
        *self * *self
    }

    /// `|x|`, without forming `x²`. Exact at every magnitude the type represents.
    #[inline]
    fn modulus(&self) -> T {
        crate::Real::abs(*self)
    }

    #[inline]
    fn scale_by_real(&self, s: T) -> T {
        *self * s
    }
}
