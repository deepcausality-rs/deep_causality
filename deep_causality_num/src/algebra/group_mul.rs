/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::MulMonoid;
use core::ops::{Div, DivAssign};

/// Represents a **Multiplicative Group**.
///
/// A multiplicative group is a `Group` where the binary operation is
/// multiplication (`*`).
///
/// # Mathematical Definition
///
/// A set `G` is a group under multiplication if it satisfies:
/// 1.  **Closure:** `a * b` is in `G`. (Implicit in Rust).
/// 2.  **Associativity:** `(a * b) * c = a * (b * c)`. (Implied by `MulMonoid`).
/// 3.  **Identity Element:** There is an element `1` such that `a * 1 = a`.
///     (Provided by the `MulMonoid` -> `One` trait).
/// 4.  **Inverse Element:** For each `a`, there is an inverse `a⁻¹` such that
///     `a * a⁻¹ = 1`. (Provided by the `inverse()` method and `Div` trait).
///
/// In a `Field`, the set of all non-zero elements forms a multiplicative group.
pub trait MulGroup: MulMonoid + Div<Output = Self> + DivAssign {
    /// Computes the multiplicative inverse of an element.
    ///
    /// For a non-zero element `a`, its inverse `a⁻¹` is the unique element
    /// such that `a * a⁻¹ = 1`.
    ///
    /// # Justification for Deviation
    ///
    /// In pure mathematics, the zero element does not have a multiplicative
    /// inverse. To align with standard floating-point behavior (IEEE 754),
    /// implementations for types that can be zero (like `f32`, `f64`) should
    /// handle this case gracefully by returning `Infinity` or `NaN` rather
    /// than panicking.
    ///
    /// # Example
    /// ```
    /// use deep_causality_num::{MulGroup, RealField};
    /// let x = 2.0f64;
    /// let inv_x = x.inverse();
    /// assert_eq!(x * inv_x, 1.0);
    ///
    /// let z = 0.0f64;
    /// assert!(z.inverse().is_infinite());
    /// ```
    fn inverse(&self) -> Self;
}

impl MulGroup for f32 {
    /// Returns the multiplicative inverse (reciprocal) of the number.
    /// `1.0 / self`.
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}

impl MulGroup for f64 {
    /// Returns the multiplicative inverse (reciprocal) of the number.
    /// `1.0 / self`.
    fn inverse(&self) -> Self {
        1.0 / *self
    }
}
