/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Complex, ComplexNumber, Float};

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
