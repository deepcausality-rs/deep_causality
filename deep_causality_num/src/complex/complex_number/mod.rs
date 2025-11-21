/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::float::Float;
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

mod arithmetic;
mod arithmetic_assign;
mod as_primitive;
mod complex_number_impl;
mod constructors;
mod debug;
mod display;
mod float;
mod from_primitives;
mod identity;
mod neg;
mod num_cast;
mod part_ord;
mod to_primitive;

/// A trait for types that represent complex numbers.
///
/// This trait defines the fundamental operations and properties expected of a complex number,
/// such as accessing its real and imaginary parts, computing its magnitude and argument,
/// and calculating its conjugate.
///
/// Implementors of this trait are expected to provide these operations for a generic float type `F`.
pub trait ComplexNumber<F>: Sized
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + Neg<Output = Self>
        + Sum
        + Product
        + PartialEq
        + Copy
        + Clone,
{
    /// Returns the real part of the complex number.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    ///
    /// let c = Complex::new(3.0, 4.0);
    /// assert_eq!(c.re(), 3.0);
    /// ```
    fn re(&self) -> F;

    /// Returns the imaginary part of the complex number.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    ///
    /// let c = Complex::new(3.0, 4.0);
    /// assert_eq!(c.im(), 4.0);
    /// ```
    fn im(&self) -> F;

    /// Computes the squared norm (magnitude squared) of the complex number.
    ///
    /// For a complex number `z = a + bi`, the squared norm is `a^2 + b^2`.
    /// This method avoids the square root operation, making it more efficient
    /// when only relative magnitudes are needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    ///
    /// let c = Complex::new(3.0, 4.0);
    /// assert_eq!(c.norm_sqr(), 25.0);
    /// ```
    fn norm_sqr(&self) -> F;

    /// Computes the norm (magnitude or absolute value) of the complex number.
    ///
    /// For a complex number `z = a + bi`, the norm is `sqrt(a^2 + b^2)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    ///
    /// let c = Complex::new(3.0, 4.0);
    /// assert_eq!(c.norm(), 5.0);
    /// ```
    fn norm(&self) -> F;

    /// Computes the argument (phase angle) of the complex number.
    ///
    /// The argument is the angle `theta` such that `z = r * (cos(theta) + i * sin(theta))`, where `r` is the norm.
    /// It is typically in the range `(-PI, PI]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    /// use std::f64::consts::FRAC_PI_4;
    ///
    /// let c = Complex::new(1.0, 1.0);
    /// assert!((c.arg() - FRAC_PI_4).abs() < 1e-9);
    /// ```
    fn arg(&self) -> F;

    /// Computes the complex conjugate of the complex number.
    ///
    /// For a complex number `z = a + bi`, the conjugate is `a - bi`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality_num::{Complex, ComplexNumber};
    ///
    /// let c = Complex::new(3.0, 4.0);
    /// let conj_c = c.conj();
    /// assert_eq!(conj_c.re(), 3.0);
    /// assert_eq!(conj_c.im(), -4.0);
    /// ```
    fn conj(&self) -> Self;
}

/// Represents a complex number with real and imaginary parts.
///
/// A complex number is a number that can be expressed in the form `a + bi`,
/// where `a` and `b` are real numbers, and `i` is the imaginary unit,
/// satisfying the equation `i^2 = -1`.
///
/// The `Complex` struct is generic over a float type `F`, allowing it to work
/// with different floating-point precisions (e.g., `f32` or `f64`).
///
/// # Fields
///
/// * `re`: The real part of the complex number.
/// * `im`: The imaginary part of the complex number.
///
/// # Examples
///
/// ```
/// use deep_causality_num::Complex;
///
/// let c1 = Complex::new(1.0, 2.0); // Represents 1.0 + 2.0i
/// let c2 = Complex { re: 3.0, im: -1.0 }; // Represents 3.0 - 1.0i
///
/// assert_eq!(c1.re, 1.0);
/// assert_eq!(c2.im, -1.0);
/// ```

#[derive(Copy, Clone, PartialEq, Default)]
pub struct Complex<F>
where
    F: Float,
{
    pub re: F,
    pub im: F,
}

pub type Complex32 = Complex<f32>;
pub type Complex64 = Complex<f64>;
