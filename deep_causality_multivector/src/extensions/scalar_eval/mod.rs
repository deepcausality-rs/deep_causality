/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ScalarEval;
use deep_causality_num::{Complex, DoubleFloat, RealField};
use std::iter::Sum;

impl ScalarEval for f32 {
    type Real = f32;

    #[inline]
    fn modulus_squared(&self) -> Self::Real {
        *self * *self
    }

    #[inline]
    fn scale_by_real(&self, s: Self::Real) -> Self {
        *self * s
    }
}

impl ScalarEval for f64 {
    type Real = f64;

    #[inline]
    fn modulus_squared(&self) -> Self::Real {
        *self * *self
    }

    #[inline]
    fn scale_by_real(&self, s: Self::Real) -> Self {
        *self * s
    }
}

impl ScalarEval for DoubleFloat {
    type Real = DoubleFloat;

    #[inline]
    fn modulus_squared(&self) -> Self::Real {
        *self * *self
    }

    #[inline]
    fn scale_by_real(&self, s: Self::Real) -> Self {
        *self * s
    }
}
impl<T> ScalarEval for Complex<T>
where
    T: RealField + Copy + Sum,
{
    type Real = T;

    #[inline]
    fn modulus_squared(&self) -> T {
        // |z|^2 = re^2 + im^2
        (self.re * self.re) + (self.im * self.im)
    }

    #[inline]
    fn scale_by_real(&self, s: T) -> Self {
        // Scalar multiplication: (re * s, im * s)
        Complex::new(self.re * s, self.im * s)
    }
}
