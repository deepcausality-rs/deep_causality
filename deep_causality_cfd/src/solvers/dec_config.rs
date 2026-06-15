/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Owned configuration and type-state builder for the DEC incompressible
//! Navier–Stokes solver.
//!
//! [`DecNsConfig`] is a fully-owned description that holds **no** borrow of the
//! manifold (design D2): it carries the physical and numerical knobs, and the
//! manifold-bound marcher ([`DecNsSolver`]) is materialized from it at run time via
//! [`DecNsConfig::materialize_with_zones`]. The same configuration can therefore be
//! reused across runs (factual + counterfactual).
//!
//! The builder is type-state (modeled on the Discovery `CdlBuilder`): the required
//! viscosity and time step must be supplied, in order, before the optional knobs and
//! the terminal `build`. Reynolds-number / CFL conveniences that need reference
//! scales live in the case-solver layer, which computes `ν` and `dt` and populates
//! this config.

use crate::solvers::dec::DecNsSolver;
use crate::solvers::dec::boundary::BoundaryZone;
use crate::types::CfdScalar;
use deep_causality_physics::PhysicsError;
use deep_causality_topology::{HodgeDecomposeOptions, LatticeComplex, Manifold};

/// Entry point for the DEC incompressible Navier–Stokes solver configuration:
/// `DecNs::config().viscosity(nu).time_step(dt)…build()`.
pub struct DecNs;

impl DecNs {
    /// Start a DEC NS configuration. The viscosity and time step are required next.
    pub fn config() -> DecNsConfigNeedsViscosity {
        DecNsConfigNeedsViscosity { _seal: () }
    }
}

/// Type-state: awaiting the kinematic viscosity.
pub struct DecNsConfigNeedsViscosity {
    _seal: (),
}

impl DecNsConfigNeedsViscosity {
    /// Set the kinematic viscosity `ν` (m²/s). Validated at `build`.
    pub fn viscosity<R: CfdScalar>(self, nu: R) -> DecNsConfigNeedsTimeStep<R> {
        DecNsConfigNeedsTimeStep { nu }
    }
}

/// Type-state: viscosity set, awaiting the time step.
pub struct DecNsConfigNeedsTimeStep<R: CfdScalar> {
    nu: R,
}

impl<R: CfdScalar> DecNsConfigNeedsTimeStep<R> {
    /// Set the marching time step `dt` (s). Validated at `build`.
    pub fn time_step(self, dt: R) -> DecNsConfigReady<R> {
        DecNsConfigReady::new(self.nu, dt)
    }
}

/// Type-state: required knobs set; optional knobs may be tuned before `build`.
pub struct DecNsConfigReady<R: CfdScalar> {
    nu: R,
    dt: R,
    cg_options: HodgeDecomposeOptions<R>,
    cfl_advective: R,
    cfl_diffusive: R,
    warm_start: bool,
    staircase_noslip: bool,
    spectral_diffusion: bool,
}

impl<R: CfdScalar> DecNsConfigReady<R> {
    fn new(nu: R, dt: R) -> Self {
        let safety = R::from_f64(0.9).expect("0.9 lifts into R");
        Self {
            nu,
            dt,
            cg_options: HodgeDecomposeOptions::default(),
            cfl_advective: safety,
            cfl_diffusive: safety,
            warm_start: false,
            staircase_noslip: false,
            spectral_diffusion: false,
        }
    }

    /// Replace the projection CG options (tolerance, iteration budget).
    pub fn cg_options(mut self, opts: HodgeDecomposeOptions<R>) -> Self {
        self.cg_options = opts;
        self
    }

    /// Replace the CFL safety factors (advective, diffusive). Validated at `build`.
    pub fn cfl_factors(mut self, advective: R, diffusive: R) -> Self {
        self.cfl_advective = advective;
        self.cfl_diffusive = diffusive;
        self
    }

    /// Enable projection warm start (off by default).
    pub fn warm_start(mut self) -> Self {
        self.warm_start = true;
        self
    }

    /// Use the staircase immersed no-slip instead of the aperture-resolved cut-face
    /// rows (the validation-comparison path).
    pub fn staircase_noslip(mut self) -> Self {
        self.staircase_noslip = true;
        self
    }

    /// Opt into spectral evaluation of the viscous term (fully periodic lattices
    /// only). Validated at materialization.
    pub fn spectral_diffusion(mut self) -> Self {
        self.spectral_diffusion = true;
        self
    }

    /// Finalize the configuration, validating the physical and numerical knobs.
    ///
    /// # Errors
    /// * `PhysicsError::NumericalInstability` when `ν` or the CFL factors are not finite.
    /// * `PhysicsError::PhysicalInvariantBroken` when `ν` is negative, `dt` is not
    ///   finite/positive, or a CFL factor is not positive.
    pub fn build(self) -> Result<DecNsConfig<R>, PhysicsError> {
        if !self.nu.is_finite() {
            return Err(PhysicsError::NumericalInstability(
                "DecNsConfig: viscosity must be finite".into(),
            ));
        }
        if self.nu < R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DecNsConfig: viscosity cannot be negative".into(),
            ));
        }
        if !self.dt.is_finite() || self.dt <= R::zero() {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DecNsConfig: dt must be finite and positive".into(),
            ));
        }
        if !self.cfl_advective.is_finite()
            || self.cfl_advective <= R::zero()
            || !self.cfl_diffusive.is_finite()
            || self.cfl_diffusive <= R::zero()
        {
            return Err(PhysicsError::PhysicalInvariantBroken(
                "DecNsConfig: CFL safety factors must be finite and positive".into(),
            ));
        }
        Ok(DecNsConfig {
            nu: self.nu,
            dt: self.dt,
            cg_options: self.cg_options,
            cfl_advective: self.cfl_advective,
            cfl_diffusive: self.cfl_diffusive,
            warm_start: self.warm_start,
            staircase_noslip: self.staircase_noslip,
            spectral_diffusion: self.spectral_diffusion,
        })
    }
}

/// An owned, validated DEC NS solver configuration carrying no manifold borrow.
/// Materialize it against a manifold and a boundary-zone set to obtain the marcher.
#[derive(Debug, Clone)]
pub struct DecNsConfig<R: CfdScalar> {
    nu: R,
    dt: R,
    cg_options: HodgeDecomposeOptions<R>,
    cfl_advective: R,
    cfl_diffusive: R,
    warm_start: bool,
    staircase_noslip: bool,
    spectral_diffusion: bool,
}

impl<R: CfdScalar> DecNsConfig<R> {
    /// The configured kinematic viscosity.
    pub fn nu(&self) -> R {
        self.nu
    }

    /// The configured time step.
    pub fn dt(&self) -> R {
        self.dt
    }

    /// The projection CG options.
    pub fn cg_options(&self) -> &HodgeDecomposeOptions<R> {
        &self.cg_options
    }

    /// Materialize the manifold-bound marcher from this configuration, the manifold,
    /// and a composable boundary-zone set (`()` for a closed domain). The borrows
    /// stay inside the returned marcher; the configuration is untouched and reusable.
    ///
    /// # Errors
    /// Every rejection of [`DecNsSolver::with_zones`] and the optional
    /// spectral-diffusion / CFL-factor application.
    pub fn materialize_with_zones<'m, const D: usize, Z>(
        &self,
        manifold: &'m Manifold<LatticeComplex<D, R>, R>,
        zones: Z,
    ) -> Result<DecNsSolver<'m, D, R>, PhysicsError>
    where
        Z: BoundaryZone<D, R>,
    {
        let mut solver = DecNsSolver::with_zones(manifold, self.nu, self.dt, zones)?
            .with_cg_options(self.cg_options.clone())
            .with_cfl_factors(self.cfl_advective, self.cfl_diffusive)?;
        if self.warm_start {
            solver = solver.with_warm_start();
        }
        if self.staircase_noslip {
            solver = solver.with_staircase_noslip();
        }
        if self.spectral_diffusion {
            solver = solver.with_spectral_diffusion()?;
        }
        Ok(solver)
    }
}
