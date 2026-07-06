/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! All console output for the INS / GNSS-blackout example, kept out of `main` so the orchestration
//! reads cleanly.

use crate::FloatType;
use crate::model::{Epoch, NavProcess};

pub fn print_intro(sat: &str) {
    println!("=== INS / GNSS-Blackout Clock Holdover (real Galileo {sat}) ===\n");
}

pub fn print_loaded(sat: &str, n_orbit: usize, n_clock: usize) {
    println!(
        "Loaded real {sat}: {n_orbit} orbit epochs, {n_clock} clock samples (GFZ MGEX week 1877 day 0)"
    );
}

pub fn print_stream_summary(stream: &[Epoch], threshold: FloatType) {
    let denied = stream
        .iter()
        .filter(|e| e.denial_indicator > threshold)
        .count();
    let r_min = stream.iter().map(|e| e.radius_m).fold(f64::MAX, f64::min) / 1000.0;
    let r_max = stream.iter().map(|e| e.radius_m).fold(0.0, f64::max) / 1000.0;
    let span_h = stream.last().map(|e| e.t_sec).unwrap_or(0.0) / 3600.0;
    println!(
        "Stream: {} epochs, span {:.1} h; orbit radius {:.0}–{:.0} km; modelled outage ≈ {} epochs.\n",
        stream.len(),
        span_h,
        r_min,
        r_max,
        denied
    );
}

/// Print the result block, the gates, the EffectLog head, and the finding. Returns whether all gates
/// passed (so `main` can set the exit status).
pub fn report(sat: &str, open: &NavProcess, closed: &NavProcess) -> bool {
    let (os, cs) = (open.state(), closed.state());

    println!("--- Result (real {sat}, day-long stream, extended GNSS outage) ---");
    println!(
        "Open loop  (pure INS, no GNSS fix): final position error {:.0} m",
        os.final_err
    );
    println!(
        "Closed loop (regime-gated intervene): pre-outage {:.1} m, peak-in-outage {:.0} m, post-reacquire {:.1} m",
        cs.pre_blackout_err, cs.peak_blackout_err, cs.post_reacq_err
    );
    println!(
        "Clock holdover across the outage (vs the real measured {sat} clock):\n  relativistic-model carry  {:.1} ns   |   naive last-rate hold  {:.1} ns",
        cs.clock_carry_err_relativistic, cs.clock_carry_err_naive
    );
    println!(
        "GNSS regime changes detected: {}; GNSS fixes applied: {}\n",
        cs.regime_changes, cs.correction_count
    );

    println!("--- Gates ---");
    let g1 = gate(
        "two regime changes detected (blackout entry + exit)",
        cs.regime_changes == 2,
    );
    let g2 = gate(
        "closed loop bounded + snaps back (post-reacquire < peak-in-outage)",
        cs.peak_blackout_err.is_finite() && cs.post_reacq_err < cs.peak_blackout_err.max(1.0),
    );
    let g3 = gate(
        "GNSS coupling is the lever (open-loop error >> closed-loop final)",
        os.final_err > 10.0 * cs.final_err.max(1.0),
    );
    let g4 = gate(
        "relativistic clock carry no worse than naive hold, and bounded",
        cs.clock_carry_err_relativistic <= cs.clock_carry_err_naive + 1.0
            && cs.clock_carry_err_relativistic < 1.0e5,
    );
    let g5 = gate(
        "intervene loop fired (GNSS fixes applied)",
        cs.correction_count > 0,
    );

    println!("\n--- Closed-loop EffectLog (regime changes + interventions; head) ---");
    let log_text = format!("{:?}", closed.logs());
    for line in log_text
        .split(',')
        .map(str::trim)
        .filter(|l| l.contains("REGIME") || l.contains("Intervention"))
        .take(8)
    {
        println!("  {line}");
    }

    let all = g1 && g2 && g3 && g4 && g5;
    if all {
        let factor = (os.final_err / cs.final_err.max(1.0)).max(1.0);
        println!(
            "\n=== FINDING: on real Galileo {sat} data, the regime detector flags the GNSS blackout, the\n\
             corrective intervene loop holds the INS bounded with GNSS and is withheld through the dark\n\
             (open loop drifts {factor:.0}× further), and the relativistic clock kernel is carried forward\n\
             across the outage. This is the general GPS-denial navigation/timing core: regime-gated GNSS\n\
             denial + carried relativistic clock + corrective reacquisition, in one auditable CausalFlow. ==="
        );
    }
    all
}

fn gate(label: &str, pass: bool) -> bool {
    println!("  [{}] {label}", if pass { "PASS" } else { "FAIL" });
    pass
}
