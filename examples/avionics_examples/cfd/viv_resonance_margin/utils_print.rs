/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering and the margin gates. The boolean from [`report`] tells the caller
//! whether to exit nonzero; all printing lives here, main.rs stays logic.

use crate::FloatType;
use crate::constants::{DIAMETER_M, F_STRUCT_HZ, MARGIN_MIN, ST_BAND};
use crate::model::MarginRow;
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::Gates;
use std::path::Path;

pub fn print_intro(schedule_len: usize, schedule_path: &Path) {
    println!("=== Vortex-shedding resonance margin: airspeed sweep over a circular member ===");
    println!(
        "member D = {:.1} mm, structural mode f_struct = {F_STRUCT_HZ} Hz (stated demonstration value)",
        DIAMETER_M * 1e3
    );
    println!(
        "schedule: {schedule_len} airspeeds from {}\n",
        schedule_path.display()
    );
}

pub fn print_rows(rows: &[MarginRow]) {
    println!("airspeed [m/s]   Re      St      f_shed [Hz]   margin");
    for r in rows {
        println!(
            "{:>10.2}    {:>6.0}  {:>6.4}   {:>9.1}    {:>6.3}",
            Into::<f64>::into(r.airspeed),
            Into::<f64>::into(r.reynolds),
            Into::<f64>::into(r.strouhal),
            Into::<f64>::into(r.f_shed_hz),
            Into::<f64>::into(r.margin),
        );
    }
    println!();
}

/// The margin gates. Returns false on any regression; the caller exits nonzero.
pub fn report(rows: &[MarginRow], scheduled: usize, table_path: &Path, elapsed_s: f64) -> bool {
    println!("margin table written to {}\n", table_path.display());

    let st_ok = rows
        .iter()
        .all(|r| r.strouhal >= ft(ST_BAND.0) && r.strouhal <= ft(ST_BAND.1));
    let (st_lo, st_hi) = rows.iter().fold(
        (FloatType::INFINITY, -FloatType::INFINITY),
        |(lo, hi), r| (lo.min(r.strouhal), hi.max(r.strouhal)),
    );
    let min_margin = rows
        .iter()
        .fold(FloatType::INFINITY, |m, r| m.min(r.margin));
    let intact = rows.len() == scheduled
        && rows
            .iter()
            .all(|r| r.f_shed_hz.is_finite() && r.strouhal > ft(0.0));

    let ok = Gates::new("viv_resonance_margin")
        .gate(
            "strouhal band",
            st_ok,
            format!(
                "extracted St in [{:.4}, {:.4}], validated band [{}, {}] for this grid",
                Into::<f64>::into(st_lo),
                Into::<f64>::into(st_hi),
                ST_BAND.0,
                ST_BAND.1
            ),
        )
        .gate(
            "resonance margin",
            min_margin >= ft(MARGIN_MIN),
            format!(
                "min |f_struct - f_shed| / f_struct = {:.3}, placard minimum {MARGIN_MIN}",
                Into::<f64>::into(min_margin)
            ),
        )
        .gate(
            "run integrity",
            intact,
            format!(
                "{} of {scheduled} sweeps returned a finite, oscillating wake",
                rows.len()
            ),
        )
        .finish();
    println!("wall-clock: {elapsed_s:.1} s");
    ok
}
