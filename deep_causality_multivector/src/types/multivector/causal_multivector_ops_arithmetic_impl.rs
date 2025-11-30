/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, MultiVector};
use deep_causality_num::{Num, RealField, Zero};
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

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

impl<T> Mul for CausalMultiVector<T>
where
    T: RealField + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
{
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.geometric_product(&rhs)
    }
}

impl<'b, T> Mul<&'b CausalMultiVector<T>> for &CausalMultiVector<T>
where
    T: RealField + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
{
    type Output = CausalMultiVector<T>;

    fn mul(self, rhs: &'b CausalMultiVector<T>) -> Self::Output {
        self.geometric_product(rhs)
    }
}

// Support: Reference addition ( &A + &B )
// This allows x.as_inner() + z.as_inner() to work without manual cloning everywhere
impl<'b, T> Add<&'b CausalMultiVector<T>> for &CausalMultiVector<T>
where
    T: Copy + Add<Output = T> + Zero,
{
    type Output = CausalMultiVector<T>;

    fn add(self, rhs: &'b CausalMultiVector<T>) -> Self::Output {
        if self.metric != rhs.metric {
            panic!("Metric mismatch in addition");
        }
        // Zip and add
        let new_data = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        CausalMultiVector {
            data: new_data,
            metric: self.metric,
        }
    }
}
