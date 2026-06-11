/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The periodic DEC-native incompressible Navier–Stokes solver: owns the
//! physics configuration, borrows the manifold, and exposes the projected
//! step, the run loops, initial-condition seeding, and the opt-in
//! pressure diagnostic. See the parent module doc for the formulation.

mod pressure;
mod run;
mod seed;
mod step;

use alloc::format;
use deep_causality_num::RealField;
use deep_causality_topology::{HodgeDecomposeOptions, LatticeComplex, Manifold};

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::body_force_one_form::BodyForceOneForm;
use crate::theories::fluid_dynamics::dec::DecNsScalar;
use crate::theories::fluid_dynamics::dec::dec_ns_rate::DecNsRate;

/// The DEC Navier–Stokes solver on a periodic lattice manifold.
///
/// Constructed once per (manifold, ν, dt, forcing) configuration; the
/// constructor validates every per-step operator precondition through
/// [`DecNsRate::new`], caches the minimum edge length for the CFL guard,
/// and rejects a non-finite or non-positive `dt`. Safety factors and CG
/// options are adjusted through the builder methods.
#[derive(Debug)]
pub struct DecNsSolver<'m, const D: usize, R: RealField> {
    pub(super) rate: DecNsRate<'m, D, R>,
    pub(super) manifold: &'m Manifold<LatticeComplex<D, R>, R>,
    pub(super) dt: R,
    pub(super) cg_options: HodgeDecomposeOptions<R>,
    pub(super) cfl_advective: R,
    pub(super) cfl_diffusive: R,
    pub(super) dx_min: R,
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
        })
    }

    /// Replaces the projection CG options (tolerance, iteration budget).
    pub fn with_cg_options(mut self, opts: HodgeDecomposeOptions<R>) -> Self {
        self.cg_options = opts;
        self
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
