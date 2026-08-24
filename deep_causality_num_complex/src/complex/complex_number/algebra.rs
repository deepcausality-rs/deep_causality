/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Annihilating, Associative, Commutative, Complex, ComplexField, Distributive, DivisionAlgebra,
    Invertible, RealField,
};
use deep_causality_algebra::IntegralDomain;
use deep_causality_algebra::{Additive, Multiplicative};
// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
// | :--- | :---: | :---: | :---: | :--- |
// | **Complex** | ✅ | ✅ | ✅ | `Field`  `ComplexField` |

// Marker Traits
impl<T: RealField> Associative<Multiplicative> for Complex<T> {}
// Componentwise addition, so the additive laws come straight from the scalar.
impl<T: RealField> Associative<Additive> for Complex<T> {}
impl<T: RealField> Commutative<Additive> for Complex<T> {}
impl<T: RealField> Commutative<Multiplicative> for Complex<T> {}
impl<T: RealField> Distributive for Complex<T> {}
// Zero annihilates: the law is derivable here, but the marker is stated because `Semiring`
// requires it and cannot derive it (see `Annihilating`).
impl<T: RealField> Annihilating for Complex<T> {}
// Reached through the `AbelianGroup` blanket now that the additive markers are present.
// ℂ is a field: every non-zero `z` has `z⁻¹ = z̄ / |z|²`, so `Div` really does invert.
impl<T: RealField> Invertible for Complex<T> {}

// The blanket impls for Ring, Field, and AssociativeDivisionAlgebra
// will apply automatically as Complex<T> now satisfies their super-traits.

// Implement all methods for DivisionAlgebra, delegating to inherent methods.
impl<T: RealField> DivisionAlgebra<T> for Complex<T> {
    fn conjugate(&self) -> Self {
        self._conjugate_impl()
    }

    fn norm_sqr(&self) -> T {
        self._norm_sqr_impl()
    }

    fn inverse(&self) -> Self {
        self._inverse_impl()
    }
}

// Implement ComplexField for Complex<T>
impl<T: RealField> ComplexField<T> for Complex<T> {
    #[inline]
    fn real(&self) -> T {
        self.re
    }

    #[inline]
    fn imag(&self) -> T {
        self.im
    }

    #[inline]
    fn conjugate(&self) -> Self {
        self._conjugate_impl()
    }

    #[inline]
    fn norm_sqr(&self) -> T {
        self._norm_sqr_impl()
    }

    #[inline]
    fn norm(&self) -> T {
        self._norm_sqr_impl().sqrt()
    }

    #[inline]
    fn arg(&self) -> T {
        self.im.atan2(self.re)
    }

    #[inline]
    fn from_re_im(re: T, im: T) -> Self {
        Self::new(re, im)
    }

    #[inline]
    fn from_polar(r: T, theta: T) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    #[inline]
    fn i() -> Self {
        Self::new(T::zero(), T::one())
    }

    #[inline]
    fn is_real(&self) -> bool {
        self.im.is_zero()
    }

    #[inline]
    fn is_imaginary(&self) -> bool {
        self.re.is_zero()
    }
}

// ℂ is a field, so it has no zero divisors.
impl<T: RealField> IntegralDomain for Complex<T> {}
