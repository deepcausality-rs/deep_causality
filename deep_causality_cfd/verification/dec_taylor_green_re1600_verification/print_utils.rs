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
/// summary on stderr. `n` pins the cell-volume normalization and the convective time step.
///
/// The summary's dissipation figure is the **maximum over the sampled horizon**, not necessarily an
/// interior maximum of `−dE*/dt*`. Which of three cases holds is decided here from the first and the
/// last sample attaining that maximum, and the summary says which one it is: the maximum falls
/// strictly before the horizon (an interior peak), the last sample ties a maximum already reached
/// earlier (a terminal plateau, which does not establish a turnover), or the last sample alone holds
/// the maximum (an endpoint value on a still-rising curve).
///
/// At the shipped 16³ / `t*_max = 10` configuration it does not turn over: the maximum lands on the
/// final sample at `t* ≈ 10.05` with the curve still rising, so that number is an endpoint value,
/// not a peak. Extending the same 16³ run to `t*_max = 40` does turn it over, at `t* ≈ 14.06` —
/// late against the `t* ≈ 9` the README quotes for the published DNS curve, because 16³ is grossly
/// under-resolved. Neither figure is gated; the two gates in [`verify`] are internal invariants.
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
    // Indices into the dissipation samples (the `t* = 0` row carries no dissipation and is not a
    // candidate): the first and the last sample attaining `peak.1`, plus the final sample of the
    // horizon. The two peak indices differ when the maximum recurs, which is what separates a
    // terminal plateau from an interior peak.
    let mut peak_first_idx = 0usize;
    let mut peak_last_idx = 0usize;
    let mut last_idx = 0usize;
    let e0 = e_prev;
    emit(t_star, e_prev, FloatType::zero());

    for (i, &e_raw) in energy[1..].iter().enumerate() {
        let e = e_raw / volume;
        t_star += dt_star;
        let dissipation = (e_prev - e) / dt_star;
        emit(t_star, e, dissipation);
        if i == 0 || dissipation > peak.1 {
            // A strictly larger sample (or the first one, which seeds the maximum): the reported
            // time is the earliest the maximum is reached.
            peak = (t_star, dissipation);
            peak_first_idx = i;
            peak_last_idx = i;
        } else if dissipation >= peak.1 {
            // A tie with the running maximum: keep the reported time, record the later occurrence.
            peak_last_idx = i;
        }
        last_idx = i;
        e_prev = e;
    }

    let e_t = e_prev;
    eprintln!(
        "\nmarched to t* = {:.2}: E*/E0 = {:.4}, max sampled dissipation {:.6} at t* = {:.2}",
        Into::<f64>::into(t_star),
        Into::<f64>::into(e_t / e0),
        Into::<f64>::into(peak.1),
        Into::<f64>::into(peak.0)
    );
    if peak_last_idx < last_idx {
        eprintln!("the curve turned over inside the horizon, so that maximum is an interior peak.");
    } else if peak_first_idx < peak_last_idx {
        eprintln!(
            "the last sample ties that maximum: the curve is back at its maximum at the horizon, so \
             the run does not establish a turnover. Extend t*_max (or refine the grid) to place the peak."
        );
    } else {
        eprintln!(
            "the maximum is the last sample: the curve is still rising at the horizon, so that is an \
             endpoint value, not the dissipation peak. Extend t*_max (or refine the grid) to reach it."
        );
    }
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
