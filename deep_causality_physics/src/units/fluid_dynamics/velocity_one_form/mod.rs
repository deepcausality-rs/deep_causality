/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Velocity as an edge 1-form: the DEC solver's marching state.

mod velocity_one_form_ops;

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::units::fluid_dynamics::validate_graded_field;

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
