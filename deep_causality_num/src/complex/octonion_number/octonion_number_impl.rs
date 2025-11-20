/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::{Octonion, OctonionNumber};

use crate::float::Float;

impl<F> OctonionNumber<F> for Octonion<F>
where
    F: Float,
{
    /// Computes the conjugate of the octonion.
    ///
    /// The conjugate of an octonion `s + e₁i + ... + e₇p` is `s - e₁i - ... - e₇p`.
    /// The scalar part remains unchanged, while all imaginary parts are negated.
    ///
    /// # Returns
    /// A new octonion representing the conjugate of `self`.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber};
    ///
    /// let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    /// let conj_o = o.conjugate();
    /// assert_eq!(conj_o.s, 1.0);
    /// assert_eq!(conj_o.e1, -2.0);
    /// assert_eq!(conj_o.e7, -8.0);
    /// ```
    fn conjugate(&self) -> Self {
        Self {
            s: self.s,
            e1: -self.e1,
            e2: -self.e2,
            e3: -self.e3,
            e4: -self.e4,
            e5: -self.e5,
            e6: -self.e6,
            e7: -self.e7,
        }
    }

    /// Computes the square of the norm (magnitude) of the octonion.
    ///
    /// The norm squared is calculated as the sum of the squares of all its components:
    /// `s² + e₁² + e₂² + e₃² + e₄² + e₅² + e₆² + e₇²`.
    ///
    /// # Returns
    /// The scalar value `F` representing the squared norm.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber};
    ///
    /// let o = Octonion::new(1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(o.norm_sqr(), 5.0); // 1*1 + 2*2 = 5
    /// ```
    fn norm_sqr(&self) -> F {
        self.s * self.s
            + self.e1 * self.e1
            + self.e2 * self.e2
            + self.e3 * self.e3
            + self.e4 * self.e4
            + self.e5 * self.e5
            + self.e6 * self.e6
            + self.e7 * self.e7
    }

    /// Computes the norm (magnitude) of the octonion.
    ///
    /// The norm is the square root of the sum of the squares of all its components,
    /// which is `sqrt(norm_sqr())`.
    ///
    /// # Returns
    /// The scalar value `F` representing the norm.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber};
    ///
    /// let o = Octonion::new(3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(o.norm(), 5.0); // sqrt(3*3 + 4*4) = sqrt(9 + 16) = sqrt(25) = 5
    /// ```
    fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    /// Returns a normalized version of the octonion (a unit octonion).
    ///
    /// A unit octonion has a norm of 1. If the current octonion's norm is zero,
    /// the original octonion is returned to avoid division by zero, meaning
    /// `normalize()` on a zero octonion returns a zero octonion.
    ///
    /// # Returns
    /// A new `Octonion` with a norm of 1, or `self` if its norm is zero.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber, Zero};
    ///
    /// let o = Octonion::new(3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// let unit_o = o.normalize();
    /// assert!((unit_o.norm() - 1.0) < 1e-9);
    /// assert_eq!(unit_o.s, 0.6000000000000001); // 3/5
    /// assert_eq!(unit_o.e1, 0.8); // 4/5
    ///
    /// let zero_o = Octonion::<f64>::zero();
    /// assert_eq!(zero_o.normalize(), zero_o);
    /// ```
    fn normalize(&self) -> Self {
        let n = self.norm();
        if n.is_zero() { *self } else { *self / n }
    }

    /// Computes the inverse of the octonion.
    ///
    /// The inverse `o⁻¹` of an octonion `o` is defined as `conjugate(o) / norm_sqr(o)`.
    ///
    /// # Returns
    /// A new `Octonion` representing the inverse of `self`. If `norm_sqr()` is zero,
    /// an octonion with `NaN` components is returned to indicate an undefined inverse.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber, Zero, One};
    ///
    /// let o = Octonion::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1 + e1
    /// let inverse_o = o.inverse();
    /// // (1 + e1) * (0.5 - 0.5e1) = 0.5 - 0.5e1 + 0.5e1 - 0.5e1*e1 = 0.5 + 0.5 = 1
    /// let expected_inverse = Octonion::new(0.5, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    ///
    /// // Use approximate equality due to floating point arithmetic
    /// assert!((inverse_o.s - expected_inverse.s) < 1e-9);
    /// assert!((inverse_o.e1 - expected_inverse.e1) < 1e-9);
    ///
    /// let zero_o = Octonion::<f64>::zero();
    /// let inv_zero = zero_o.inverse();
    /// assert!(inv_zero.s.is_nan());
    /// ```
    fn inverse(&self) -> Self {
        let n_sqr = self.norm_sqr();
        if n_sqr.is_zero() {
            let nan = F::nan();
            Self::new(nan, nan, nan, nan, nan, nan, nan, nan)
        } else {
            self.conjugate() / n_sqr
        }
    }

    /// Computes the dot product of `self` with another octonion `other`.
    ///
    /// The dot product is the sum of the products of corresponding components:
    /// `s*other.s + e₁*other.e₁ + e₂*other.e₂ + ... + e₇*other.e₇`.
    ///
    /// # Arguments
    /// * `other` - A reference to another `Octonion` with which to compute the dot product.
    ///
    /// # Returns
    /// The scalar value `F` representing the dot product.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, OctonionNumber};
    ///
    /// let o1 = Octonion::new(1.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// let o2 = Octonion::new(3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(o1.dot(&o2), 1.0 * 3.0 + 2.0 * 4.0); // 3 + 8 = 11
    /// ```
    fn dot(&self, other: &Self) -> F {
        self.s * other.s
            + self.e1 * other.e1
            + self.e2 * other.e2
            + self.e3 * other.e3
            + self.e4 * other.e4
            + self.e5 * other.e5
            + self.e6 * other.e6
            + self.e7 * other.e7
    }
}
