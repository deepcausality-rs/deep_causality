/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # QTT Park-2T plasma blackout (Tier-A) — the LER coupling on the incompressible rollout
//!
//! Closes and verifies Gap 2 (Tier-A) of the plasma-blackout corridor: the Park-2T reacting/ionization
//! physics that turns the closed Gap-1 QTT flowfield into the flagship's regime driver
//! (recovery-temperature reconstruction → ionization → electron density → plasma frequency → blackout),
//! via the **Lagging-Equilibrium Relaxation (LER)** between-step coupling hosted in the QTT march
//! (`run_coupled`, design D5/D8).
//!
//! `main` runs the coupled march over a blunt forebody and self-verifies (exit nonzero on break) the six
//! LER acceptance gates: (i) stability at stiffness, (ii) the relaxation kernel against an independent
//! sub-stepped reference, (iii) the
//! mandatory Rankine–Hugoniot temperature band, (iv) ionization lag + rate grounding, (v) counterfactual
//! path-dependence, (vi) electrons produced. Published references are reported as cross-references with
//! Tier-A disclaimers (incompressible rollout; `T_tr` is a recovery-temperature reconstruction).
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_park2t_blackout
//! ```

mod config;
mod print_utils;

use deep_causality_cfd::{Ambient, CfdFlow, CoupledField};

/// The working precision for the whole computation (the single alias to change).
pub type FloatType = f64;

/// The grid: `2^L × 2^L`.
const L: usize = 5;
/// The bond cap for the round policy.
const CAP: usize = 16;

fn main() {
    println!("=== QTT Park-2T plasma blackout (Tier-A, LER on the incompressible rollout) ===\n");
    println!(
        "Case: M = {}, T_inf = {} K, n_tot = {:.1e} m^-3, steps = {}, grid {}^2, precision {}\n",
        config::MACH,
        config::T_INF,
        config::NUMBER_DENSITY,
        config::STEPS,
        1usize << L,
        core::any::type_name::<FloatType>(),
    );

    let cfg = config::build_config(L, CAP).unwrap_or_else(|e| fail("blackout config", e));
    let field = CoupledField::new(Ambient::new(
        config::ft(config::NU),
        config::ft(config::U_INF),
        None,
    ));
    let report = CfdFlow::march(&cfg)
        .run_coupled(
            config::coupling(),
            field,
            config::trigger(),
            config::ft(config::SCALAR_KAPPA),
        )
        .unwrap_or_else(|e| fail("blackout coupled march", e));

    print_utils::render(&report);
    if print_utils::verify(&report) {
        print_utils::summary();
    } else {
        std::process::exit(1);
    }
}

/// Print a stage-failure context and its error on stderr, then exit the process non-zero.
fn fail(context: &str, error: impl core::fmt::Debug) -> ! {
    eprintln!("{context} failed: {error:?}");
    std::process::exit(1);
}
