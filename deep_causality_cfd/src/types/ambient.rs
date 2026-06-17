/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::CfdScalar;
use deep_causality_physics::BodyForceOneForm;

/// The per-step ambient a marcher reads each step: kinematic viscosity, the
/// freestream inflow speed, and an optional body force. Coupling stages and
/// dynamic-law counterfactuals write into it *between* steps (e.g. `ν(T)` feedback
/// or a thrust-driven freestream); the marching rate only reads it. When no
/// coupling is present the ambient is constant and the march reproduces the
/// construction-fixed behaviour.
#[derive(Debug, Clone)]
pub struct Ambient<R: CfdScalar> {
    nu: R,
    freestream: R,
    body_force: Option<BodyForceOneForm<R>>,
}

impl<R: CfdScalar> Ambient<R> {
    /// Construct an ambient from kinematic viscosity, freestream speed, and an
    /// optional body force.
    pub fn new(nu: R, freestream: R, body_force: Option<BodyForceOneForm<R>>) -> Self {
        Self {
            nu,
            freestream,
            body_force,
        }
    }

    /// Kinematic viscosity for this step.
    pub fn nu(&self) -> &R {
        &self.nu
    }

    /// Freestream inflow speed for this step.
    pub fn freestream(&self) -> &R {
        &self.freestream
    }

    /// The optional body force for this step.
    pub fn body_force(&self) -> Option<&BodyForceOneForm<R>> {
        self.body_force.as_ref()
    }

    /// Drive the viscosity from a coupling stage (e.g. temperature-dependent `ν(T)`).
    pub fn set_nu(&mut self, nu: R) {
        self.nu = nu;
    }

    /// Drive the freestream speed from a coupling stage (e.g. a thrust schedule).
    pub fn set_freestream(&mut self, freestream: R) {
        self.freestream = freestream;
    }

    /// Drive the body force from a coupling stage.
    pub fn set_body_force(&mut self, body_force: Option<BodyForceOneForm<R>>) {
        self.body_force = body_force;
    }
}
