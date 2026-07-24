/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Display layer for the Re-1600 Taylor–Green run: renders the `CfdFlow` report's
//! kinetic-energy series into the dissipation CSV (stdout) and the closing summary (stderr).
//!
//! All bookkeeping runs at the working precision [`FloatType`]; values are cast to `f64` only at
//! the `println!`/`eprintln!` boundary — the single display-boundary downcast.

use crate::FloatType;
use crate::config;
use deep_causality_cfd::{EvidenceClass, Report};
use deep_causality_num::Zero;

/// Render the dissipation curve: the CSV header + per-step rows on stdout, then the closing
/// summary (E\*/E0 and peak dissipation) on stderr. `n` pins the cell-volume normalization and the
/// convective time step.
pub fn render(report: &Report<FloatType>, n: usize) {
    let energy = report
        .series("kinetic_energy")
        .expect("kinetic_energy series");
    let volume = config::volume(n);
    let dt_star = config::dt_star(n);

    println!("t_star,kinetic_energy_per_vol,dissipation_rate");
    let mut t_star = FloatType::zero();
    let mut e_prev = energy[0] / volume;
    let mut peak = (FloatType::zero(), FloatType::zero()); // (t_star, dissipation)
    let e0 = e_prev;
    emit(t_star, e_prev, FloatType::zero());

    for &e_raw in &energy[1..] {
        let e = e_raw / volume;
        t_star += dt_star;
        let dissipation = (e_prev - e) / dt_star;
        emit(t_star, e, dissipation);
        if dissipation > peak.1 {
            peak = (t_star, dissipation);
        }
        e_prev = e;
    }

    let e_t = e_prev;
    eprintln!(
        "\nmarched to t* = {:.2}: E*/E0 = {:.4}, peak dissipation {:.6} at t* = {:.2}",
        Into::<f64>::into(t_star),
        Into::<f64>::into(e_t / e0),
        Into::<f64>::into(peak.1),
        Into::<f64>::into(peak.0)
    );
    eprintln!(
        "compare the dissipation column against the published Re-1600 DNS curve (references.md)."
    );
}

/// Self-verification (internal consistency / structure preservation). The unforced viscous
/// Taylor–Green vortex can only **lose** kinetic energy, so a correct projected DEC march must keep
/// the per-step energy non-increasing (no spurious energy production) and drive the final energy
/// strictly below the initial. Returns `false` on violation; `main` exits nonzero. No reference data
/// is needed — this is an invariant the DEC structure guarantees at any grid/precision.
pub fn verify(report: &Report<FloatType>, n: usize) -> bool {
    let energy = report
        .series("kinetic_energy")
        .expect("kinetic_energy series");
    let volume = config::volume(n);
    let e0 = energy[0] / volume;
    // Allow tiny roundoff energy growth (relative to E0) before flagging spurious production.
    let tol = e0 * config::ft(1e-9);
    let mut ok = true;
    let mut prev = e0;
    // Evidence class: **tripwire** for both. These are internal structure-preservation invariants
    // of the discretisation (an unforced viscous flow cannot gain energy), not comparisons against
    // an external reference. The DNS dissipation curve this run emits for comparison is NOT gated —
    // at 16³ it is grossly under-resolved, which the README states.
    //
    // BREAKING CONDITIONS: break the viscous sign (or the projection) so energy grows, and gate 1
    // fails; run with nu = 0 so nothing dissipates, and gate 2 fails.
    let mut max_rise = config::ft(0.0);
    for &e_raw in &energy[1..] {
        let e = e_raw / volume;
        if e > prev + tol && e - prev > max_rise {
            max_rise = e - prev;
        }
        prev = e;
    }
    let monotone = max_rise <= config::ft(0.0);
    eprintln!(
        "  [{}] [{}] kinetic energy monotonically non-increasing: max step-to-step rise {:.3e} (tol {:.1e})",
        if monotone { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
        Into::<f64>::into(max_rise),
        Into::<f64>::into(tol),
    );
    ok &= monotone;

    let dissipated = prev < e0;
    eprintln!(
        "  [{}] [{}] net dissipation over the horizon: E*/E0 = {:.4}",
        if dissipated { "PASS" } else { "FAIL" },
        EvidenceClass::Tripwire,
        Into::<f64>::into(prev / e0),
    );
    ok &= dissipated;
    ok
}

/// One CSV row: `t_star,kinetic_energy_per_vol,dissipation_rate`. The working-precision values are
/// cast to `f64` here — the only display-boundary downcast.
fn emit(t_star: FloatType, energy_per_vol: FloatType, dissipation: FloatType) {
    println!(
        "{:.4},{:.8},{:.8}",
        Into::<f64>::into(t_star),
        Into::<f64>::into(energy_per_vol),
        Into::<f64>::into(dissipation)
    );
}
