/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering (the intro and the `inspect` table seam). The gating sequences and the
//! ensemble reduction live in `model`; main stays the study expression, and the verdict renders
//! itself.

use crate::constants;
use crate::model::WorldRow;

pub fn print_intro() {
    println!("=== Plasma-blackout weather-dispersion table: six counterfactual atmospheres ===\n");
    println!(
        "Baseline: the corridor's validated descent | {} coupled steps per world, {} s of flight each",
        constants::STEPS,
        constants::STEPS as f64 * avionics_examples::shared::constants::DT_FLIGHT,
    );
    println!(
        "IMU thermal model: bias departure 1 + {:.3}/K away from the calibration point; filter priors stay standard-day",
        constants::IMU_THERMAL_COEFF_PER_K,
    );
    println!(
        "Monte Carlo: {} deterministic receiver-noise draws per condition ({} descents total)\n",
        constants::MC_DRAWS,
        constants::WEATHER.len() * constants::MC_DRAWS,
    );
}

/// The dispersion table (the digital-twin deliverable): navigation cells carry error bars over the
/// receiver-noise draws.
pub fn print_rows(rows: &[WorldRow]) {
    println!(
        "  world          dT K   rho    IMU dep  onset s  exit s  dwell s   peak n_e     peak q      drift(dark) m     terminal m"
    );
    for r in rows {
        println!(
            "  {:<13} {:>5.0}  {:>5.2}  {:>7.2}  {:>7.1}  {:>6.1}  {:>7.1}  {:>10.3e}  {:>10.3e}  {:>7.2} +- {:>4.2}  {:>6.3} (max {:.3})",
            r.name,
            r.d_temp,
            r.rho_scale,
            r.bias_departure,
            r.onset_s,
            r.exit_s,
            r.dwell_s,
            r.ne_max,
            r.q_max,
            r.drift_mean_m,
            r.drift_sd_m,
            r.terminal_mean_m,
            r.terminal_max_m,
        );
    }
    println!();
}
