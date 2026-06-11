/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Vorticity as a face 2-form (grade-2 cochain), `ω = d u♭`.

use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::units::fluid_dynamics::validate_graded_field;

/// A vorticity field as a face 2-form on a cubical lattice. Closedness
/// (`dω = 0`) is automatic for any `ω = d u♭` by `d² = 0` and therefore not a
/// runtime invariant of this carrier (design open question 3 of
/// `add-dec-solver-foundations`: plain typed wrapper, no type-state).
#[derive(Debug, Clone, PartialEq)]
pub struct VorticityTwoForm<R: RealField> {
    field: CausalTensor<R>,
}

impl<R: RealField> VorticityTwoForm<R> {
    /// Construct from a grade-2 cochain, validating length and finiteness.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` on a length/grade mismatch.
    /// * `PhysicsError::NumericalInstability` on NaN or infinite coefficients.
    pub fn new<const D: usize>(
        field: CausalTensor<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<Self, PhysicsError> {
        validate_graded_field(&field, 2, "VorticityTwoForm", manifold)?;
        Ok(Self { field })
    }

    /// The underlying face cochain.
    pub fn as_tensor(&self) -> &CausalTensor<R> {
        &self.field
    }

    /// Number of face coefficients.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// True when the carrier holds no coefficients.
    pub fn is_empty(&self) -> bool {
        self.field.len() == 0
    }
}
