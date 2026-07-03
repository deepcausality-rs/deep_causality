/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering and the operating-map gates. The boolean from [`report`] tells the
//! caller whether to exit nonzero; all printing lives here, main.rs stays logic.

use crate::FloatType;
use crate::constants::{
    AREA_MACH_BAND, CELLS, EXIT_AREA_M2, INLET_AREA_M2, LENGTH_M, P0_PA, SHOCK_BAND_CELLS,
    SONIC_AT_THROAT_BAND_CELLS, T0_K, THROAT_AREA_M2,
};
use crate::model::MapRow;
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::Gates;
use std::path::Path;

pub fn print_intro(schedule_len: usize, schedule_path: &Path) {
    println!("=== Nozzle operating map: back-pressure sweep over a converging-diverging duct ===");
    println!(
        "geometry: inlet {INLET_AREA_M2} / throat {THROAT_AREA_M2} / exit {EXIT_AREA_M2} m^2 over {LENGTH_M} m; reservoir {P0_PA} Pa, {T0_K} K"
    );
    println!(
        "schedule: {schedule_len} back pressures from {}\n",
        schedule_path.display()
    );
}

pub fn print_rows(rows: &[MapRow]) {
    println!("  p_back/p0   M_exit   shock x [m]      Cf");
    for row in rows {
        let shock = row
            .shock_x
            .map(|x| format!("{x:>10.4}"))
            .unwrap_or_else(|| "      none".to_string());
        println!(
            "     {:>5.2}    {:>5.3}   {shock}   {:>6.3}",
            row.p_ratio, row.mach_exit, row.cf
        );
    }
}

pub fn print_footer(
    out_path: &Path,
    first_critical: FloatType,
    shock_at_exit: FloatType,
    elapsed_s: f64,
) {
    println!("\noperating map written to {}", out_path.display());
    println!(
        "analytic references: first critical p/p0 = {first_critical:.4}, exit-plane shock at p/p0 = {shock_at_exit:.4}"
    );
    println!("wall-clock: {elapsed_s:.1} s\n");
}

/// Everything the gates need: the reduced rows, the analytic regime boundaries, and the
/// per-row closed-form shock positions (computed by main.rs, printed and judged here).
pub struct GateInputs<'a> {
    pub rows: &'a [MapRow],
    pub scheduled: usize,
    pub first_critical: FloatType,
    /// Closed-form shock station per row, `None` where the regime has no internal shock.
    pub analytic_shocks: &'a [Option<FloatType>],
}

/// The operating-map gates. Returns false on any regression; the caller exits nonzero.
pub fn report(inputs: &GateInputs<'_>) -> bool {
    let h = ft(LENGTH_M) / ft(CELLS as f64);
    let throat_x = ft(LENGTH_M) * ft(0.5);

    let mut sonic_ok = true;
    let mut sonic_detail = String::from("every choked row crosses Mach 1 at the throat");
    let mut shock_ok = true;
    let mut shock_detail = String::from("every internal-shock row matches the closed form");
    let mut profile_ok = true;
    let mut profile_detail = String::from("every shock-free row tracks the area-Mach relation");
    let mut cf_ok = true;
    let mut cf_detail = String::from("thrust coefficient finite and positive on every row");

    for (row, analytic) in inputs.rows.iter().zip(inputs.analytic_shocks) {
        let choked = row.p_ratio < inputs.first_critical;
        if choked {
            match row.sonic_x {
                Some(x) if (x - throat_x).abs() <= ft(SONIC_AT_THROAT_BAND_CELLS) * h => {}
                Some(x) => {
                    sonic_ok = false;
                    sonic_detail = format!(
                        "p_ratio {:.2}: sonic crossing at x = {x:.4} m, throat at {throat_x:.4} m, band {:.4} m",
                        row.p_ratio,
                        ft(SONIC_AT_THROAT_BAND_CELLS) * h
                    );
                }
                None => {
                    sonic_ok = false;
                    sonic_detail = format!(
                        "p_ratio {:.2}: choked row never reaches Mach 1",
                        row.p_ratio
                    );
                }
            }
        }
        match (analytic, row.shock_x) {
            (Some(a), Some(x)) if (x - *a).abs() <= ft(SHOCK_BAND_CELLS) * h => {}
            (Some(a), Some(x)) => {
                shock_ok = false;
                shock_detail = format!(
                    "p_ratio {:.2}: shock at {x:.4} m, closed form {a:.4} m, band {:.4} m",
                    row.p_ratio,
                    ft(SHOCK_BAND_CELLS) * h
                );
            }
            (Some(a), None) => {
                shock_ok = false;
                shock_detail = format!(
                    "p_ratio {:.2}: the closed form places a shock at {a:.4} m, none reported",
                    row.p_ratio
                );
            }
            (None, _) => {
                if row
                    .area_mach_dev
                    .is_some_and(|dev| dev > ft(AREA_MACH_BAND))
                {
                    profile_ok = false;
                    profile_detail = format!(
                        "p_ratio {:.2}: worst interior deviation {:.4} exceeds the {AREA_MACH_BAND} band",
                        row.p_ratio,
                        row.area_mach_dev.unwrap_or_default()
                    );
                }
            }
        }
        if !(row.cf.is_finite() && row.cf > ft(0.0)) {
            cf_ok = false;
            cf_detail = format!("p_ratio {:.2}: Cf = {}", row.p_ratio, row.cf);
        }
    }

    Gates::new("nozzle_operating_map")
        .gate(
            "schedule integrity",
            inputs.rows.len() == inputs.scheduled,
            format!(
                "{} of {} scheduled back pressures produced a converged march",
                inputs.rows.len(),
                inputs.scheduled
            ),
        )
        .gate(
            "choked rows reach Mach 1 at the throat",
            sonic_ok,
            sonic_detail,
        )
        .gate(
            "internal shocks match the closed form",
            shock_ok,
            shock_detail,
        )
        .gate(
            "shock-free profiles track the area-Mach relation",
            profile_ok,
            profile_detail,
        )
        .gate("thrust coefficient is physical", cf_ok, cf_detail)
        .finish()
}
