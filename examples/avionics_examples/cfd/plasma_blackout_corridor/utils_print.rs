/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering plus the corridor's coupled validation gates (the re-pinned Stage 5.2 set):
//! flow-resolved blackout window, the RAM-C II anchor at the 61 km passage, real INS drift and
//! reacquisition, real steering, trajectory-derived miss spread, tensor compression, bounded
//! rebuilds, and the wall-clock budget. The boolean from [`report`] tells the caller whether to
//! exit nonzero.

use crate::FloatType;
use crate::constants::{
    CAP, COMMS_BAND_RAD_S, DIVERGENCE_MIN_M, DT_FLIGHT, L, MAX_REBUILDS, RAMC_NE_REFERENCE,
    WALL_CLOCK_BUDGET_S,
};
use crate::model::{BranchScore, LegSnapshot};
use crate::utils::ft;
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
        "Compressed time: one coupled step = {DT_FLIGHT} s of flight; one solver pseudo-time step per coupled step (quasi-steady layer)",
    );
    println!(
        "Precision: {} (the single alias to change)\n",
        core::any::type_name::<FloatType>()
    );
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

pub fn print_branches(branches: &[BranchScore], committed: usize) {
    println!(
        "--- Counterfactual bank commands (forked from the shared flow-resolved onset) ---"
    );
    println!(
        "  bank    peak heat      thermal load   dwell      miss (traj)   miss (t2 x-check)  peak n_e      alternation"
    );
    for (i, b) in branches.iter().enumerate() {
        let mark = if i == committed { " <- committed" } else { "" };
        println!(
            "  {:>3.0}deg  {:>10.3e}  {:>12.3e}  {:>7.2} s  {:>9.3} m  {:>13.4} m  {:>10.3e}   {}{}",
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

/// Everything the gates read: the four leg snapshots, the scored branches, and the run-level
/// witnesses.
pub struct GateInputs<'a> {
    pub leg1: &'a LegSnapshot,
    pub leg2: &'a LegSnapshot,
    pub leg3: &'a LegSnapshot,
    pub leg4: &'a LegSnapshot,
    pub branches: &'a [BranchScore],
    pub committed: usize,
    pub compression: (usize, usize),
    pub rebuilds: usize,
    pub elapsed_s: f64,
    /// The rendered provenance log of the full descent (the regime-transition witness).
    pub regime_log: &'a str,
}

/// The coupled validation gates. Returns `false` on any regression; the caller exits nonzero.
pub fn report(inputs: &GateInputs<'_>) -> bool {
    let GateInputs {
        leg1,
        leg2,
        leg3,
        leg4,
        branches,
        committed,
        compression,
        rebuilds,
        elapsed_s,
        regime_log,
    } = *inputs;
    println!("--- Coupled validation gates ---");
    let mut all = true;
    let mut gate = |label: &str, pass: bool, detail: String| {
        println!(
            "  [{}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        all &= pass;
    };

    let legs = [leg1, leg2, leg3, leg4];
    gate(
        "(0) corridor integrity",
        legs.iter().all(|l| !l.errored),
        "no leg captured a step error (the envelope held)".into(),
    );

    // Onset found as an event, still denied through the peak passage, exit found as an event,
    // with a nonzero dwell between them. Ordering is structural: onset -> peak -> exit.
    let onset_found = leg1.gnss_denied && leg1.steps < crate::constants::STEPS;
    let exit_found = !leg3.gnss_denied && leg3.steps < crate::constants::STEPS;
    let dwell_steps = leg2.steps + leg3.steps;
    gate(
        "(1) flow-resolved blackout window",
        onset_found && leg2.gnss_denied && exit_found && dwell_steps > 0,
        format!(
            "onset at step {} ({:.1} km), denied through the peak, exit at {:.1} km after a {:.1} s dwell",
            leg1.steps,
            leg1.altitude_km,
            leg3.altitude_km,
            dwell_steps as f64 * DT_FLIGHT,
        ),
    );

    let ne_ok = (ft(RAMC_NE_REFERENCE / 5.0)..=ft(RAMC_NE_REFERENCE * 5.0)).contains(&leg2.ne_peak);
    gate(
        "(2) peak n_e vs the RAM-C II anchor",
        ne_ok,
        format!(
            "n_e = {:.3e} m^-3 at the {:.1} km passage, in [{:.1e}, {:.1e}] around the flight \
             anchor {:.0e} m^-3 (evolved state, Park two-temperature controller)",
            leg2.ne_peak,
            leg2.altitude_km,
            RAMC_NE_REFERENCE / 5.0,
            RAMC_NE_REFERENCE * 5.0,
            RAMC_NE_REFERENCE,
        ),
    );

    // Drift is measured mid-dwell (the peak passage, still denied); the exit pause has already
    // folded its first fix, so it witnesses the collapse, not the drift.
    let drift = leg2.nav_err_m > leg1.nav_err_m && leg2.nav_var > leg1.nav_var;
    let reacq = leg4.nav_err_m < leg2.nav_err_m && leg4.nav_var < leg2.nav_var;
    gate(
        "(3) real INS drift -> reacquisition",
        drift && reacq,
        format!(
            "err {:.4} m -> {:.4} m dead-reckoning to the peak passage, {:.4} m after \
             reacquisition; variance {:.3e} -> {:.3e} -> {:.3e} m^2",
            leg1.nav_err_m,
            leg2.nav_err_m,
            leg4.nav_err_m,
            leg1.nav_var,
            leg2.nav_var,
            leg4.nav_var,
        ),
    );

    gate(
        "(4a) regime change",
        regime_log.contains("regime -> slip") && regime_log.contains("regime -> continuum"),
        "the descent crossed a Knudsen band (slip -> continuum), logged as provenance events"
            .into(),
    );

    gate(
        "(4b) multiphysics chain",
        leg2.ne_peak > ft(0.0)
            && leg2.heat_flux > ft(0.0)
            && leg2.g_load > ft(0.0)
            && leg2.nav_var > ft(0.0),
        "evolved flow -> reacting plasma -> steered aero force -> loads -> navigation all live in \
         one coupling"
            .into(),
    );

    // Steering is real: the committed branch's terminal state diverges from the zero-bank
    // branch's, and the trajectory-derived misses spread across branches.
    let zero_bank = branches.iter().position(|b| b.bank_deg == 0.0).unwrap_or(0);
    let sep: [FloatType; 3] =
        core::array::from_fn(|i| branches[committed].terminal[i] - branches[zero_bank].terminal[i]);
    let divergence = crate::utils::norm3(sep);
    let misses: Vec<FloatType> = branches.iter().map(|b| b.outcome.miss_distance).collect();
    let hi = misses
        .iter()
        .copied()
        .fold(ft(f64::MIN), |a, x| if x > a { x } else { a });
    let lo = misses
        .iter()
        .copied()
        .fold(ft(f64::MAX), |a, x| if x < a { x } else { a });
    gate(
        "(4c) counterfactual steering",
        branches.len() >= 2
            && branches.iter().all(|b| b.has_alternation_marker)
            && (committed == zero_bank || divergence > ft(DIVERGENCE_MIN_M))
            && hi > lo,
        format!(
            "{} alternated worlds from one shared onset; committed-vs-ballistic terminal \
             separation {:.2} m; trajectory-derived miss spread [{:.2}, {:.2}] m",
            branches.len(),
            divergence,
            lo,
            hi
        ),
    );

    let (bond, dense) = compression;
    gate(
        "(4d) tensor compression",
        (1..=CAP).contains(&bond),
        format!(
            "the committed branch's final evolved state re-quantizes at peak bond {bond} (cap \
             {CAP}) vs the dense {dense}-cell grid",
        ),
    );

    gate(
        "(5a) bounded schedule rebuilds",
        rebuilds <= MAX_REBUILDS,
        format!("{rebuilds} carrier rebuild(s) while following the descent (cap {MAX_REBUILDS})"),
    );

    gate(
        "(5b) wall-clock budget",
        elapsed_s < WALL_CLOCK_BUDGET_S,
        format!("{elapsed_s:.1} s elapsed (budget {WALL_CLOCK_BUDGET_S:.0} s)"),
    );

    println!();
    if all {
        println!("=== All corridor gates passed. ===");
    } else {
        println!("=== Corridor gate REGRESSION: see the FAIL lines above. ===");
    }
    all
}
