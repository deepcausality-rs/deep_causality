/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::CausalMultiVector;
use deep_causality_num::Num;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

/// Implements component-wise addition of multivectors.
/// $$ A + B = \sum_{I} (a_I + b_I) e_I $$
impl<T> Add for CausalMultiVector<T>
where
    T: Add<Output = T> + Copy + Clone + PartialEq,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!(
                "Dimension mismatch in addition: {:?} vs {:?}",
                self.metric, rhs.metric
            );
        }
        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        Self {
            data,
            metric: self.metric,
        }
    }
}

/// Implements component-wise subtraction of multivectors.
/// $$ A - B = \sum_{I} (a_I - b_I) e_I $$
impl<T> Sub for CausalMultiVector<T>
where
    T: Sub<Output = T> + Copy + Clone + PartialEq,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!(
                "Dimension mismatch in subtraction: {:?} vs {:?}",
                self.metric, rhs.metric
            );
        }
        let data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a - *b)
            .collect();

        Self {
            data,
            metric: self.metric,
        }
    }
}

/// Implements scalar multiplication.
/// $$ A s = \sum_{I} (a_I s) e_I $$
impl<T> Mul<T> for CausalMultiVector<T>
where
    T: Mul<Output = T> + Copy + Clone,
{
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let data = self.data.iter().map(|a| *a * rhs).collect();
        Self {
            data,
            metric: self.metric,
        }
    }
}

/// Implements scalar division.
/// $$ A / s = \sum_{I} (a_I / s) e_I $$
impl<T> Div<T> for CausalMultiVector<T>
where
    T: Div<Output = T> + Copy + Clone,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let data = self.data.iter().map(|a| *a / rhs).collect();
        Self {
            data,
            metric: self.metric,
        }
    }
}

/// Implements the Geometric Product of two multivectors.
/// $$ AB = \sum_{I,J} a_I b_J (e_I e_J) $$
///
/// The product $e_I e_J$ is calculated using `calculate_basis_product`, handling sign changes from reordering and the metric signature.
impl<T> Mul for CausalMultiVector<T>
where
    T: Num + Copy + Clone + AddAssign + SubAssign,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        if self.metric != rhs.metric {
            panic!("Dimension mismatch in geometric product");
        }

        let dim = self.metric.dimension();
        let count = 1 << dim;
        let mut result_data = vec![T::zero(); count];

        for i in 0..count {
            if self.data[i].is_zero() {
                continue;
            }
            for j in 0..count {
                if rhs.data[j].is_zero() {
                    continue;
                }

                // Use the helper defined in the struct impl
                let (sign, result_idx) = Self::calculate_basis_product(i, j, &self.metric);

                if sign != 0 {
                    let val = self.data[i] * rhs.data[j];
                    if sign > 0 {
                        result_data[result_idx] += val;
                    } else {
                        result_data[result_idx] -= val;
                    }
                }
            }
        }

        Self {
            data: result_data,
            metric: self.metric,
        }
    }
}
