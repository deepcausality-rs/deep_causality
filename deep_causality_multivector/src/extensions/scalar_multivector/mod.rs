/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalMultiVectorError, MultiVector};

// =========================================================
// MultiVector Implementation for f64 (Scalar)
// =========================================================

impl MultiVector<f64> for f64 {
    fn grade_projection(&self, k: u32) -> Self {
        if k == 0 { *self } else { 0.0 }
    }

    fn reversion(&self) -> Self {
        *self
    }

    fn squared_magnitude(&self) -> Self {
        self * self
    }

    fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        Self: Sized,
    {
        if *self == 0.0 {
            Err(CausalMultiVectorError::zero_magnitude())
        } else {
            Ok(1.0 / *self)
        }
    }

    fn dual(&self) -> Result<Self, CausalMultiVectorError>
    where
        Self: Sized,
    {
        // For a scalar, the dual depends on the algebra's pseudoscalar.
        // For simplicity and since it's not directly needed by cup_product,
        // we'll return the scalar itself or a placeholder for now.
        // A more robust implementation might require a pseudoscalar context.
        Ok(*self)
    }

    fn geometric_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn outer_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn inner_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn commutator_lie(&self, _rhs: &Self) -> Self {
        // Scalars commute
        0.0
    }

    fn commutator_geometric(&self, _rhs: &Self) -> Self {
        // Scalars commute
        0.0
    }

    fn basis_shift(&self, _index: usize) -> Self {
        *self
    }
}

// =========================================================
// MultiVector Implementation for f32 (Scalar)
// =========================================================

impl MultiVector<f32> for f32 {
    fn grade_projection(&self, k: u32) -> Self {
        if k == 0 { *self } else { 0.0 }
    }

    fn reversion(&self) -> Self {
        *self
    }

    fn squared_magnitude(&self) -> Self {
        self * self
    }

    fn inverse(&self) -> Result<Self, CausalMultiVectorError>
    where
        Self: Sized,
    {
        if *self == 0.0 {
            Err(CausalMultiVectorError::zero_magnitude())
        } else {
            Ok(1.0 / *self)
        }
    }

    fn dual(&self) -> Result<Self, CausalMultiVectorError>
    where
        Self: Sized,
    {
        Ok(*self)
    }

    fn geometric_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn outer_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn inner_product(&self, rhs: &Self) -> Self {
        self * rhs
    }

    fn commutator_lie(&self, _rhs: &Self) -> Self {
        0.0
    }

    fn commutator_geometric(&self, _rhs: &Self) -> Self {
        0.0
    }

    fn basis_shift(&self, _index: usize) -> Self {
        *self
    }
}
