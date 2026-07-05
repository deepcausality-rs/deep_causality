/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering (the `inspect` seam). The gating sequence lives in `model`; all printing
//! lives here, main.rs stays the study expression.

use crate::constants::{EXIT_AREA_M2, INLET_AREA_M2, LENGTH_M, P0_PA, T0_K, THROAT_AREA_M2};
use crate::model::{self, MapRow};
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

pub fn print_footer(out_path: &Path) {
    let first_critical = model::subsonic_exit_pressure_ratio();
    let shock_at_exit = model::exit_shock_back_pressure_ratio();
    println!("\noperating map written to {}", out_path.display());
    println!(
        "analytic references: first critical p/p0 = {first_critical:.4}, exit-plane shock at p/p0 = {shock_at_exit:.4}\n"
    );
}
