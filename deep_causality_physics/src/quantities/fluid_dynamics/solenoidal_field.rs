/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The divergence-free velocity type-state.
//!
//! `SolenoidalField<R>` unifies the gap note's `ProjectedVelocityOneForm` and
//! `3DCausalFluidDynamics.md` B4's `SolenoidalField` into one type with
//! exactly two construction paths, both of which *are* projections:
//!
//! 1. [`SolenoidalField::from_leray_projection`] — the per-step solver path
//!    (`Manifold::leray_project`, one gauge-fixed CG solve).
//! 2. [`SolenoidalField::from_hodge_projection`] — the per-snapshot analysis
//!    path (`δβ + h` of a grade-1 Hodge decomposition).
//!
//! "You cannot time-step an unprojected field" is thereby a compile-time
//! fact: the type has no other constructor, and **no arithmetic** — the sum
//! of two discretely projected fields is not projected, so re-projection is
//! the only path back into the type. Read access is provided by
//! [`SolenoidalField::as_one_form`]; there is no path that re-wraps a
//! modified tensor.

use alloc::format;
use core::fmt::{Debug, Display};

use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MaybeParallel;
use deep_causality_topology::{HodgeDecomposition, LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::velocity_one_form::VelocityOneForm;

/// A divergence-free velocity 1-form: constructible only by projection.
///
/// External construction does not compile:
///
/// ```compile_fail
/// use deep_causality_physics::SolenoidalField;
/// use deep_causality_tensor::CausalTensor;
/// let t = CausalTensor::new(vec![0.0_f64; 4], vec![4]).unwrap();
/// let s = SolenoidalField { field: t }; // private field: no public constructor
/// ```
///
/// Arithmetic on the projected type does not compile (re-projection is the
/// only path back):
///
/// ```compile_fail
/// use deep_causality_physics::SolenoidalField;
/// fn add(a: SolenoidalField<f64>, b: SolenoidalField<f64>) -> SolenoidalField<f64> {
///     a + b // `Add` is deliberately unimplemented
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SolenoidalField<R: RealField> {
    field: CausalTensor<R>,
}

impl<R> SolenoidalField<R>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug + Display,
{
    /// Per-step solver path: Leray-project an (unprojected) velocity 1-form.
    /// Returns the projected field together with the grade-0 potential (the
    /// pressure-recovery diagnostic input).
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` wrapping the projection failure
    /// (dimension mismatch, missing metric, or CG non-convergence).
    pub fn from_leray_projection<const D: usize>(
        velocity: &VelocityOneForm<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
    ) -> Result<(Self, CausalTensor<R>), PhysicsError> {
        Self::from_leray_projection_opts(
            velocity,
            manifold,
            &deep_causality_topology::HodgeDecomposeOptions::default(),
        )
    }

    /// [`Self::from_leray_projection`] with a caller-supplied CG tolerance
    /// and iteration budget — the same construction path, parameterized.
    ///
    /// # Errors
    /// As [`Self::from_leray_projection`].
    pub fn from_leray_projection_opts<const D: usize>(
        velocity: &VelocityOneForm<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        opts: &deep_causality_topology::HodgeDecomposeOptions<R>,
    ) -> Result<(Self, CausalTensor<R>), PhysicsError> {
        let projection = manifold
            .leray_project_opts(velocity.as_tensor(), opts)
            .map_err(|e| PhysicsError::TopologyError(format!("Leray projection failed: {e}")))?;
        let (projected, potential) = projection.into_parts();
        Ok((Self { field: projected }, potential))
    }

    /// Per-snapshot analysis path: the divergence-free part (`δβ + h`) of a
    /// grade-1 Hodge decomposition.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` when the decomposition is not of
    ///   grade 1 or its components disagree in length.
    /// * `PhysicsError::NumericalInstability` on non-finite coefficients.
    pub fn from_hodge_projection(
        decomposition: &HodgeDecomposition<R>,
    ) -> Result<Self, PhysicsError> {
        if decomposition.grade() != 1 {
            return Err(PhysicsError::DimensionMismatch(format!(
                "SolenoidalField requires a grade-1 decomposition, got grade {}",
                decomposition.grade()
            )));
        }
        let co_exact = decomposition.co_exact().as_slice();
        let harmonic = decomposition.harmonic().as_slice();
        if co_exact.len() != harmonic.len() {
            return Err(PhysicsError::DimensionMismatch(format!(
                "SolenoidalField: component length mismatch ({} vs {})",
                co_exact.len(),
                harmonic.len()
            )));
        }
        let data: alloc::vec::Vec<R> = co_exact
            .iter()
            .zip(harmonic.iter())
            .map(|(b, h)| *b + *h)
            .collect();
        if let Some(idx) = data.iter().position(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(format!(
                "SolenoidalField: non-finite coefficient at index {idx}"
            )));
        }
        let len = data.len();
        let field =
            CausalTensor::new(data, alloc::vec![len]).expect("1-D tensor allocation cannot fail");
        Ok(Self { field })
    }

    /// Read-only access to the underlying divergence-free edge cochain.
    /// There is no mutable or re-wrapping counterpart by design.
    pub fn as_one_form(&self) -> &CausalTensor<R> {
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
