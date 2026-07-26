/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The divergence-free velocity type-state.
//!
//! `SolenoidalField<R>` unifies the gap note's `ProjectedVelocityOneForm` and
//! `3DCausalFluidDynamics.md` B4's `SolenoidalField` into one type whose every
//! constructor *is* a projection:
//!
//! 1. [`SolenoidalField::from_leray_projection`] and its parameterized twin
//!    [`SolenoidalField::from_leray_projection_opts`] — the per-step solver
//!    path (`Manifold::leray_project`, one gauge-fixed CG solve).
//! 2. [`SolenoidalField::from_open_leray_projection_weighted_opts`] — the same
//!    solve with open-boundary and weighted cut-face rows.
//! 3. [`SolenoidalField::from_hodge_projection`] — the per-snapshot analysis
//!    path (`δβ + h` of a grade-1 Hodge decomposition).
//!
//! The carrier is a private field and the type implements **no arithmetic**:
//! the sum of two discretely projected fields is not projected, so `a + b`
//! does not compile and re-projection is the only way to combine fields. Read
//! access is [`SolenoidalField::as_one_form`], which hands out a shared
//! reference and has no mutable counterpart.
//!
//! What the type-state does **not** enforce.
//! [`SolenoidalField::constrain_edges`] and [`SolenoidalField::with_lift`] are
//! `pub`, take `self`, and re-wrap a modified tensor. They are not
//! crate-internal and cannot be: the DEC solver that drives them lives in the
//! sibling crate `deep_causality_cfd`
//! (`solvers/dec/dec_ns_solver/step.rs:121-123` and `seed.rs:82-84`), so
//! `pub(crate)` would not reach it. That solver calls them only on the output
//! of a constrained projection that already pinned exactly those edges, which
//! is why the writes are invariant-preserving there.
//!
//! Off that path the caller carries the invariant. Writing a coefficient the
//! preceding projection did not fix leaves `‖δu‖ ≈ 0` broken while the value
//! keeps the type `SolenoidalField`, and `DecNsSolver::step` will accept it.
//! Re-project before time-stepping such a field. Both methods also index the
//! carrier by raw edge index and panic on an out-of-range index.

use alloc::format;
use core::fmt::{Debug, Display};

use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
use deep_causality_tensor::CausalTensor;
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

    /// The **open-boundary** projection (`Manifold::leray_project_open_weighted_opts`) admitting a
    /// prescribed inflow with a pressure reference, optionally extended with the weighted cut-face
    /// no-slip rows of an immersed body so the body no-slip is enforced on the *state* (not only the
    /// per-stage rate). `zeroed` edges are pinned to zero (no-slip walls + body interior),
    /// `prescribed` edges are fixed at their field value with their flux counted, and `reference`
    /// vertices pin the outflow pressure. With empty `rows` it is bit-identical to the binary open
    /// path; with empty `prescribed`/`reference` it reduces to the constrained projection. `x0`
    /// warm-starts the projection's grade-0 CG.
    ///
    /// # Errors
    /// As [`Self::from_leray_projection`].
    #[allow(clippy::too_many_arguments)]
    pub fn from_open_leray_projection_weighted_opts<const D: usize>(
        velocity: &VelocityOneForm<R>,
        manifold: &Manifold<LatticeComplex<D, R>, R>,
        zeroed: &[usize],
        prescribed: &[usize],
        reference: &[usize],
        rows: &[deep_causality_topology::CutFaceConstraint<R>],
        opts: &deep_causality_topology::HodgeDecomposeOptions<R>,
        x0: Option<&[R]>,
    ) -> Result<(Self, CausalTensor<R>), PhysicsError> {
        let projection = manifold
            .leray_project_open_weighted_opts(
                velocity.as_tensor(),
                zeroed,
                prescribed,
                reference,
                rows,
                opts,
                x0,
            )
            .map_err(|e| {
                PhysicsError::TopologyError(format!("open weighted Leray projection failed: {e}"))
            })?;
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

    /// Wall-bounded path: zero the given (wall-tangential) edge coefficients,
    /// the homogeneous no-slip constraint `P_S` applied as the step's final
    /// operation (design D8). A bit-exact no-op when `edges` is empty (fully
    /// periodic), so the periodic construction path is preserved.
    ///
    /// # Visibility and invariant
    /// This is `pub`, not `pub(crate)`. It writes into the carrier and
    /// re-wraps it, so it bypasses the projection that establishes
    /// `‖δu‖ ≈ 0`; the type-state cannot enforce the invariant across it. The
    /// intended caller is the DEC solver's per-step no-slip chain in the
    /// sibling crate `deep_causality_cfd`
    /// (`solvers/dec/dec_ns_solver/step.rs:121-123`, `seed.rs:82-84`), which a
    /// `pub(crate)` seam would not reach. There the argument is exactly the
    /// `zeroed` set the immediately preceding constrained projection pinned,
    /// so this stage removes only the residual no-slip violation that solve
    /// left behind. The wall-bounded-ns spec sanctions that trade: no-slip
    /// exact, divergence at the solve's exactness.
    ///
    /// A caller off that path takes the invariant on itself. Zeroing edges the
    /// preceding projection did not pin breaks divergence-freedom by the
    /// amount those coefficients carried, and the result still types as
    /// `SolenoidalField`. Re-project before time-stepping it.
    ///
    /// # Panics
    /// Indexes the carrier directly: an edge index at or past
    /// [`Self::len`] panics.
    pub fn constrain_edges(self, edges: &[usize]) -> Self {
        if edges.is_empty() {
            return self;
        }
        let mut data = self.field.into_vec();
        for &e in edges {
            data[e] = R::zero();
        }
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
    }

    /// Wall-bounded path: set the prescribed tangential wall values (the
    /// moving-wall lift, edge index → edge integral). Applied after the
    /// constrained projection, whose output is exactly zero on every
    /// constrained edge, so this is assignment onto zeros: the lift edges
    /// carry their prescribed values exactly while the projection's free-edge
    /// values are untouched. A no-op when `lift` is empty.
    ///
    /// # Visibility and invariant
    /// This is `pub`, not `pub(crate)`. It writes caller-chosen values at
    /// caller-chosen indices and re-wraps the carrier, so it is the widest
    /// hole in the type-state: `field.with_lift(&[(0, 1e9)])` compiles,
    /// returns a `SolenoidalField` with large divergence, and
    /// `DecNsSolver::step` accepts it. The intended caller is the DEC solver's
    /// per-step no-slip chain in the sibling crate `deep_causality_cfd`
    /// (`solvers/dec/dec_ns_solver/step.rs:121-123`, `seed.rs:82-84`), which a
    /// `pub(crate)` seam would not reach. There the lift edges are constrained
    /// edges the preceding projection zeroed, and the projector ignores
    /// constrained-edge input values (`P(u) = P(u − lift)`), so re-applying
    /// the lift restores the boundary condition without disturbing the solve.
    ///
    /// A caller off that path takes the invariant on itself: re-project before
    /// time-stepping any field this method has written to.
    ///
    /// # Panics
    /// Indexes the carrier directly: an edge index at or past
    /// [`Self::len`] panics.
    pub fn with_lift(self, lift: &[(usize, R)]) -> Self {
        if lift.is_empty() {
            return self;
        }
        let mut data = self.field.into_vec();
        for &(e, value) in lift {
            data[e] = value;
        }
        let len = data.len();
        Self {
            field: CausalTensor::new(data, alloc::vec![len])
                .expect("1-D tensor allocation cannot fail"),
        }
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
