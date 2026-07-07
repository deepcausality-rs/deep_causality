/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Typed differential-form carriers for the DEC fluid solver
//! (`add-dec-solver-foundations`, capability `typed-fluid-forms`).
//!
//! Each type wraps a cochain (`CausalTensor<R>`) of a fixed grade and
//! validates, at construction, the grade/length match against the manifold and
//! the finiteness of every coefficient — physical invariants in the crate's
//! units-as-invariants tradition: impossible states are unconstructible.
//!
//! * [`PressureZeroForm`] — grade 0 (vertices).
//! * [`VelocityOneForm`] / [`BodyForceOneForm`] — grade 1 (edges). Only the
//!   velocity carrier implements `Add`/`Mul<R>`: it is the `Rk4` march state.
//! * [`VorticityTwoForm`] — grade 2 (faces).
//! * [`SolenoidalField`] — the divergence-free type-state; constructible only
//!   through a projection (Leray per-step, or from a Hodge decomposition
//!   per-snapshot). Deliberately has **no** arithmetic: the sum of two
//!   projected fields is not discretely projected, so re-projection is the
//!   only path back into the type.

pub mod body_force_one_form;
pub mod pressure_zero_form;
pub mod solenoidal_field;
pub mod velocity_one_form;
pub mod vorticity_two_form;

use alloc::format;

use deep_causality_algebra::RealField;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;

/// Shared constructor validation for graded-form carriers: length must match
/// `num_cells(grade)` on the manifold's complex, and every coefficient must be
/// finite.
pub(crate) fn validate_graded_field<const D: usize, R>(
    field: &CausalTensor<R>,
    grade: usize,
    type_name: &str,
    manifold: &Manifold<LatticeComplex<D, R>, R>,
) -> Result<(), PhysicsError>
where
    R: RealField,
{
    let expected = manifold.complex().num_cells(grade);
    if field.len() != expected {
        return Err(PhysicsError::DimensionMismatch(format!(
            "{type_name}: expected {expected} grade-{grade} coefficients, got {}",
            field.len()
        )));
    }
    if let Some(idx) = field.as_slice().iter().position(|v| !v.is_finite()) {
        return Err(PhysicsError::NumericalInstability(format!(
            "{type_name}: non-finite coefficient at index {idx}"
        )));
    }
    Ok(())
}
