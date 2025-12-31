/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::MultiVector;
use crate::{CausalMultiVector, CausalMultiVectorError};
use deep_causality_num::Field;
use std::ops::{AddAssign, Neg, SubAssign};

impl<T> MultiVector<T> for CausalMultiVector<T> {
    fn grade_projection(&self, k: u32) -> Self
    where
        T: Field + Copy,
    {
        self.grade_projection_impl(k)
    }

    fn reversion(&self) -> Self
    where
        T: Field + Copy + Clone + Neg<Output = T>,
    {
        self.reversion_impl()
    }

    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    fn squared_magnitude(&self) -> T
    where
        T: Field
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        self.squared_magnitude_impl()
    }

    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    fn squared_magnitude(&self) -> T
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.squared_magnitude_impl()
    }

    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        self.inverse_impl()
    }

    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign,
    {
        self.inverse_impl()
    }

    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    fn dual(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        self.dual_impl()
    }

    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    fn dual(&self) -> Result<Self, CausalMultiVectorError>
    where
        T: Field
            + Copy
            + Clone
            + Neg<Output = T>
            + core::ops::Div<Output = T>
            + PartialEq
            + AddAssign
            + SubAssign,
    {
        self.dual_impl()
    }

    #[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
    fn geometric_product(&self, rhs: &Self) -> Self
    where
        T: Field
            + Copy
            + Clone
            + AddAssign
            + SubAssign
            + Neg<Output = T>
            + Default
            + PartialOrd
            + Send
            + Sync
            + 'static,
    {
        self.geometric_product_impl(rhs)
    }

    #[cfg(not(all(feature = "mlx", target_os = "macos", target_arch = "aarch64")))]
    fn geometric_product(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.geometric_product_impl(rhs)
    }

    fn outer_product(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign,
    {
        self.outer_product_impl(rhs)
    }

    fn inner_product(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign,
    {
        self.inner_product_impl(rhs)
    }

    fn commutator_lie(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.commutator_lie_impl(rhs)
    }

    fn commutator_geometric(&self, rhs: &Self) -> Self
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.commutator_geometric_impl(rhs)
    }

    fn basis_shift(&self, index: usize) -> Self
    where
        T: Clone,
    {
        self.basis_shift_impl(index)
    }
}
