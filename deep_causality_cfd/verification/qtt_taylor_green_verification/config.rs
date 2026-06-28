/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the QTT Taylor–Green verification: every case parameter, the analytic
//! Taylor–Green reference, and the single `QttMarchConfig` case builder. `main.rs` orchestrates the
//! run (the CfdFlow march) and `print_utils.rs` renders + verifies — this file only *describes* the
//! case.
//!
//! The case is expressed through the `deep_causality_cfd` configuration layer
//! ([`build_config`]): a `2^L × 2^L` periodic grid, the `QttIncompressible2d` tensor-train solver, the
//! analytic Taylor–Green seed, and the kinetic-energy / divergence / max-speed / bond observations.
//!
//! Precision is a parameter: the exact `f64` specifications enter once through [`ft`] (a `from_f64`
//! lift) and the whole computation runs at the working precision [`FloatType`].

use crate::FloatType;
use deep_causality_cfd::{
    MarchStop, PhysicsError, QttMarchConfig, QttMarchConfigBuilder, QttObserve,
};
use deep_causality_num::FromPrimitive;
use deep_causality_tensor::Truncation;

/// Kinematic viscosity (exact specification).
pub const NU: f64 = 0.05;
/// Explicit-Euler time step (exact specification).
pub const DT: f64 = 0.01;
/// Number of marched steps (horizon `t = DT·STEPS = 0.2`).
pub const STEPS: usize = 20;
/// Bond-dimension cap for the per-step round (large — let rounding find the true rank).
pub const MAX_BOND: usize = 4096;

/// Lift an exact `f64` specification into the working precision through `FromPrimitive`.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// The grid spacing `Δx = 2π / 2^L` of the periodic `[0, 2π]` axis, at the working precision.
pub fn spacing(l: usize) -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft((1usize << l) as f64)
}

/// The analytic decay factor `e^{-2νt}` over the full horizon, at the working precision.
pub fn decay() -> FloatType {
    ft((-2.0 * NU * DT * STEPS as f64).exp())
}

/// Taylor–Green `u`-velocity `u = −cos(x)sin(y)`.
pub fn tg_u(x: f64, y: f64) -> f64 {
    -(x.cos() * y.sin())
}
/// Taylor–Green `v`-velocity `v = sin(x)cos(y)`.
pub fn tg_v(x: f64, y: f64) -> f64 {
    x.sin() * y.cos()
}

/// The round policy shared by the solver and codec.
pub fn trunc() -> Truncation<FloatType> {
    Truncation::<FloatType>::by_bond(MAX_BOND).expect("bond cap is valid")
}

/// The `QttMarchConfig` container for a `2^L × 2^L` periodic Taylor–Green vortex marched `STEPS`
/// steps, observing kinetic energy, divergence, max speed, and bond dimension — built through
/// `QttMarchConfigBuilder` (configuration), to be composed and run by the `CfdFlow` DSL in `main`.
///
/// # Errors
/// Any builder validation failure.
pub fn build_config(l: usize) -> Result<QttMarchConfig<FloatType>, PhysicsError> {
    let dx = spacing(l);
    QttMarchConfigBuilder::<FloatType>::new()
        .name("qtt-taylor-green")
        .grid(l, l, dx, dx)
        .solver(ft(DT), ft(NU), trunc())
        .taylor_green()?
        .stop(MarchStop::Fixed(STEPS))
        .observe(
            QttObserve::default()
                .kinetic_energy()
                .divergence()
                .max_speed()
                .bond(),
        )
        .build()
}
