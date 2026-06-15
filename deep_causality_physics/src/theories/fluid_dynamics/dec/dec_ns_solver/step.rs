/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The projected march step: `Rk4` over the **projected rate** (the
//! Leray projector sits inside each stage, per the governing equation of
//! `cfd-gap.md` §2 — the marched ODE is exactly the projected dynamics,
//! with no splitting error), then the type-state re-entry projection
//! (near-free: its input is already divergence-free, so the CG terminates
//! almost immediately), then the CFL guard. Pure numerics in the arrow,
//! fallible plumbing in `Result`; a CG failure inside a stage is deferred
//! through a `Cell` to the step boundary, where it short-circuits.

use alloc::format;
use core::cell::Cell;

use deep_causality_calculus::Rk4;
use deep_causality_haft::Arrow;

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::solenoidal_field::SolenoidalField;
use crate::quantities::fluid_dynamics::velocity_one_form::VelocityOneForm;
use crate::theories::fluid_dynamics::dec::diagnostics::{dec_divergence_residual, dec_max_speed};
use crate::theories::fluid_dynamics::dec::step_output::StepOutput;

use super::DecNsSolver;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

impl<const D: usize, R: DecNsScalar> DecNsSolver<'_, D, R> {
    /// Advances the state by one time step.
    ///
    /// The chain is exactly the design's bind sequence:
    ///
    /// ```text
    /// SolenoidalField ──as_one_form──► VelocityOneForm
    ///     ──Rk4::run over P∘rhs──► VelocityOneForm  (the arrow: projected stages)
    ///     ──leray_project──► SolenoidalField        (type-state re-entry; near-free)
    ///     ──cfl_check──► StepOutput                 (fallible: violation short-circuits)
    /// ```
    ///
    /// Each RK4 stage evaluates the projected rate (one CG solve per
    /// stage); the increments are therefore divergence-free and so is
    /// their RK4 combination, which makes the re-entry projection a
    /// near-no-op solve that exists to keep [`SolenoidalField`]'s
    /// construction contract (projection is the only path in). There is no
    /// public path that marches an unprojected field.
    ///
    /// # Errors
    /// * `PhysicsError::TopologyError` when a stage's (or the re-entry's)
    ///   projection CG does not converge within its budget; a stage
    ///   failure is recorded in a deferred slot during the arrow run and
    ///   surfaced here.
    /// * `PhysicsError::PhysicalInvariantBroken` when the advective or
    ///   diffusive CFL limit rejects the configured `dt`, naming the
    ///   violated limit and the actual value.
    pub fn step(&self, state: &SolenoidalField<R>) -> Result<StepOutput<R>, PhysicsError> {
        // The arrow: RK4 over the projected rate. `Rk4` requires an
        // infallible `Fn(&S) -> S`, so a CG failure inside a stage parks
        // its error in the deferred slot and yields a zero rate; the slot
        // is checked (and short-circuits) immediately after the run.
        let u = VelocityOneForm::from_raw(state.as_one_form().clone());
        let deferred: Cell<Option<PhysicsError>> = Cell::new(None);
        let n1 = state.as_one_form().len();
        let rk4 = Rk4::new(self.dt, |s: &VelocityOneForm<R>| {
            // On wall-bounded lattices the rate's projector is the
            // constrained (no-slip ∩ divergence-free) projection, so each
            // stage rate is already exactly in the no-slip subspace.
            match self.rate.eval_projected(s, &self.cg_options) {
                Ok(rate) => rate,
                Err(e) => {
                    deferred.set(Some(e));
                    VelocityOneForm::from_raw(
                        deep_causality_tensor::CausalTensor::new(
                            alloc::vec![R::zero(); n1],
                            alloc::vec![n1],
                        )
                        // Coverage exemption: a 1-D zero tensor of the
                        // state's length cannot fail to allocate.
                        .expect("1-D tensor allocation cannot fail"),
                    )
                }
            }
        });
        let advanced = rk4.run(u);
        if let Some(e) = deferred.take() {
            return Err(e);
        }

        // Type-state re-entry: the advanced field is a combination of
        // divergence-free increments, so this projection is consistent
        // and terminates almost immediately. Its error branch is a
        // documented coverage exemption: any CG budget that starves this
        // near-no-op solve starves the full stage solves above first,
        // which fail through the deferred slot. Kept as `?` for defense
        // in depth.
        // The open-boundary projection: no-slip walls zeroed, the prescribed inflow edges held at
        // their (lifted) value with their flux counted, and the outflow reference pressure-pinned.
        // With empty inflow/reference (closed domains) this is bit-identical to the constrained
        // projection.
        // Aperture-resolved bodies carry weighted cut-face rows; the re-entry must enforce them on
        // the state, since the plain Leray gradient correction `dφ` would otherwise reintroduce wall
        // slip (`Cᵀ dφ ≠ 0`). With empty rows this is bit-identical to the binary open re-entry.
        let (projected, _potential) = SolenoidalField::from_open_leray_projection_weighted_opts(
            &advanced,
            self.manifold,
            self.rate.no_slip_edges(),
            self.rate.inflow_edges(),
            self.rate.reference_vertices(),
            self.rate.no_slip_rows(),
            &self.cg_options,
            None,
        )?;
        // No-slip chain stage: re-assert the exact zeros on the
        // wall-tangential set, then the prescribed moving-wall values (the
        // inhomogeneous lift) as the step's final operations. The
        // constrained projection already produced the zeros and ignores
        // constrained-edge input values (`P(u) = P(u − lift)` exactly), so
        // the lift is simply re-applied. Both are no-ops on periodic
        // lattices, keeping that path bit-unchanged.
        let projected = projected
            .constrain_edges(self.rate.no_slip_edges())
            .with_lift(&self.lift);

        // Bind 2: the CFL guard on the projected state; violation
        // short-circuits.
        let max_speed = dec_max_speed(self.manifold, projected.as_one_form())?;
        self.cfl_check(max_speed)?;

        let divergence_residual = dec_divergence_residual(self.manifold, projected.as_one_form())?;

        Ok(StepOutput::new(projected, max_speed, divergence_residual))
    }

    /// Enforces the advective limit `dt ≤ C_adv · dx_min / max|u|`
    /// (skipped while the field is at rest) and the diffusive limit
    /// `dt ≤ C_diff · dx_min² / (2·D·ν)` (skipped when `ν = 0`).
    pub(super) fn cfl_check(&self, max_speed: R) -> Result<(), PhysicsError> {
        if max_speed > R::zero() {
            let advective_limit = self.cfl_advective * self.dx_min / max_speed;
            if self.dt > advective_limit {
                return Err(PhysicsError::PhysicalInvariantBroken(format!(
                    "CFL violation (advective): dt {} exceeds the limit {} \
                     (C_adv {} · dx_min {} / max|u| {})",
                    self.dt, advective_limit, self.cfl_advective, self.dx_min, max_speed
                )));
            }
        }

        let nu = self.rate.nu();
        if nu > R::zero() {
            let two_d = R::from_usize(2 * D)
                // Coverage exemption: small integers lift into every real field.
                .expect("2·D lifts into R");
            let diffusive_limit = self.cfl_diffusive * self.dx_min * self.dx_min / (two_d * nu);
            if self.dt > diffusive_limit {
                return Err(PhysicsError::PhysicalInvariantBroken(format!(
                    "CFL violation (diffusive): dt {} exceeds the limit {} \
                     (C_diff {} · dx_min² {} / (2·{D}·ν {}))",
                    self.dt,
                    diffusive_limit,
                    self.cfl_diffusive,
                    self.dx_min * self.dx_min,
                    nu
                )));
            }
        }

        Ok(())
    }
}
