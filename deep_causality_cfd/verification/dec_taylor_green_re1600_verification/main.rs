/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 3D Taylor–Green vortex at Re 1600, DEC-native — via the CfdFlow DSL
//!
//! The smooth 3D Taylor–Green vortex transitions toward turbulence, and the kinetic-energy
//! dissipation-rate curve `−dE*/dt*` against the published DNS reference data is the standard
//! structure-preservation benchmark.
//!
//! The case is declared through the `deep_causality_cfd` configuration layer
//! ([`config::build_march_config`]) and run through the **CfdFlow** DSL: a periodic cubic mesh, the DEC
//! incompressible solver, the Taylor–Green seed, and the kinetic-energy observation. `CfdFlow`
//! lowers onto the same projected DEC step the hand-rolled solver used, so the marched energy
//! series is reproduced exactly; [`print_utils::render`] turns it into the dissipation CSV.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example dec_taylor_green_re1600_verification [grid] [t_star_max]
//! ```
//!
//! `grid` defaults to 16 (a smoke-scale run). Output is CSV on stdout
//! (`t_star,kinetic_energy_per_vol,dissipation_rate`); the closing summary is on stderr.

mod config;
mod print_utils;

use config::RE;
use deep_causality_cfd::{CfdFlow, fail};

/// The working precision for the whole computation. **This is the single alias to change**: the
/// manifold metric, the projection CG, the RK4 march, and the energy series all run at this
/// precision (`f32`, `f64`, or `Float106` with `use deep_causality_num::Float106;`). The
/// configuration and display layers import it from here.
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
        RE,
        core::any::type_name::<FloatType>()
    );

    // ── The CfdFlow case ────────────────────────────────────────────────
    //   CfdConfigBuilder (config) ──► materialize geometry (B1) ──► CfdFlow run ──► render
    // Configuration and workflow composition are separated: the container is built by
    // `CfdConfigBuilder`, the caller owns the geometry, and `CfdFlow` composes + runs.
    // ────────────────────────────────────────────────────────────────────
    // Build configuration
    let steps = config::build_steps(n, t_star_max);
    let case_config = config::build_march_config(n, steps)
        .unwrap_or_else(|e| fail("DEC Taylor-Green configuration", e));

    //  Materialize geometry
    let manifold = case_config
        .materialize()
        .unwrap_or_else(|e| fail("DEC Taylor-Green geometry", e));

    // Solve
    let results = CfdFlow::march(&case_config)
        .on(&manifold)
        .run()
        .unwrap_or_else(|e| fail("DEC Taylor-Green pipeline", e));

    // Print results, then self-verify the structure-preservation invariant (exit nonzero on break).
    print_utils::render(&results, n);
    if !print_utils::verify(&results, n) {
        std::process::exit(1);
    }
}
