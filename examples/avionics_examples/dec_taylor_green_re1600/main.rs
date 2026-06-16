/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # 3D Taylor–Green vortex at Re 1600, DEC-native — via the Flow DSL
//!
//! The smooth 3D Taylor–Green vortex transitions toward turbulence, and the kinetic-energy
//! dissipation-rate curve `−dE*/dt*` against the published DNS reference data is the standard
//! structure-preservation benchmark.
//!
//! The case is declared through the `deep_causality_cfd` **Flow** DSL ([`config::march_case`]):
//! a periodic cubic mesh, the DEC incompressible solver, the Taylor–Green seed, and the
//! kinetic-energy observation. `Flow::march` lowers onto the same projected DEC step the
//! hand-rolled solver used, so the marched energy series is reproduced exactly; `main` renders
//! it into the dissipation CSV.
//!
//! Usage:
//!
//! ```text
//! cargo run --release --example dec_taylor_green_re1600 [grid] [t_star_max]
//! ```
//!
//! `grid` defaults to 16 (a smoke-scale run). Output is CSV on stdout
//! (`t_star,kinetic_energy_per_vol,dissipation_rate`), with time in convective units
//! `t* = t·k·U` so the curve overlays the reference data directly; the closing summary is on
//! stderr.

mod config;

use config::{FloatType, RE};
use deep_causality_num::Zero;

fn main() {
    let mut args = std::env::args().skip(1);
    let n: usize = args
        .next()
        .map(|a| a.parse().expect("grid must be an integer"))
        .unwrap_or(16);
    let t_star_max: f64 = args
        .next()
        .map(|a| a.parse().expect("t_star_max must be a number"))
        .unwrap_or(10.0);

    eprintln!(
        "=== DEC-native 3D Taylor-Green at Re {} ===\ngrid {n}^3, horizon t* = {t_star_max}, precision {}\n",
        RE,
        core::any::type_name::<FloatType>()
    );

    // ── The Flow case ───────────────────────────────────────────────────
    //   declare (mesh + solver + seed + observe) ──► run ──► render CSV
    // The marched kinetic-energy series comes straight from the DSL Report.
    // ────────────────────────────────────────────────────────────────────
    let steps = config::steps(n, t_star_max);
    let report = match config::march_case(n, steps).run() {
        Ok(report) => report,
        Err(e) => {
            eprintln!("DEC Taylor-Green pipeline failed: {e:?}");
            std::process::exit(1);
        }
    };
    let energy = report
        .series("kinetic_energy")
        .expect("kinetic_energy series");

    // The dissipation curve in convective units, all at the working precision; values are
    // cast to `f64` only at the display boundary inside `emit`.
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

    // Closing summary on stderr (human); the CSV above is the machine-readable artifact.
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

/// One CSV row: `t_star,kinetic_energy_per_vol,dissipation_rate`. The working-precision values
/// are cast to `f64` here — the only display-boundary downcast.
fn emit(t_star: FloatType, energy_per_vol: FloatType, dissipation: FloatType) {
    println!(
        "{:.4},{:.8},{:.8}",
        Into::<f64>::into(t_star),
        Into::<f64>::into(energy_per_vol),
        Into::<f64>::into(dissipation)
    );
}
