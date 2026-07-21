/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stage 3 corridor composition stages, split by the stage under test: [`regime_tests`] classifies,
//! [`navigation_tests`] dead-reckons, [`lift_tests`] steers, [`gate_tests`] bounds the bank, and [`burn_tests`] enforces
//! the powered-descent axes.
//!
//! The fixtures every submodule shares live here, so a change to the shared field or step context
//! lands in one place.

mod burn_tests;
mod gate_tests;
mod lift_tests;
mod navigation_tests;
mod regime_tests;

use deep_causality_cfd::{
    Ambient, BlackoutTrigger, CoupledField, CyberneticCorrect, SafetyEnvelope, StepContext,
};

pub fn field() -> CoupledField<f64> {
    CoupledField::new(Ambient::new(0.01_f64, 0.0, None))
}

pub fn ctx(step: usize) -> StepContext<'static, 2, f64> {
    StepContext::<2, f64>::qtt(0.1, step)
}

/// A band that denies the link for any real plasma (omega_p >> 1 rad/s for any positive n_e).
pub fn denying_trigger() -> BlackoutTrigger<f64> {
    BlackoutTrigger::new(1.0)
}

/// The corridor envelope: heat <= 1e6 W/m^2, g <= 12, |bank| <= 0.5 rad, and no burn axes.
pub fn envelope() -> SafetyEnvelope<f64> {
    SafetyEnvelope::new(1.0e6, 12.0, 0.5)
}

/// The gate on the corridor path, where the envelope carries no powered-descent axes.
pub fn gate() -> CyberneticCorrect<f64> {
    CyberneticCorrect::new(envelope())
}
