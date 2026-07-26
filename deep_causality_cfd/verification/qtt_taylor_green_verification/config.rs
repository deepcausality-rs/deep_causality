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
/// Explicit-Euler time step (exact specification). Time integration is **first-order**, and the
/// convergence ladder refines only the spatial grid `N` at this fixed `dt`, so the measured order is
/// the **spatial** order (centered FD + spectral projection, formally 2).
///
/// **The fixed-`dt` Euler error is a temporal floor, and it bounds how far the ladder may be
/// extended.** For mode-1 Taylor–Green the discrete decay factor is `(1 − 2ν·dt·s)^STEPS` with
/// `s = (2 − 2cos dx)/dx²`, against the exact `exp(−2νt)`. The *signed* error of that factor is:
///
/// ```text
///   N =   8    +9.789e-4
///   N =  16    +2.411e-4   order 2.02
///   N =  32    +5.316e-5   order 2.18
///   N =  64    +5.948e-6   order 3.16   <- spatial error now comparable to the temporal floor
///   N = 128    −5.868e-6   order 0.02   <- SIGN CHANGE: the two errors cancel
///   N = 256    −8.823e-6   order −0.59  <- settling onto the temporal floor (≈ −2e-5)
/// ```
///
/// The positive spatial error and the negative temporal error cancel near `N = 64–128`. So the
/// `N = 64` "order 3.16" is a **cancellation artifact, not super-convergence**, and beyond it the
/// observed order collapses. **Maximum usable ladder length: `max_level = 5` (the committed default,
/// `N = 32`)**, where the spatial error `5.3e-5` still sits well above the floor and the 2.02/2.18
/// orders are a legitimate spatial-order measurement. Running the harness at the documented
/// `max_level 7` yields a finest-pair order of ~0.02 and **fails** the `MIN_ORDER` gate in
/// `print_utils` — a documented user action that produces a FAIL. Fixing that needs either a two-sided order
/// gate or `dt` refined with `dx` (`dt ∝ dx²`); both are code changes, recorded and not taken here.
pub const DT: f64 = 0.01;
/// Number of marched steps (horizon `t = DT·STEPS = 0.2`). Fixed across the ladder, so every level
/// carries the same first-order-in-time Euler error, which is the temporal floor described on
/// [`DT`].
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
