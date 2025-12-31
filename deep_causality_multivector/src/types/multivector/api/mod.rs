/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
mod api_mlx;

#[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
mod api_cpu;

// No need to re-export impls, they are automatically applied when module is linked.
// We DO need to make sure the trait defs are available if used in impls below?
// MultiVector trait is defined in crate::traits.
// Here we implement MultiVectorL2Norm and ScalarEval which are shared.

use crate::CausalMultiVector;
use crate::{MultiVectorL2Norm, ScalarEval};
use deep_causality_num::Complex;
use deep_causality_num::{Field, One, RealField, Zero};
use std::iter::Sum;

impl<T> MultiVectorL2Norm<T> for CausalMultiVector<T>
where
    // T must satisfy Field (required by the trait definition)
    // AND ScalarEval (required by our implementation logic)
    T: Field + Copy + Sum + ScalarEval,
{
    // The output of a Norm is always Real (e.g., f64), even if T is Complex.
    type Output = T::Real;

    fn norm_l2(&self) -> Self::Output {
        let sum_sq = self
            .data
            .iter()
            .map(|x| x.modulus_squared()) // Works for f64 AND Complex
            .fold(T::Real::zero(), |acc, x| acc + x);

        sum_sq.sqrt()
    }

    fn normalize_l2(&self) -> Self {
        let norm = self.norm_l2();

        if norm == T::Real::zero() {
            return self.clone();
        }

        // We scale by 1.0 / norm
        let scale_factor = T::Real::one() / norm;

        let new_data = self
            .data
            .iter()
            .map(|x| x.scale_by_real(scale_factor)) // Works for f64 AND Complex
            .collect();

        Self {
            data: new_data,
            metric: self.metric,
        }
    }
}

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
