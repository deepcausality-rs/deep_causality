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
pub mod coupling_tests;
#[cfg(test)]
pub mod frequency_tests;
#[cfg(test)]
pub mod march_case_tests;
#[cfg(test)]
pub mod mms_tests;
#[cfg(test)]
pub mod operator_study_tests;
