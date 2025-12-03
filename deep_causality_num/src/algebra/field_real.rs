/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AbelianGroup, DivisionAlgebra, Field};
use core::cmp::PartialOrd;
use core::ops::{AddAssign, DivAssign, MulAssign, Neg, SubAssign};

/// Represents an ordered `Field` with additional operations for calculus and analysis.
///
/// A `RealField` extends the algebraic definition of a `Field` with properties
/// and functions characteristic of the real numbers, such as ordering (`PartialOrd`)
/// and transcendental functions (`sin`, `exp`, `ln`, etc.).
///
/// This trait abstracts over concrete floating-point types like `f32` and `f64`,
/// and could be implemented for other types like dual numbers (for automatic
/// differentiation) or custom fixed-point types.
pub trait RealField:
    Field
    + PartialOrd
    + Neg<Output = Self>
    + Copy
    + Clone
    + AddAssign
    + SubAssign
    + MulAssign
    + DivAssign
{
    /// Returns the `NaN` value.
    fn nan() -> Self;

    fn clamp(self, min: Self, max: Self) -> Self;

    /// Computes the principal square root of a number.
    /// For negative numbers, it returns `NaN`.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(4.0f64.sqrt(), 2.0);
    /// ```
    fn sqrt(self) -> Self;

    /// Computes the absolute value of a number.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// let x = -4.0f64;
    /// assert_eq!(x.abs(), 4.0);
    /// ```
    fn abs(self) -> Self;

    /// Returns the largest integer less than or equal to a number.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(2.99f64.floor(), 2.0);
    /// ```
    fn floor(self) -> Self;

    /// Returns the smallest integer greater than or equal to a number.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(2.01f64.ceil(), 3.0);
    /// ```
    fn ceil(self) -> Self;

    /// Returns the nearest integer to a number. Rounds half-way cases away from 0.0.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(2.5f64.round(), 3.0);
    /// assert_eq!(-2.5f64.round(), -3.0);
    /// ```
    fn round(self) -> Self;

    /// Returns `e^(self)`, (the exponential function).
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// assert_eq!(1.0f64.exp(), consts::E);
    /// ```
    fn exp(self) -> Self;

    /// Returns the natural logarithm of the number.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// assert_eq!(consts::E.ln(), 1.0);
    /// ```
    fn ln(self) -> Self;

    /// Returns the logarithm of the number with respect to an arbitrary base.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(8.0f64.log(2.0), 3.0);
    /// ```
    fn log(self, base: Self) -> Self;

    /// Raises a number to a floating point power.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert_eq!(2.0f64.powf(3.0), 8.0);
    /// ```
    fn powf(self, n: Self) -> Self;

    /// Computes the sine of a number (in radians).
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// assert!((consts::PI / 2.0).sin() - 1.0 < 1e-9);
    /// ```
    fn sin(self) -> Self;

    /// Computes the cosine of a number (in radians).
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// assert!((consts::PI.cos() - (-1.0)).abs() < 1e-9);
    /// ```
    fn cos(self) -> Self;

    fn acos(self) -> Self;

    /// Computes the tangent of a number (in radians).
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// assert!((consts::FRAC_PI_4.tan() - 1.0).abs() < 1e-9);
    /// ```
    fn tan(self) -> Self;

    /// Computes the hyperbolic sine of a number.
    fn sinh(self) -> Self;

    /// Computes the hyperbolic cosine of a number.
    fn cosh(self) -> Self;

    /// Computes the hyperbolic tangent of a number.
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// assert!(0.0f64.tanh() == 0.0);
    /// ```
    fn tanh(self) -> Self;

    /// Computes the four-quadrant arctangent of `self` (y) and `other` (x).
    /// # Example
    /// ```
    /// use deep_causality_num::RealField;
    /// use std::f64::consts;
    /// let y = 1.0f64;
    /// let x = -1.0f64;
    /// assert!((y.atan2(x) - (3.0 * consts::PI / 4.0)).abs() < 1e-9);
    /// ```
    fn atan2(self, other: Self) -> Self;

    /// Returns the constant π.
    fn pi() -> Self;

    /// Returns the Euler number e, the base of the natural logarithm.
    fn e() -> Self;

    /// Machine epsilon.
    /// Used for comparison comparisons to zero.
    fn epsilon() -> Self; // 1.19e-7 (f32) or 2.22e-16 (f64)
}

impl AbelianGroup for f32 {}
impl AbelianGroup for f64 {}

// CommutativeRing is derived automatically

// Division Algebra (Specific Implementation) ---
// There is no blanket for this, so we implement it manually.

/// Implements `DivisionAlgebra` for `f32`, where the base field is `f32` itself.
impl DivisionAlgebra<f32> for f32 {
    /// The conjugate of a real number is itself.
    #[inline]
    fn conjugate(&self) -> Self {
        *self
    }

    /// The squared norm of a real number `x` is `x*x`.
    #[inline]
    fn norm_sqr(&self) -> f32 {
        *self * *self
    }

    /// The inverse of a real number `x` is `1/x`.
    /// Returns `inf` if `x` is `0.0`.
    #[inline]
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}

/// Implements `DivisionAlgebra` for `f64`, where the base field is `f64` itself.
impl DivisionAlgebra<f64> for f64 {
    /// The conjugate of a real number is itself.
    #[inline]
    fn conjugate(&self) -> Self {
        *self
    }

    /// The squared norm of a real number `x` is `x*x`.
    #[inline]
    fn norm_sqr(&self) -> f64 {
        *self * *self
    }

    /// The inverse of a real number `x` is `1/x`.
    /// Returns `inf` if `x` is `0.0`.
    #[inline]
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}

impl RealField for f32 {
    #[inline]
    fn nan() -> Self {
        f32::NAN
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        f32::clamp(self, min, max)
    }

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        return self.sqrt();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sqrtf(self);
    }

    #[inline]
    fn floor(self) -> Self {
        #[cfg(feature = "std")]
        return self.floor();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::floorf(self);
    }

    #[inline]
    fn ceil(self) -> Self {
        #[cfg(feature = "std")]
        return self.ceil();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::ceilf(self);
    }

    #[inline]
    fn round(self) -> Self {
        #[cfg(feature = "std")]
        return self.round();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::roundf(self);
    }

    #[inline]
    fn exp(self) -> Self {
        #[cfg(feature = "std")]
        return self.exp();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::expf(self);
    }

    #[inline]
    fn ln(self) -> Self {
        #[cfg(feature = "std")]
        return self.ln();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::logf(self);
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        #[cfg(feature = "std")]
        return self.log(base);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::logf(self) / libm::logf(base);
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        #[cfg(feature = "std")]
        return self.powf(n);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::powf(self, n);
    }

    #[inline]
    fn sin(self) -> Self {
        #[cfg(feature = "std")]
        return self.sin();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sinf(self);
    }

    #[inline]
    fn cos(self) -> Self {
        #[cfg(feature = "std")]
        return self.cos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cosf(self);
    }

    #[inline]
    fn acos(self) -> Self {
        #[cfg(feature = "std")]
        return self.acos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::acosf(self);
    }

    #[inline]
    fn tan(self) -> Self {
        #[cfg(feature = "std")]
        return self.tan();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tanf(self);
    }

    #[inline]
    fn sinh(self) -> Self {
        #[cfg(feature = "std")]
        return self.sinh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sinhf(self);
    }

    #[inline]
    fn cosh(self) -> Self {
        #[cfg(feature = "std")]
        return self.cosh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::coshf(self);
    }

    #[inline]
    fn tanh(self) -> Self {
        #[cfg(feature = "std")]
        return self.tanh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tanhf(self);
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return self.atan2(other);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::atan2f(self, other);
    }

    #[inline]
    fn pi() -> Self {
        core::f32::consts::PI
    }

    #[inline]
    fn e() -> Self {
        core::f32::consts::E
    }

    #[inline]
    fn epsilon() -> Self {
        f32::EPSILON
    }
}

// -----------------------------------------------------------------------------
// f64 Implementation
// -----------------------------------------------------------------------------
impl RealField for f64 {
    #[inline]
    fn nan() -> Self {
        f64::NAN
    }

    #[inline]
    fn clamp(self, min: Self, max: Self) -> Self {
        f64::clamp(self, min, max)
    }

    #[inline]
    fn abs(self) -> Self {
        self.abs()
    }

    #[inline]
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        return self.sqrt();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sqrt(self);
    }

    #[inline]
    fn floor(self) -> Self {
        #[cfg(feature = "std")]
        return self.floor();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::floor(self);
    }

    #[inline]
    fn ceil(self) -> Self {
        #[cfg(feature = "std")]
        return self.ceil();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::ceil(self);
    }

    #[inline]
    fn round(self) -> Self {
        #[cfg(feature = "std")]
        return self.round();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::round(self);
    }

    #[inline]
    fn exp(self) -> Self {
        #[cfg(feature = "std")]
        return self.exp();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::exp(self);
    }

    #[inline]
    fn ln(self) -> Self {
        #[cfg(feature = "std")]
        return self.ln();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log(self);
    }

    #[inline]
    fn log(self, base: Self) -> Self {
        #[cfg(feature = "std")]
        return self.log(base);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::log(self) / libm::log(base);
    }

    #[inline]
    fn powf(self, n: Self) -> Self {
        #[cfg(feature = "std")]
        return self.powf(n);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::pow(self, n);
    }

    #[inline]
    fn sin(self) -> Self {
        #[cfg(feature = "std")]
        return self.sin();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sin(self);
    }

    #[inline]
    fn cos(self) -> Self {
        #[cfg(feature = "std")]
        return self.cos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cos(self);
    }

    #[inline]
    fn acos(self) -> Self {
        #[cfg(feature = "std")]
        return self.acos();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::acos(self);
    }

    #[inline]
    fn tan(self) -> Self {
        #[cfg(feature = "std")]
        return self.tan();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tan(self);
    }

    #[inline]
    fn sinh(self) -> Self {
        #[cfg(feature = "std")]
        return self.sinh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::sinh(self);
    }

    #[inline]
    fn cosh(self) -> Self {
        #[cfg(feature = "std")]
        return self.cosh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::cosh(self);
    }

    #[inline]
    fn tanh(self) -> Self {
        #[cfg(feature = "std")]
        return self.tanh();

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::tanh(self);
    }

    #[inline]
    fn atan2(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return self.atan2(other);

        #[cfg(all(not(feature = "std"), feature = "libm_math"))]
        return libm::atan2(self, other);
    }

    #[inline]
    fn pi() -> Self {
        core::f64::consts::PI
    }

    #[inline]
    fn e() -> Self {
        core::f64::consts::E
    }

    #[inline]
    fn epsilon() -> Self {
        f64::EPSILON
    }
}
