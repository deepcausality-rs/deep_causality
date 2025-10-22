/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ComplexNumber;
use crate::float::Float;

mod arithmetic;
mod as_primitive;
mod assign;
mod debug;
mod display;
mod float;
mod identity;
mod neg;
mod num;
mod num_cast;
mod part_ord;
mod to_primitive;

/// Represents a complex number with real and imaginary parts.
#[derive(Copy, Clone, PartialEq, Default)]
pub struct Complex<F>
where
    F: Float,
{
    pub re: F,
    pub im: F,
}

impl<F> Complex<F>
where
    F: Float,
{
    #[inline]
    pub fn new(re: F, im: F) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn from_real(re: F) -> Self {
        Self { re, im: F::zero() }
    }
}
impl<F> ComplexNumber<F> for Complex<F>
where
    F: Float,
{
    #[inline]
    fn re(&self) -> F {
        self.re
    }

    #[inline]
    fn im(&self) -> F {
        self.im
    }

    #[inline]
    fn norm_sqr(&self) -> F {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    fn norm(&self) -> F {
        self.norm_sqr().sqrt()
    }

    #[inline]
    fn arg(&self) -> F {
        self.im.atan2(self.re)
    }

    #[inline]
    fn conj(&self) -> Self {
        Self::new(self.re, -self.im)
    }
}
