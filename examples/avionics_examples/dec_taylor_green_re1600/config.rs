/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Configuration layer for the Re-1600 Taylor–Green run: every case parameter and the
//! single `Flow::march` case builder. `main.rs` orchestrates the run and renders the CSV.
//!
//! The case is expressed entirely through the `deep_causality_cfd` **Flow** DSL: a periodic
//! cubic mesh, the DEC incompressible solver at `ν = 1/(k·Re)`, the classic Taylor–Green
//! seed, and the kinetic-energy observation. `Flow::march` lowers onto the same projected DEC
//! step the hand-rolled solver used, so the marched energy series is reproduced exactly.
//!
//! Precision is a parameter: every derived quantity is computed at the working precision
//! [`FloatType`], and the exact `f64` specifications (`Re`, the CFL step, `π`) enter once
//! through [`ft`] (a `from_f64` lift) and never come back down. Switching the alias re-runs
//! the whole computation at that precision; values are cast to `f64` only at the display
//! boundary in `main`.

use crate::FloatType;
use deep_causality_cfd::{CfdConfigBuilder, MarchConfig, Mesh, Observe, PhysicsError, Seed};
use deep_causality_num::{FromPrimitive, One};

/// The benchmark Reynolds number of the workshop case (exact specification).
pub const RE: f64 = 1600.0;

/// CFL-safe time step on the unit-spacing lattice (`max|u| ≈ 1`, default safety 0.9).
pub const CFL_DT: f64 = 0.2;

/// Lifts an exact `f64` specification into the working precision [`FloatType`] through
/// `FromPrimitive` (not `From<f64>`), so the same lift serves `f32`, `f64`, and `Float106`.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// The unit wavenumber of the `[0, n]³` lattice at the working precision: `k = 2π/n`.
pub fn wavenumber(n: usize) -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft(n as f64)
}

/// The kinematic viscosity at `Re = 1600`: on this lattice `U = 1` and `L = 1/k`, so
/// `ν = U·L/Re = 1/(k·Re)`, at the working precision.
pub fn viscosity(n: usize) -> FloatType {
    FloatType::one() / (wavenumber(n) * ft(RE))
}

/// The convective time step `dt* = k·CFL_DT`, at the working precision.
pub fn dt_star(n: usize) -> FloatType {
    wavenumber(n) * ft(CFL_DT)
}

/// The cell-volume normalization `n³`, at the working precision.
pub fn volume(n: usize) -> FloatType {
    ft((n * n * n) as f64)
}

/// The number of march steps to reach the convective horizon `t*_max`. A loop count derived
/// from the `f64` CLI horizon and the `f64` step spec, so it is computed in `f64`.
pub fn build_steps(n: usize, t_star_max: f64) -> usize {
    (t_star_max / (2.0 * std::f64::consts::PI / n as f64 * CFL_DT)).ceil() as usize
}

/// The `MarchConfig` container for an `n³` periodic Taylor–Green vortex marched `steps` steps,
/// observing the kinetic-energy series — built through `CfdConfigBuilder` (configuration), to be
/// composed and run by the `CfdFlow` DSL. Generic over the working precision.
///
/// # Errors
/// Any solver-config or container validation failure.
pub fn build_march_config(
    n: usize,
    steps: usize,
) -> Result<MarchConfig<3, FloatType, (), ()>, PhysicsError> {
    CfdConfigBuilder::march::<3, FloatType>("tgv-re1600")
        .mesh(Mesh::periodic_cube(n))
        .solver(
            CfdConfigBuilder::dec_ns()
                .viscosity(viscosity(n))
                .time_step(ft(CFL_DT))
                .build()
                .expect("solver configuration"),
        )
        .seed(Seed::TaylorGreenVortex)
        .march_for(steps)
        .observe(Observe::default().kinetic_energy())
        .build()
}
