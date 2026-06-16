/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer: the manufactured-solution verification config, built through
//! `CfdConfigBuilder`. The physical constants are exact `f64` specifications lifted into the working
//! precision [`FloatType`] via [`ft`]; the autodiff and kernel arithmetic run at `FloatType`.

use crate::FloatType;
use deep_causality_cfd::{CfdConfigBuilder, PhysicsError, TaylorGreen, VerifyConfig};
use deep_causality_num::FromPrimitive;

/// Kinematic viscosity `ν` (m²/s).
pub const NU: f64 = 0.05;
/// Density `ρ` (kg/m³).
pub const RHO: f64 = 1.0;
/// Sample time.
pub const T0: f64 = 0.0;
/// Rk4 amplitude-march step.
pub const DT: f64 = 0.005;
/// Rk4 amplitude-march step count.
pub const STEPS: usize = 200;

/// Lift an exact `f64` constant into the working precision [`FloatType`].
pub fn ft(x: f64) -> FloatType {
    <FloatType as FromPrimitive>::from_f64(x).expect("constant lifts into FloatType")
}

/// The MMS-verification config: the Taylor–Green manufactured solution sampled at `(0.7, 1.1, 0)`,
/// with a 200-step `Rk4` amplitude march. Built through `CfdConfigBuilder`; run by `CfdFlow::verify`.
///
/// # Errors
/// Any verification-config validation failure (e.g. a missing sample point).
pub fn build_verify_config() -> Result<VerifyConfig<FloatType, TaylorGreen>, PhysicsError> {
    CfdConfigBuilder::verify::<FloatType, _>("tgv-mms", TaylorGreen::new(NU, RHO))
        .sample_at([ft(0.7), ft(1.1), ft(0.0)], T0)
        .amplitude_march(ft(DT), STEPS)
        .build()
}
