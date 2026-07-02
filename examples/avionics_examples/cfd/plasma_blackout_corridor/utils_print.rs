/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering plus the corridor's coupled validation gates (Stage 5.2): real electron
//! density, real blackout window, real INS drift, reacquisition, and the four required corridor
//! elements (regime change, multiphysics, counterfactuals, tensor compression). The boolean from
//! [`report`] tells the caller whether to exit nonzero.

use crate::FloatType;
use crate::constants::{CAP, COMMS_BAND_RAD_S, L, PEAK, RAMC_NE_REFERENCE};
use crate::model::{BranchScore, LegSnapshot, ft};
use deep_causality_core::EffectLog;

pub fn print_intro() {
    println!("=== Plasma-blackout corridor: flow -> plasma -> navigation -> control ===\n");
    println!(
        "Carrier: {n}x{n} QTT (bond cap {cap}), dt = 4 ms | comms band: GPS L1 (w = {band:.3e} rad/s)",
        n = 1usize << L,
        cap = CAP,
        band = COMMS_BAND_RAD_S,
    );
    println!(
        "Precision: {} (the single alias to change)\n",
        core::any::type_name::<FloatType>()
    );
}

pub fn print_leg(leg: &LegSnapshot) {
    println!("--- Leg: {} ---", leg.name);
    println!(
        "  marched {} steps | regime: {} (Kn = {:.2e}) | link: {}",
        leg.steps,
        leg.regime_model,
        leg.knudsen,
        if leg.gnss_denied {
            "GNSS DENIED"
        } else {
            "GNSS available"
        },
    );
    println!(
        "  n_e peak = {:.3e} m^-3 | w_p = {:.3e} rad/s | q = {:.3e} W/m^2 | {:.2} g",
        leg.ne_peak, leg.plasma_frequency, leg.heat_flux, leg.g_load,
    );
    println!(
        "  nav error vs truth = {:.4} m | position variance = {:.4e} m^2 | log entries: {}\n",
        leg.nav_err_m, leg.nav_var, leg.log_entries,
    );
}

pub fn print_branches(branches: &[BranchScore], committed: usize) {
    println!("--- [5] Counterfactual bank-angle branches (forked from the shared onset) ---");
    println!(
        "  bank    peak heat      thermal load   dwell      t2 miss     peak n_e      alternation"
    );
    for (i, b) in branches.iter().enumerate() {
        let mark = if i == committed { " <- committed" } else { "" };
        println!(
            "  {:>3.0}deg  {:>10.3e}  {:>12.3e}  {:>7.4} s  {:>8.4} m  {:>10.3e}   {}{}",
            b.bank_deg,
            b.outcome.peak_heat_flux,
            b.outcome.thermal_load,
            b.outcome.blackout_dwell,
            b.outcome.miss_distance,
            b.ne_peak,
            if b.has_alternation_marker {
                "!!ContextAlternation!!"
            } else {
                "(none)"
            },
            mark,
        );
    }
    println!("  committed rule: minimum integrated thermal load\n");
}

pub fn print_provenance(log: &EffectLog) {
    println!("--- [7] Provenance (the carried EffectLog, all three legs) ---");
    // The full log is dominated by the per-step bank boundings. Keep the audit counts; show the
    // transition entries that tell the corridor's story.
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

/// The coupled validation gates. Returns `false` on any regression; the caller exits nonzero.
pub fn report(
    leg1: &LegSnapshot,
    leg2a: &LegSnapshot,
    leg2b: &LegSnapshot,
    leg3: &LegSnapshot,
    branches: &[BranchScore],
    compression: (usize, usize),
) -> bool {
    println!("--- Coupled validation gates ---");
    let mut all = true;
    let mut gate = |label: &str, pass: bool, detail: String| {
        println!(
            "  [{}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        all &= pass;
    };

    let legs = [leg1, leg2a, leg2b, leg3];
    gate(
        "(0) corridor integrity",
        legs.iter().all(|l| !l.errored),
        "no leg captured a step error (the envelope held)".into(),
    );

    // Available, then denied at an early onset, denied through the dwell, clear at exit.
    let onset_found = leg2a.gnss_denied && leg2a.steps < PEAK.steps;
    gate(
        "(1) real blackout window",
        !leg1.gnss_denied && onset_found && leg2b.gnss_denied && !leg3.gnss_denied,
        format!(
            "approach available, onset at peak step {}, dwell denied, exit clear",
            leg2a.steps
        ),
    );

    let ne_ok =
        (ft(RAMC_NE_REFERENCE / 5.0)..=ft(RAMC_NE_REFERENCE * 5.0)).contains(&leg2b.ne_peak);
    gate(
        "(2) peak n_e vs the RAM-C II anchor",
        ne_ok,
        format!(
            "n_e = {:.3e} m^-3 in [{:.1e}, {:.1e}] around the flight anchor {:.0e} m^-3 (the \
             Park two-temperature controller suppresses the frozen-RH Saha saturation)",
            leg2b.ne_peak,
            RAMC_NE_REFERENCE / 5.0,
            RAMC_NE_REFERENCE * 5.0,
            RAMC_NE_REFERENCE,
        ),
    );

    let drift = leg2b.nav_var > leg2a.nav_var && leg2b.nav_err_m > leg1.nav_err_m;
    let reacq = leg3.nav_err_m < leg2b.nav_err_m && leg3.nav_var < leg2b.nav_var;
    gate(
        "(3) real INS drift -> reacquisition",
        drift && reacq,
        format!(
            "err {:.4} m -> {:.4} m through the blackout, {:.4} m after the first fix; \
             variance {:.3e} -> {:.3e} -> {:.3e} m^2",
            leg1.nav_err_m,
            leg2b.nav_err_m,
            leg3.nav_err_m,
            leg2a.nav_var,
            leg2b.nav_var,
            leg3.nav_var,
        ),
    );

    gate(
        "(4a) regime change",
        leg1.regime_model != leg2a.regime_model,
        format!(
            "governing model {} -> {} (and the comms-denial transitions in the log)",
            leg1.regime_model, leg2a.regime_model
        ),
    );

    gate(
        "(4b) multiphysics chain",
        leg2b.ne_peak > 0.0 && leg2b.heat_flux > 0.0 && leg2b.g_load > 0.0 && leg2b.nav_var > 0.0,
        "flow -> reacting plasma -> aero force -> loads -> navigation all live in one coupling"
            .into(),
    );

    let loads: Vec<FloatType> = branches.iter().map(|b| b.outcome.thermal_load).collect();
    let hi = loads
        .iter()
        .copied()
        .fold(ft(f64::MIN), |a, x| if x > a { x } else { a });
    let lo = loads
        .iter()
        .copied()
        .fold(ft(f64::MAX), |a, x| if x < a { x } else { a });
    let spread = hi - lo;
    gate(
        "(4c) counterfactual branches",
        branches.len() >= 2
            && spread > ft(0.0)
            && branches.iter().all(|b| b.has_alternation_marker),
        format!(
            "{} alternated worlds from one shared onset; thermal-load spread {:.3e}",
            branches.len(),
            spread
        ),
    );

    let (bond, dense) = compression;
    gate(
        "(4d) tensor compression",
        (1..=CAP).contains(&bond),
        format!(
            "the committed branch's final state re-quantizes at peak bond {bond} (cap {CAP}) vs \
             the dense {dense}-cell grid",
        ),
    );

    println!();
    if all {
        println!("=== All corridor gates passed. ===");
    } else {
        println!("=== Corridor gate REGRESSION: see the FAIL lines above. ===");
    }
    all
}
