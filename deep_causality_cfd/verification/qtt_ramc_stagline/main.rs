/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # RAM-C stagnation line — the Tier-B Stage-4 milestone (shock fitting + reused Tier-A LER)
//!
//! On the stagnation streamline the bow shock is a 1-D **fitted interface**: the freestream crosses it and
//! the **exact Rankine–Hugoniot jump** sets the post-shock state, with `T₂` the **real transported energy**
//! (retiring the Tier-A recovery-temperature reconstruction). The smooth post-shock relaxation zone drives
//! the reused Tier-A Saha/Park-2T ionization → electron density → plasma frequency → blackout. The example
//! self-verifies (exit nonzero on break) against the RAM-C II flight data: the post-shock temperature band,
//! the peak electron density, blackout onset, and the `O(1)` rank of the smooth post-shock profile.
//!
//! Usage:
//!
//! ```text
//! cargo run --release -p deep_causality_cfd --example qtt_ramc_stagline
//! ```

mod config;
mod print_utils;

use deep_causality_cfd::{FittedNormalShock, fail};
use deep_causality_tensor::Truncation;

/// Working precision.
pub type FloatType = f64;

fn main() {
    println!(
        "=== RAM-C stagnation line: exact Rankine–Hugoniot fit + reused Tier-A ionization ===\n"
    );
    println!(
        "Flight: M = {}, γ = {}, T_inf = {} K, n_inf = {:.0e} m^-3 (RAM-C II ~71 km)\n",
        config::MACH,
        config::GAMMA,
        config::T_INF,
        config::NUMBER_DENSITY
    );

    let shock = FittedNormalShock::<FloatType>::new(config::ft(config::GAMMA))
        .unwrap_or_else(|e| fail("fitted shock", e));
    let post = shock
        .post_shock(
            config::ft(config::T_INF),
            config::ft(config::NUMBER_DENSITY),
            config::ft(config::MACH),
        )
        .unwrap_or_else(|e| fail("post-shock state", e));
    // Post-shock residence time t_res = standoff / u₂ over which ionization lags equilibrium.
    let u2 = config::FREESTREAM_VELOCITY * post.u_ratio;
    let residence_time = config::STANDOFF_M / u2;
    let equilibrium = shock
        .stagnation_blackout(&post, config::ft(config::COMMS_BAND_RAD_S))
        .unwrap_or_else(|e| fail("Saha equilibrium reference", e));
    let outcome = shock
        .stagnation_line_blackout(
            &post,
            config::ft(residence_time),
            config::ft(config::COMMS_BAND_RAD_S),
        )
        .unwrap_or_else(|e| fail("nonequilibrium stagnation blackout", e));
    println!(
        "Residence time t_res = standoff/u2 = {:.3e} s  (Saha-equilibrium upper bound n_e = {:.3e} m^-3)\n",
        residence_time, equilibrium.electron_density
    );

    let trunc = Truncation::<FloatType>::by_tol(1e-10).unwrap_or_else(|e| {
        eprintln!("truncation: {e:?}");
        std::process::exit(2);
    });
    let (profile_bond, _peak) = shock
        .relaxation_profile_bond(
            &post,
            config::PROFILE_L,
            config::ft(config::RELAX_LENGTH),
            &trunc,
        )
        .unwrap_or_else(|e| fail("relaxation profile", e));

    print_utils::render(&post, &outcome, profile_bond);
    if print_utils::verify(&post, &outcome, profile_bond) {
        print_utils::summary(&outcome);
    } else {
        std::process::exit(1);
    }
}
