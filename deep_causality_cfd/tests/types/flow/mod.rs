/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_cfd::{
    BoundaryZone, CfdFlow, CfdScalar, MarchConfig, PhysicsError, PhysicsStage, Report,
};

/// Shared test helper: materialize the config's geometry (B1) and run it through `CfdFlow`.
#[allow(dead_code)]
pub fn run_march<const D: usize, R: CfdScalar, Z, C>(
    config: MarchConfig<D, R, Z, C>,
) -> Result<Report<R>, PhysicsError>
where
    Z: BoundaryZone<D, R> + Clone,
    C: PhysicsStage<D, R>,
{
    let manifold = config.materialize()?;
    CfdFlow::march(&config).on(&manifold).run()
}

#[cfg(test)]
pub mod blackout_tests;
#[cfg(test)]
#[cfg(not(miri))]
pub mod compressible_march_run_tests;
#[cfg(test)]
pub mod corridor_tests;
#[cfg(test)]
pub mod coupled_march_tests;
#[cfg(test)]
pub mod coupling_tests;
#[cfg(test)]
pub mod duct_march_tests;
#[cfg(not(miri))]
mod finite_rate_ionization_tests;
mod flight_sensors_tests;
#[cfg(test)]
pub mod frequency_tests;
#[cfg(test)]
pub mod gates_tests;
#[cfg(test)]
pub mod inheritance_guard_tests;
// IO operations are unsupported under MIRI.
#[cfg(test)]
#[cfg(not(miri))]
pub mod march_case_tests;
#[cfg(test)]
#[cfg(not(miri))]
pub mod march_run_tests;
#[cfg(test)]
pub mod march_state_tests;
#[cfg(test)]
pub mod mms_tests;
#[cfg(test)]
#[cfg(not(miri))]
pub mod operator_study_tests;
#[cfg(test)]
#[cfg(not(miri))]
pub mod qtt_march_pause_tests;
#[cfg(test)]
#[cfg(not(miri))]
pub mod qtt_march_run_tests;
#[cfg(test)]
pub mod report_tests;
#[cfg(test)]
pub mod retropulsion_tests;
#[cfg(test)]
pub mod study_effect_tests;
#[cfg(test)]
pub mod study_error_tests;
#[cfg(test)]
pub mod study_grammar_tests;
#[cfg(test)]
pub mod study_warning_tests;
#[cfg(test)]
pub mod sweep_tests;
mod terminal_descent_tests;
mod throttle_guidance_tests;
// Filesystem round trips are unsupported under MIRI.
#[cfg(test)]
#[cfg(not(miri))]
pub mod state_snapshot_tests;
// IO operations are unsupported under MIRI.
#[cfg(test)]
#[cfg(not(miri))]
pub mod uncertain_march_run_tests;
#[cfg(test)]
pub mod verify_tests;
