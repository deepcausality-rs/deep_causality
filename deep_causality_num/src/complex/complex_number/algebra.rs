/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    AbelianGroup, Associative, Commutative, Complex, Distributive, DivisionAlgebra, MulGroup,
    RealField, Zero,
};

// | Type | `Distributive` | `Associative` | `Commutative` | Trait |
// | :--- | :---: | :---: | :---: | :--- |
// | **Complex** | ✅ | ✅ | ✅ | `Field` |

// Marker Traits
impl<T: RealField> Associative for Complex<T> {}
impl<T: RealField> Commutative for Complex<T> {}
impl<T: RealField> Distributive for Complex<T> {}

impl<T: RealField> AbelianGroup for Complex<T> {}

// The blanket impls for AssociativeRing, Field, and AssociativeDivisionAlgebra
// will apply automatically as Complex<T> now satisfies their super-traits.

impl<T: RealField> MulGroup for Complex<T> {
    // Required by Field -> CommutativeRing -> Ring -> MulMonoid -> MulGroup
    // This delegates to the safe, inherent `inverse` method via the `Div` trait.
    fn inverse(&self) -> Self {
        self._inverse_impl()
    }
}

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

impl<T: RealField> Complex<T> {
    /// Computes the squared norm (magnitude squared) of the complex number.
    #[inline]
    pub(crate) fn _norm_sqr_impl(&self) -> T {
        self.re * self.re + self.im * self.im
    }
    /// Computes the complex conjugate of the complex number.
    #[inline]
    pub(crate) fn _conjugate_impl(&self) -> Self {
        Self::new(self.re, -self.im)
    }

    /// Computes the multiplicative inverse of an element.
    #[inline]
    pub(crate) fn _inverse_impl(&self) -> Self {
        if self.is_zero() {
            return Self::new(T::nan(), T::nan());
        }
        let inv_norm_sq = self.norm_sqr().inverse();
        Self {
            re: self.re * inv_norm_sq,
            im: -self.im * inv_norm_sq,
        }
    }
}
