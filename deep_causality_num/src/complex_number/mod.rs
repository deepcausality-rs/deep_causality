/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Float;
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

pub trait ComplexNumber<F>: Sized
where
    F: Float,
    Self: Add<Output = Self>
        + Sub<Output = Self>
        + Mul<Output = Self>
        + Div<Output = Self>
        + Rem<Output = Self>
        + Neg<Output = Self>
        + PartialEq
        + Copy
        + Clone,
{
    /// Returns the real part of the complex number.
    fn re(&self) -> F;

    /// Returns the imaginary part of the complex number.
    fn im(&self) -> F;

    /// Computes the squared norm (magnitude squared) of the complex number.
    fn norm_sqr(&self) -> F;

    /// Computes the norm (magnitude or absolute value) of the complex number.
    fn norm(&self) -> F;

    /// Computes the argument (phase angle) of the complex number.
    fn arg(&self) -> F;

    /// Computes the complex conjugate of the complex number.
    fn conj(&self) -> Self;
}
