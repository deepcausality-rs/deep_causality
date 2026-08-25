/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! API module for CausalMultiVector.

use crate::{CausalMultiVector, CausalMultiVectorError};
use crate::{MultiVector, MultiVectorL2Norm};
use core::iter::Sum;
use core::ops::{AddAssign, Neg, SubAssign};
use deep_causality_algebra::{DivisibleByIntegers, Field, Normed, NormedScalar};
use deep_causality_linear::vector_norm_l2;
use deep_causality_num::{One, Zero};

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

    fn squared_magnitude(&self) -> T
    where
        T: Field + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
    {
        self.squared_magnitude_impl()
    }

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
        T: DivisibleByIntegers + Copy + Clone + AddAssign + SubAssign + Neg<Output = T>,
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

/// # Routed through `deep_causality_linear`
///
/// The body was `data.iter().map(modulus_squared).fold(...).sqrt()` — correct, and a second
/// definition of the Euclidean norm. It now calls
/// [`vector_norm_l2`](deep_causality_linear::vector_norm_l2), which is the same expression with the
/// same bound, defined once.
///
/// The slice form rather than a `DenseVector`: the coefficients are already a `Vec<T>`, and
/// wrapping them in a vector type would allocate and copy every coefficient on every norm.
///
/// The bound moves from `ScalarEval` to [`NormedScalar`]. `ScalarEval` is this crate's facade over
/// `Normed` and reaches every scalar through a blanket, but a `T: ScalarEval` bound does not let
/// the compiler conclude `T: Normed`, which is what the shared norm asks for. `NormedScalar` adds
/// `FromPrimitive` on top of `Normed`; every scalar that satisfied the old bound satisfies this
/// one, so nothing that compiled stops compiling.
impl<T> MultiVectorL2Norm<T> for CausalMultiVector<T>
where
    T: Field + Copy + Sum + NormedScalar,
    <T as Normed>::Real: Sum,
{
    // The output of a Norm is always Real (e.g. f64), even if T is Complex.
    type Output = <T as Normed>::Real;

    fn norm_l2(&self) -> Self::Output {
        vector_norm_l2(&self.data)
    }

    fn normalize_l2(&self) -> Self {
        let norm = self.norm_l2();

        if norm == <T as Normed>::Real::zero() {
            return self.clone();
        }

        // We scale by 1.0 / norm
        let scale_factor = <T as Normed>::Real::one() / norm;

        let new_data = self
            .data
            .iter()
            .map(|x| Normed::scale_by_real(x, scale_factor)) // Works for f64 AND Complex
            .collect();

        Self {
            data: new_data,
            metric: self.metric,
        }
    }
}
