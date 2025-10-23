/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::float::Float;
use std::iter::{Product, Sum};
use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

mod arithmetic;
mod arithmetic_assign;
mod arithmetic_complex;
mod as_primitive;
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

/// Represents a complex number with real and imaginary parts.
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Complex<F>
where
    F: Float,
{
    pub re: F,
    pub im: F,
}
