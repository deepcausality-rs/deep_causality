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
use deep_causality_cfd::Report;
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
