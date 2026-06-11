/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pressure as a vertex 0-form (grade-0 cochain).

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use super::validate_graded_field;

/// A pressure field as a vertex 0-form on a cubical lattice. Diagnostic
/// carrier (the Leray-form solver removes pressure from the time loop); no
/// arithmetic is provided.
#[derive(Debug, Clone, PartialEq)]
pub struct PressureZeroForm<R: RealField> {
    field: CausalTensor<R>,
}

impl<R: RealField> PressureZeroForm<R> {
    /// Construct from a grade-0 cochain, validating length and finiteness.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` on a length/grade mismatch.
    /// * `PhysicsError::NumericalInstability` on NaN or infinite coefficients.
    pub fn new<const D: usize>(
        field: CausalTensor<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<Self, PhysicsError> {
        validate_graded_field(&field, 0, "PressureZeroForm", manifold)?;
        Ok(Self { field })
    }

    /// The underlying vertex cochain.
    pub fn as_tensor(&self) -> &CausalTensor<R> {
        &self.field
    }

    /// Number of vertex coefficients.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// True when the carrier holds no coefficients.
    pub fn is_empty(&self) -> bool {
        self.field.len() == 0
    }
}
