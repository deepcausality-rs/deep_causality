/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{AbelianGroup, CommutativeRing, DivisionAlgebra, Field};
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

impl CommutativeRing for f32 {}
impl CommutativeRing for f64 {}

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
    fn nan() -> Self {
        f32::NAN
    }
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
    fn exp(self) -> Self {
        self.exp()
    }
    fn ln(self) -> Self {
        self.ln()
    }
    fn log(self, base: Self) -> Self {
        self.log(base)
    }
    fn powf(self, n: Self) -> Self {
        self.powf(n)
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn sinh(self) -> Self {
        self.sinh()
    }
    fn cosh(self) -> Self {
        self.cosh()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }
    fn pi() -> Self {
        std::f32::consts::PI
    }
    fn e() -> Self {
        std::f32::consts::E
    }
    fn epsilon() -> Self {
        f32::EPSILON
    }
}

impl RealField for f64 {
    fn nan() -> Self {
        f64::NAN
    }
    fn sqrt(self) -> Self {
        self.sqrt()
    }
    fn abs(self) -> Self {
        self.abs()
    }
    fn floor(self) -> Self {
        self.floor()
    }
    fn ceil(self) -> Self {
        self.ceil()
    }
    fn round(self) -> Self {
        self.round()
    }
    fn exp(self) -> Self {
        self.exp()
    }
    fn ln(self) -> Self {
        self.ln()
    }
    fn log(self, base: Self) -> Self {
        self.log(base)
    }
    fn powf(self, n: Self) -> Self {
        self.powf(n)
    }
    fn sin(self) -> Self {
        self.sin()
    }
    fn cos(self) -> Self {
        self.cos()
    }
    fn tan(self) -> Self {
        self.tan()
    }
    fn sinh(self) -> Self {
        self.sinh()
    }
    fn cosh(self) -> Self {
        self.cosh()
    }
    fn tanh(self) -> Self {
        self.tanh()
    }
    fn atan2(self, other: Self) -> Self {
        self.atan2(other)
    }
    fn pi() -> Self {
        std::f64::consts::PI
    }
    fn e() -> Self {
        std::f64::consts::E
    }
    fn epsilon() -> Self {
        f64::EPSILON
    }
}
