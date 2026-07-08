/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Body force (e.g. gravity) as an edge 1-form (grade-1 cochain).

use deep_causality_algebra::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};

use super::validate_graded_field;
use crate::error::physics_error::PhysicsError;

/// A body-force-per-unit-mass field as an edge 1-form on a cubical lattice.
/// A forcing input, not a marching state: no arithmetic is provided.
#[derive(Debug, Clone, PartialEq)]
pub struct BodyForceOneForm<R: RealField> {
    field: CausalTensor<R>,
}

impl<R: RealField> BodyForceOneForm<R> {
    /// Construct from a grade-1 cochain, validating length and finiteness.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` on a length/grade mismatch.
    /// * `PhysicsError::NumericalInstability` on NaN or infinite coefficients.
    pub fn new<const D: usize>(
        field: CausalTensor<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<Self, PhysicsError> {
        validate_graded_field(&field, 1, "BodyForceOneForm", manifold)?;
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
