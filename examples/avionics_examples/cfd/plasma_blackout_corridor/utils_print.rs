/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering (the `inspect` seam and the leg/provenance prints). The gating sequences and
//! the branch scoring live in `model`; all printing lives here, `main` stays the corridor program.
//! The final verdict renders itself (`Display`); `main` prints it.

use crate::FloatType;
use crate::model::{BranchRow, LegSnapshot, pick_committed};
use avionics_examples::shared::constants::{CAP, COMMS_BAND_RAD_S, L};
use deep_causality_core::EffectLog;

pub fn print_intro() {
    println!("=== Plasma-blackout corridor: one continuous descent ===\n");
    println!(
        "Carrier: {n}x{n} compressible tensor-train layer (bond cap {cap}) | comms band: GPS L1 (w = {band:.3e} rad/s)",
        n = 1usize << L,
        cap = CAP,
        band = COMMS_BAND_RAD_S,
    );
    println!(
        "Compressed time: one coupled step = {} s of flight; one solver pseudo-time step per coupled step (quasi-steady layer)",
        avionics_examples::shared::constants::DT_FLIGHT,
    );
    println!("Precision: {} \n", core::any::type_name::<FloatType>());
}

pub fn print_leg(leg: &LegSnapshot) {
    println!("--- Leg: {} ---", leg.name);
    println!(
        "  marched {} steps to {:.1} km, Mach {:.1} | regime: {} (Kn = {:.2e}) | link: {}",
        leg.steps,
        leg.altitude_km,
        leg.mach,
        leg.regime_model,
        leg.knudsen,
        if leg.gnss_denied {
            "GNSS DENIED"
        } else {
            "GNSS available"
        },
    );
    println!(
        "  n_e peak = {:.3e} m^-3 | w_p = {:.3e} rad/s | q = {:.3e} W/m^2 | {:.1} g",
        leg.ne_peak, leg.plasma_frequency, leg.heat_flux, leg.g_load,
    );
    println!(
        "  nav error vs truth = {:.4} m | position variance = {:.4e} m^2 | log entries: {}\n",
        leg.nav_err_m, leg.nav_var, leg.log_entries,
    );
}

/// The `inspect` seam of each counterfactual round: the committed branch (minimum trajectory miss)
/// is marked, exactly as the campaign's gates score it.
pub fn print_branches(title: &str, rows: &[BranchRow]) {
    let committed = pick_committed(rows);
    println!("--- {title} ---");
    println!(
        "  bank     peak heat      thermal load   dwell      miss (traj)   miss (t2 x-check)  peak n_e      alternation"
    );
    for (i, b) in rows.iter().enumerate() {
        let mark = if i == committed { " <- committed" } else { "" };
        println!(
            "  {:>5.1}deg  {:>10.3e}  {:>12.3e}  {:>7.2} s  {:>9.3} m  {:>13.4} m  {:>10.3e}   {}{}",
            b.bank_deg,
            b.outcome.peak_heat_flux,
            b.outcome.thermal_load,
            b.outcome.blackout_dwell,
            b.outcome.miss_distance,
            b.t2_miss_m,
            b.ne_peak,
            if b.has_alternation_marker {
                "!!ContextAlternation!!"
            } else {
                "(none)"
            },
            mark,
        );
    }
    println!("  committed rule: minimum trajectory-derived miss to the shared aim point\n");
}

pub fn print_provenance(log: &EffectLog) {
    println!("--- Provenance (the carried EffectLog, the whole descent) ---");
    // The full log is dominated by the per-step bank boundings of the over-cap branch world.
    // Keep the audit counts; show the transition entries that tell the corridor's story.
    let rendered = format!("{log}");
    let entries: Vec<&str> = rendered.lines().skip(1).collect();
    let boundings = entries
        .iter()
        .filter(|e| e.contains("bank correction bounded"))
        .count();
    println!(
        "  {} entries total ({} bank-correction boundings by the cybernetic gate, omitted below):",
        entries.len(),
        boundings
    );
    for entry in entries
        .iter()
        .filter(|e| !e.contains("bank correction bounded"))
    {
        // Strip the microsecond timestamp prefix for readability.
        let msg = entry.split_once("] ").map(|(_, m)| m).unwrap_or(entry);
        println!("    {msg}");
    }
    println!();
}
