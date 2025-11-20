//! This module defines the `Octonion` struct and its core implementations, providing
//! a robust representation and arithmetic for octonion numbers.
//!
//! # Octonion Numbers
//! Octonions are a non-associative, non-commutative extension of quaternion numbers,
//! forming an 8-dimensional normed division algebra over the real numbers.
//! An octonion can be expressed in the form:
//!
//! `s + e₁i + e₂j + e₃k + e₄l + e₅m + e₆n + e₇p`
//!
//! where `s` is the scalar part and `e₁` through `e₇` are the imaginary (vector) parts,
//! each multiplied by an imaginary unit. These imaginary units satisfy specific
//! multiplication rules, which are typically represented by a Fano plane or a
//! multiplication table.
//!
//! # Cayley-Dickson Construction
//! The multiplication rules for octonions implemented in this module follow the
//! Cayley-Dickson construction. This construction defines a doubling process
//! that generates complex numbers from real numbers, quaternions from complex numbers,
//! and octonions from quaternions. Key properties of octonion multiplication include:
//!
//! - **Non-commutativity:** For two octonions `a` and `b`, `a * b ≠ b * a` in general.
//! - **Non-associativity:** For three octonions `a`, `b`, and `c`, `(a * b) * c ≠ a * (b * c)` in general.
//!
//! Despite these differences from more familiar number systems, octonions retain
//! properties such as distributivity over addition and an inverse for every non-zero octonion.
//!
//! The multiplication table, based on the Fano plane, defines how the imaginary units
//! multiply. A common representation (used here) is:
//!
//! - `eᵢ * eᵢ = -1` for `i = 1..7`
//! - `e₁ * e₂ = e₃`, `e₂ * e₁ = -e₃`
//! - `e₁ * e₃ = -e₂`, `e₃ * e₁ = e₂`
//! - `e₆ * e₇ = e₁`, `e₇ * e₆ = -e₁`
//!
//! These rules are meticulously applied in the `Mul` implementation for the `Octonion` struct.
//!
//! # Structure
//! The `Octonion` struct is a generic representation, parameterized by a floating-point type `F`.
//! It holds eight components: `s` (scalar) and `e1` through `e7` (imaginary units).
//!
//! The `OctonionNumber` trait defines a set of common operations applicable to octonions,
//! such as `conjugate`, `norm_sqr`, `norm`, `normalize`, `inverse`, and `dot` product.
//!
//! # Examples
//!
//! ```
//! use deep_causality_num::{Octonion, OctonionNumber};
//! use deep_causality_num::{One, Zero};
//!
//! // Create an octonion
//! let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
//! let o2 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
//!
//! // Addition
//! let sum = o1 + o2;
//! assert_eq!(sum.s, 10.0);
//!
//! // Multiplication (non-commutative)
//! let e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
//! let e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
//! let e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
//!
//! assert_eq!(e1 * e2, e3);
//! assert_eq!(e2 * e1, -e3); // Non-commutative
//!
//! // Identity
//! let identity_octonion: Octonion<f64> = Octonion::identity();
//! assert_eq!(identity_octonion.s, 1.0);
//! assert!(identity_octonion.e1.is_zero());
//!
//! // Norm
//! let o = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Scalar 1
//! assert_eq!(o.norm(), 1.0);
//! ```

use crate::{Float, Num};
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

mod arithmetic;
mod arithmetic_assign;
mod as_primitive;
mod constructors;
mod debug;
mod display;
mod from_primitives;
mod identity;
mod neg;
mod num_cast;
mod octonion_number_impl;
mod part_ord;
mod to_primitive;

/// A trait defining common operations for octonion numbers.
///
/// This trait provides an abstraction for various mathematical operations that
/// can be performed on octonion numbers, ensuring a consistent interface.
pub trait OctonionNumber<F>: Num + Sized
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
    /// Computes the conjugate of the octonion.
    ///
    /// The conjugate of an octonion `s + e₁i + ... + e₇p` is `s - e₁i - ... - e₇p`.
    ///
    /// # Returns
    /// A new octonion representing the conjugate of `self`.
    fn conjugate(&self) -> Self;

    /// Computes the square of the norm (magnitude) of the octonion.
    ///
    /// The norm squared is calculated as `s² + e₁² + ... + e₇²`.
    ///
    /// # Returns
    /// The scalar value `F` representing the squared norm.
    fn norm_sqr(&self) -> F;

    /// Computes the norm (magnitude) of the octonion.
    ///
    /// The norm is the square root of the sum of the squares of all its components.
    ///
    /// # Returns
    /// The scalar value `F` representing the norm.
    fn norm(&self) -> F;

    /// Returns a normalized version of the octonion (a unit octonion).
    ///
    /// If the norm of the octonion is zero, it returns the original octonion
    /// to prevent division by zero, though this might result in a non-unit octonion.
    ///
    /// # Returns
    /// A new octonion with a norm of 1, or `self` if its norm is zero.
    fn normalize(&self) -> Self;

    /// Computes the inverse of the octonion.
    ///
    /// The inverse `o⁻¹` is defined such that `o * o⁻¹ = o⁻¹ * o = 1`.
    /// For a non-zero octonion `o`, `o⁻¹ = conjugate(o) / norm_sqr(o)`.
    /// If the norm squared is zero, it returns an octonion with NaN components.
    ///
    /// # Returns
    /// A new octonion representing the inverse of `self`.
    fn inverse(&self) -> Self;

    /// Computes the dot product of `self` with another octonion `other`.
    ///
    /// The dot product is the sum of the products of corresponding components:
    /// `s*other.s + e₁*other.e₁ + ... + e₇*other.e₇`.
    ///
    /// # Arguments
    /// * `other` - A reference to another `Octonion` with which to compute the dot product.
    ///
    /// # Returns
    /// The scalar value `F` representing the dot product.
    fn dot(&self, other: &Self) -> F;
}

/// Represents an octonion number with a scalar part and seven imaginary parts.
///
/// An octonion is an 8-dimensional hypercomplex number, extending quaternions.
/// It consists of a scalar component `s` and seven imaginary components `e1` through `e7`.
///
/// # Type Parameters
/// * `F` - The floating-point type used for the components of the octonion (e.g., `f32`, `f64`).
///
/// # Fields
/// * `s` - The scalar (real) part of the octonion.
/// * `e1` - The coefficient of the first imaginary unit.
/// * `e2` - The coefficient of the second imaginary unit.
/// * `e3` - The coefficient of the third imaginary unit.
/// * `e4` - The coefficient of the fourth imaginary unit.
/// * `e5` - The coefficient of the fifth imaginary unit.
/// * `e6` - The coefficient of the sixth imaginary unit.
/// * `e7` - The coefficient of the seventh imaginary unit.
///
/// # Derives
/// * `Copy`: Allows instances of `Octonion` to be copied by bitwise copy.
/// * `Clone`: Implements cloning behavior for `Octonion`.
/// * `Default`: Provides a default (zero) value for `Octonion`.
#[derive(Copy, Clone, Default)]
pub struct Octonion<F>
where
    F: Float,
{
    pub s: F,  // Scalar part
    pub e1: F, // Vector part 1
    pub e2: F, // Vector part 2
    pub e3: F, // Vector part 3
    pub e4: F, // Vector part 4
    pub e5: F, // Vector part 5
    pub e6: F, // Vector part 6
    pub e7: F, // Vector part 7
}

/// Implements the `Num` marker trait for `Octonion`, indicating it behaves like a number.
impl<F: Float> Num for Octonion<F> {}
