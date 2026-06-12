/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Initial-condition seeding: the de Rham map followed by exactly one
//! Leray projection at `t = 0`. An analytically divergence-free field is
//! not discretely divergence-free; the projection makes it so before the
//! first step.

use alloc::format;

use deep_causality_tensor::CausalTensor;

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::solenoidal_field::SolenoidalField;
use crate::quantities::fluid_dynamics::velocity_one_form::VelocityOneForm;

use super::DecNsSolver;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

impl<const D: usize, R: DecNsScalar> DecNsSolver<'_, D, R> {
    /// Seeds the starting state from vertex-sampled vectors (layout
    /// `vertex * D + axis`) through the trapezoid de Rham map, then one
    /// projection.
    ///
    /// # Errors
    /// * `PhysicsError::TopologyError` wrapping a de Rham rejection
    ///   (wrong length, missing metric) or projection CG failure.
    /// * `PhysicsError::NumericalInstability` on non-finite samples
    ///   (through the [`VelocityOneForm`] validation).
    pub fn seed_from_vertex_vectors(
        &self,
        vertex_vectors: &CausalTensor<R>,
    ) -> Result<SolenoidalField<R>, PhysicsError> {
        let edge_form = self
            .manifold
            .de_rham(vertex_vectors)
            .map_err(|e| PhysicsError::TopologyError(format!("de Rham seeding failed: {e}")))?;
        self.project_seed(edge_form)
    }

    /// Seeds the starting state from exact per-edge line integrals
    /// (`∫_e u·dl`, one coefficient per edge) through the validating
    /// passthrough, then one projection.
    ///
    /// # Errors
    /// As [`Self::seed_from_vertex_vectors`].
    pub fn seed_from_edge_integrals(
        &self,
        edge_integrals: &CausalTensor<R>,
    ) -> Result<SolenoidalField<R>, PhysicsError> {
        let edge_form = self
            .manifold
            .de_rham_from_integrals(edge_integrals)
            .map_err(|e| {
                PhysicsError::TopologyError(format!("de Rham integral seeding failed: {e}"))
            })?;
        self.project_seed(edge_form)
    }

    /// The shared `t = 0` projection of a seeded edge cochain.
    fn project_seed(&self, edge_form: CausalTensor<R>) -> Result<SolenoidalField<R>, PhysicsError> {
        let velocity = VelocityOneForm::new(edge_form, self.manifold)?;
        // Wall-bounded lattices seed through the constrained projection
        // (no-slip ∩ divergence-free), so the march starts with exact zeros
        // on the wall-tangential set; the explicit constraint stage then
        // re-asserts them and applies the prescribed moving-wall lift. All
        // are no-ops on periodic lattices.
        let (state, _potential) = SolenoidalField::from_constrained_leray_projection_opts(
            &velocity,
            self.manifold,
            self.rate.no_slip_edges(),
            &self.cg_options,
        )?;
        let state = state
            .constrain_edges(self.rate.no_slip_edges())
            .with_lift(&self.lift);
        Ok(state)
    }
}
