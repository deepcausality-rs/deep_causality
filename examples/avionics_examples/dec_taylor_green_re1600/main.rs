/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 3D Taylor–Green vortex at Re 1600, DEC-native
//!
//! The smooth 3D Taylor–Green vortex transitions toward turbulence, and the
//! kinetic-energy dissipation-rate curve `−dE*/dt*` against the published
//! DNS reference data is the standard structure-preservation benchmark.
//!
//! Three DeepCausality abstractions appear together here:
//!
//! - **The DEC solver** from `deep_causality_physics`: velocity is an edge
//!   1-form for the entire solve, each `Rk4` stage evaluates the
//!   Leray-projected rate `P(−i_u ω − ν Δ_dR u♭)` — the projector *is* the
//!   incompressibility equation — and the marching state is the
//!   [`SolenoidalField`](deep_causality_physics::SolenoidalField)
//!   type-state that only projection can construct.
//! - **The causal flow** sequences the two stages — seed, then march. Each
//!   binds onto the previous, so a CG failure or CFL violation
//!   short-circuits the chain through the effect's error channel.
//! - **Precision is a parameter**: every struct in the model is generic
//!   over `R: RealField`, and the single `FloatType` alias below selects
//!   the precision for the manifold metric, the projection CG, the `Rk4`
//!   march, and the energy series alike. Values are cast only at the
//!   display boundary.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --example dec_taylor_green_re1600 [grid] [t_star_max]
//! ```
//!
//! `grid` defaults to 16 (a smoke-scale run); the reporting resolutions
//! from the Stage 1 roadmap are 64–128, which take minutes to hours of
//! unpreconditioned CG time. Output is CSV on stdout
//! (`t_star,kinetic_energy_per_vol,dissipation_rate`), with time in
//! convective units `t* = t·k·U` so the curve overlays the reference data
//! directly. CI never runs the reporting resolutions; the library tests
//! gate correctness, this program produces the recognizable artifact.

mod model;
mod print_utils;

use deep_causality_core::CausalFlow;

/// Change this to `f32` for low precision, `f64` for standard precision,
/// or `Float106` (also `use deep_causality_num::Float106;`) for high
/// precision. The whole pipeline — metric, de Rham seeding, the projected
/// `Rk4` stages with every CG solve, and the energy series — runs at this
/// precision.
pub type FloatType = f64;

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args
        .next()
        .map(|a| a.parse().expect("grid must be an integer"))
        .unwrap_or(16);
    let t_star_max: f64 = args
        .next()
        .map(|a| a.parse().expect("t_star_max must be a number"))
        .unwrap_or(10.0);

    eprintln!(
        "=== DEC-native 3D Taylor-Green at Re {} ===\ngrid {n}^3, horizon t* = {t_star_max}, precision {}\n",
        model::RE,
        core::any::type_name::<FloatType>()
    );

    // The lattice manifold and the solver own the heavyweight geometry at
    // the working precision; the flow threads the lightweight payloads
    // (the seeded cochain, the report) between the stages and short-circuits on the first error.
    let manifold = model::unit_manifold3::<FloatType>(n);
    let solver = match model::solver(&manifold, n) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("solver configuration failed: {e:?}");
            std::process::exit(1);
        }
    };

    // ── The CausalFlow chain ────────────────────────────────────────────
    //   seed ──► march ──► print
    // Each stage is a plain `Value -> Result<U, CausalityError>`; the flow
    // unwraps the value and short-circuits the error channel for us.
    // ────────────────────────────────────────────────────────────────────
    CausalFlow::effect()
        .try_step(|_| model::stage_seed(&solver, &manifold, n))
        .try_step(|seeded| model::stage_march(&solver, &manifold, n, t_star_max, seeded))
        .run(
            |report| print_utils::print_csv(&report),
            |err| {
                eprintln!("DEC Taylor-Green pipeline failed: {err:?}");
                std::process::exit(1);
            },
        );
}
