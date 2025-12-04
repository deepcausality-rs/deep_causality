/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, MultiVectorL2Norm, ScalarEval};
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
