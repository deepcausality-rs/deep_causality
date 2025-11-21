/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalMultiVector, CausalMultiVectorError, Metric, MultiVector};
use deep_causality_num::Num;
use std::ops::{AddAssign, Div, Neg, SubAssign};

impl<T> MultiVector<T> for CausalMultiVector<T> {
    fn grade_projection(&self, k: u32) -> Self
    where
        T: Num + Copy + Clone,
    {
        self.grade_projection_impl(k)
    }

    fn reversion(&self) -> Self
    where
        T: Num + Copy + Clone + Neg<Output = T>,
    {
        self.reversion_impl()
    }

    fn squared_magnitude(&self) -> T
    where
        T: Num + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.squared_magnitude_impl()
    }

    fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Num
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Div<Output = T>
            + PartialEq,
        Self: Sized,
    {
        self.inverse_impl()
    }

    fn dual(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Num
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Div<Output = T>
            + PartialEq,
        Self: Sized,
    {
        self.dual_impl()
    }

    fn basis_shift(&self, index: usize) -> Self
    where
        T: Clone,
    {
        self.basis_shift_impl(index)
    }

    fn outer_product(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
        self.outer_product_impl(rhs)
    }

    fn inner_product(&self, rhs: &Self) -> Self
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
        self.inner_product_impl(rhs)
    }

    fn basis_product(a_map: usize, b_map: usize, metric: &Metric) -> (i32, usize)
    where
        T: Num + Copy + Clone + AddAssign + SubAssign,
    {
        Self::basis_product_impl(a_map, b_map, metric)
    }
}
