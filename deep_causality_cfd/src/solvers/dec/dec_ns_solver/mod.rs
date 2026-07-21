/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The periodic DEC-native incompressible Navier–Stokes solver: owns the
//! physics configuration, borrows the manifold, and exposes the projected
//! step, the run loops, initial-condition seeding, and the opt-in
//! pressure diagnostic. See the parent module doc for the formulation.

pub(in crate::solvers::dec) mod no_slip;
mod pressure;
mod run;
mod seed;
mod step;

use alloc::format;
use deep_causality_topology::{ChainComplex, HodgeDecomposeOptions, LatticeComplex, Manifold};

use crate::solvers::dec::DecNsScalar;
use crate::solvers::dec::dec_ns_rate::DecNsRate;
use deep_causality_physics::BodyForceOneForm;
use deep_causality_physics::PhysicsError;

/// The DEC Navier–Stokes solver on a periodic lattice manifold.
///
/// Constructed once per (manifold, ν, dt, forcing) configuration; the
/// constructor validates every per-step operator precondition through
/// [`DecNsRate::new`], caches the minimum edge length for the CFL guard,
/// and rejects a non-finite or non-positive `dt`. Safety factors and CG
/// options are adjusted through the builder methods.
#[derive(Debug)]
pub struct DecNsSolver<'m, const D: usize, R: DecNsScalar> {
    pub(super) rate: DecNsRate<'m, D, R>,
    pub(super) manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    pub(super) dt: R,
    pub(super) cg_options: HodgeDecomposeOptions<R>,
    pub(super) cfl_advective: R,
    pub(super) cfl_diffusive: R,
    pub(super) dx_min: R,
    /// Prescribed tangential wall values (edge index → edge integral): the
    /// inhomogeneous no-slip lift of a moving wall (Couette, the cavity
    /// lid). Empty by default; populated by [`Self::with_moving_wall`].
    /// Re-applied after every projection — the constrained projector
    /// ignores constrained-edge input values, so `P(u) = P(u − lift)`
    /// exactly and no subtraction is needed.
    pub(super) lift: Vec<(usize, R)>,
}

impl<'m, const D: usize, R: DecNsScalar> DecNsSolver<'m, D, R> {
    /// Builds a solver for time step `dt` and viscosity `nu`, with an
    /// optional body force.
    ///
    /// # Errors
    /// * Every rejection of [`DecNsRate::new`] (dimension, metric, ν,
    ///   body-force length).
    /// * `PhysicsError::PhysicalInvariantBroken` when `dt` is not finite
    ///   or not strictly positive.
    pub fn new(
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
        nu: R,
        dt: R,
        body_force: Option<&BodyForceOneForm<R>>,
    ) -> Result<Self, PhysicsError> {
        let rate = DecNsRate::new(manifold, nu, body_force)?;

        if !dt.is_finite() || dt <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "DecNsSolver: dt must be finite and positive, got {dt}"
            )));
        }

        // Minimum edge length for the CFL guard, from the Regge geometry.
        let complex = manifold.complex();
        let metric = manifold
            .metric()
            // Coverage exemption: metric presence was validated by DecNsRate::new.
            .expect("metric presence validated by DecNsRate::new");
        let mut dx_min: Option<R> = None;
        for cell in complex.iter_cells(1) {
            let len = metric.cell_volume(complex, &cell);
            dx_min = Some(match dx_min {
                Some(m) if m <= len => m,
                _ => len,
            });
        }
        let dx_min = dx_min.ok_or_else(|| {
            PhysicsError::DimensionMismatch(
                "DecNsSolver: the lattice has no edges (empty shape)".into(),
            )
        })?;

        let default_safety = R::from_f64(0.9)
            // Coverage exemption: 0.9 lifts into every real field.
            .expect("0.9 lifts into R");

        Ok(Self {
            rate,
            manifold,
            dt,
            cg_options: HodgeDecomposeOptions::default(),
            cfl_advective: default_safety,
            cfl_diffusive: default_safety,
            dx_min,
            lift: alloc::vec::Vec::new(),
        })
    }

    /// Builds a solver from a **composable boundary-zone set** (CFD Stage-4
    /// `add-boundary-zone-abstraction`) — the canonical surface for the explicit boundary
    /// actuators. The static zone composition is folded into the solver at construction: every
    /// zone's [`crate::solvers::dec::boundary::BoundaryZone::collect_rate_source`] forms the body
    /// force, and every zone's [`crate::solvers::dec::boundary::BoundaryZone::collect_lift`] (at
    /// step 0) forms the prescribed lift. Structural boundaries
    /// — wall no-slip and immersed cut bodies — are still derived automatically from the lattice
    /// and metric. A `()` zone set is the plain closed-domain solver.
    ///
    /// This is equivalent to (and bit-identical with) building the solver via [`Self::new`] with
    /// the composed body force and applying the composed moving-wall lift.
    ///
    /// # Errors
    /// As [`Self::new`], plus any failure validating the composed body force.
    pub fn with_zones<Z>(
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
        nu: R,
        dt: R,
        zones: Z,
    ) -> Result<Self, PhysicsError>
    where
        Z: crate::solvers::dec::boundary::BoundaryZone<D, R>,
    {
        let n1 = manifold.complex().num_cells(1);

        // Fold the rate source (body force) over the zone set.
        let mut source = alloc::vec![R::zero(); n1];
        zones.collect_rate_source(manifold, &mut source);
        let body_force = if source.iter().any(|v| *v != R::zero()) {
            let tensor = deep_causality_tensor::CausalTensor::new(source, alloc::vec![n1])
                .expect("1-D tensor allocation cannot fail");
            Some(BodyForceOneForm::new(tensor, manifold)?)
        } else {
            None
        };

        let mut solver = Self::new(manifold, nu, dt, body_force.as_ref())?;

        // Fold the prescribed lift (static zones evaluate at step 0).
        let mut lift = alloc::vec::Vec::new();
        zones.collect_lift(manifold, 0, &mut lift);
        solver.lift = lift;

        // Fold the free-slip un-pin first (it shrinks the no-slip set), then the open-boundary
        // sets (inflow prescribed edges, outflow reference vertices) recompute the rate constraint.
        let mut slip = alloc::vec::Vec::new();
        zones.collect_slip_edges(manifold, &mut slip);
        solver.rate.apply_slip(&slip);

        // Fold the zone-supplied constrained edges. After the slip un-pin, so an explicitly
        // supplied constraint is not removed by a free-slip zone (union, per `recompute_rate_constrained`).
        let mut constrained = alloc::vec::Vec::new();
        zones.collect_constrained_edges(manifold, &mut constrained);
        if !constrained.is_empty() {
            solver.rate.set_zone_constrained(constrained);
        }

        let mut prescribed = alloc::vec::Vec::new();
        zones.collect_prescribed_edges(manifold, &mut prescribed);
        let mut reference = alloc::vec::Vec::new();
        zones.collect_reference_vertices(manifold, &mut reference);
        if !prescribed.is_empty() || !reference.is_empty() {
            solver.rate.set_open_boundary(prescribed, reference);
        }

        Ok(solver)
    }

    /// Prescribes a moving wall: the wall perpendicular to `wall_axis` (the
    /// `max_side` face when true, the zero face otherwise) carries the
    /// tangential `velocity` — the inhomogeneous no-slip condition of
    /// Couette flow and the lid-driven cavity. The lift values are edge
    /// integrals (`velocity[a] · edge length`) on that wall's tangential
    /// edges; they are held exactly at every step boundary while the
    /// remaining wall edges stay pinned to zero.
    ///
    /// # Errors
    /// * `PhysicsError::DimensionMismatch` when `wall_axis ≥ D`.
    /// * `PhysicsError::PhysicalInvariantBroken` when `wall_axis` is
    ///   periodic (no wall to move), when the velocity has a non-zero
    ///   wall-normal component, or when it is not finite.
    pub fn with_moving_wall(
        mut self,
        wall_axis: usize,
        max_side: bool,
        velocity: [R; D],
    ) -> Result<Self, PhysicsError> {
        if wall_axis >= D {
            return Err(PhysicsError::DimensionMismatch(format!(
                "with_moving_wall: wall axis {wall_axis} out of range for D = {D}"
            )));
        }
        let complex = self.manifold.complex();
        if complex.periodic()[wall_axis] {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "with_moving_wall: axis {wall_axis} is periodic — there is no wall to move"
            )));
        }
        if velocity.iter().any(|v| !v.is_finite()) {
            return Err(PhysicsError::NumericalInstability(
                "with_moving_wall: velocity must be finite".into(),
            ));
        }
        if velocity[wall_axis] != R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "with_moving_wall: the wall-normal velocity component (axis {wall_axis}) \
                 must be zero — wall-normal flux is the projection's Neumann condition"
            )));
        }

        let metric = self
            .manifold
            .metric()
            // Coverage exemption: metric presence validated by DecNsRate::new.
            .expect("metric presence validated by DecNsRate::new");
        let shape = complex.shape();
        let wall_pos = if max_side { shape[wall_axis] - 1 } else { 0 };
        let mut lift = alloc::vec::Vec::new();
        for (idx, cell) in complex.iter_cells(1).enumerate() {
            let axis = cell.orientation().trailing_zeros() as usize;
            if axis == wall_axis
                || velocity[axis] == R::zero()
                || cell.position()[wall_axis] != wall_pos
            {
                continue;
            }
            let length = metric.cell_volume(complex, &cell);
            lift.push((idx, velocity[axis] * length));
        }
        self.lift = lift;
        Ok(self)
    }

    /// Replaces the projection CG options (tolerance, iteration budget).
    pub fn with_cg_options(mut self, opts: HodgeDecomposeOptions<R>) -> Self {
        self.cg_options = opts;
        self
    }

    /// Enable projection **warm start**: each per-stage Leray solve seeds its CG with the previous
    /// solve's potential, which converges in far fewer iterations as the flow develops and the
    /// per-step right-hand side stops changing. Off by default; the marched result is identical to
    /// the cold path within the CG tolerance.
    pub fn with_warm_start(mut self) -> Self {
        self.rate.set_warm_start(true);
        self
    }

    /// Use the **staircase** immersed no-slip instead of the (default) aperture-resolved cut-face
    /// rows — the validation-comparison / fallback path. The body geometry (cut volumes, apertures,
    /// cut star) is unchanged; only the no-slip mechanism flips, so a side-by-side run isolates the
    /// effect of the aperture-resolved wall on shedding / Strouhal / drag. A no-op without an
    /// immersed body or `Cut` cells.
    pub fn with_staircase_noslip(mut self) -> Self {
        self.rate.set_staircase_noslip();
        self
    }

    /// Opt into the spectral evaluation of the viscous term (fully
    /// periodic uniform lattices only; the `spectral-diffusion`
    /// capability). Off by default — the validation ladder gates any
    /// future default-on.
    ///
    /// # Errors
    /// `PhysicsError::TopologyError` when the lattice is not fully
    /// periodic or the metric carries no per-axis Euclidean spacings.
    pub fn with_spectral_diffusion(mut self) -> Result<Self, PhysicsError> {
        self.rate = self.rate.with_spectral_diffusion()?;
        Ok(self)
    }

    /// Replaces the CFL safety factors (advective, diffusive).
    ///
    /// # Errors
    /// `PhysicsError::PhysicalInvariantBroken` when either factor is not
    /// finite or not strictly positive.
    pub fn with_cfl_factors(mut self, advective: R, diffusive: R) -> Result<Self, PhysicsError> {
        if !advective.is_finite()
            || advective <= R::zero()
            || !diffusive.is_finite()
            || diffusive <= R::zero()
        {
            return Err(PhysicsError::PhysicalInvariantBroken(format!(
                "DecNsSolver: CFL safety factors must be finite and positive, \
                 got advective {advective}, diffusive {diffusive}"
            )));
        }
        self.cfl_advective = advective;
        self.cfl_diffusive = diffusive;
        Ok(self)
    }

    /// The configured time step.
    pub fn dt(&self) -> R {
        self.dt
    }

    /// The configured kinematic viscosity.
    pub fn nu(&self) -> R {
        self.rate.nu()
    }

    /// The minimum lattice edge length the CFL guard divides by.
    pub fn dx_min(&self) -> R {
        self.dx_min
    }

    /// The rate field (for direct RHS evaluation, e.g. cross-validation).
    pub fn rate(&self) -> &DecNsRate<'m, D, R> {
        &self.rate
    }
}
