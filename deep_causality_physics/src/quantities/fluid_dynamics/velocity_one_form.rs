/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Velocity as an edge 1-form: the DEC solver's marching state.

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};
use std::ops::{Add, Mul};

use super::validate_graded_field;
use crate::error::physics_error::PhysicsError;

/// A velocity field as an edge 1-form (grade-1 cochain) on a cubical lattice.
///
/// This is the **unprojected** marching state: it implements `Clone`, `Add`,
/// and `Mul<R>` (and nothing more) so a whole-field state satisfies the
/// `Rk4`/`Euler` arrow bounds. Divergence-freeness is *not* an invariant of
/// this type — that is [`crate::SolenoidalField`]'s job, reachable only
/// through a projection.
#[derive(Debug, Clone, PartialEq)]
pub struct VelocityOneForm<R: RealField> {
    field: CausalTensor<R>,
}

impl<R: RealField> VelocityOneForm<R> {
    /// Construct from a grade-1 cochain, validating length against the
    /// manifold and finiteness of every coefficient.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` on a length/grade mismatch.
    /// * `PhysicsError::NumericalInstability` on NaN or infinite coefficients.
    pub fn new<const D: usize>(
        field: CausalTensor<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<Self, PhysicsError> {
        validate_graded_field(&field, 1, "VelocityOneForm", manifold)?;
        Ok(Self { field })
    }

    /// The underlying edge cochain.
    pub fn as_tensor(&self) -> &CausalTensor<R> {
        &self.field
    }

    /// Number of edge coefficients.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// True when the carrier holds no coefficients.
    pub fn is_empty(&self) -> bool {
        self.field.len() == 0
    }
}

/// `Add` and `Mul<R>` for [`VelocityOneForm`] — exactly the bounds the
/// `Rk4`/`Euler` arrows require of a marching state, and nothing more.
impl<R: RealField> Add for VelocityOneForm<R> {
    type Output = Self;

    /// Element-wise sum of two velocity 1-forms.
    ///
    /// # Panics
    /// Panics when the operands carry different edge counts (fields from
    /// different lattices); validated construction makes this a programming
    /// error, not a runtime condition.
    fn add(self, rhs: Self) -> Self {
        assert_eq!(
            self.field.len(),
            rhs.field.len(),
            "VelocityOneForm + VelocityOneForm requires matching edge counts"
        );
        let data: alloc::vec::Vec<R> = self
            .field
            .as_slice()
            .iter()
            .zip(rhs.field.as_slice().iter())
            .map(|(a, b)| *a + *b)
            .collect();
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
    }
}

impl<R: RealField> Mul<R> for VelocityOneForm<R> {
    type Output = Self;

    /// Scalar scaling of the whole field.
    fn mul(self, rhs: R) -> Self {
        let data: alloc::vec::Vec<R> = self.field.as_slice().iter().map(|a| *a * rhs).collect();
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
    }
}
