/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering (the `inspect` seam). The gating sequence lives in `model`; all printing
//! lives here, main.rs stays the study expression.

use crate::constants::{DIAMETER_M, F_STRUCT_HZ};
use crate::model::MarginRow;
use std::path::Path;

pub fn print_intro(schedule_len: usize, schedule_path: &Path) {
    println!("=== Vortex-shedding resonance margin: airspeed sweep over a circular member ===");
    println!(
        "member D = {:.1} mm, structural mode f_struct = {F_STRUCT_HZ} Hz (stated demonstration value)",
        DIAMETER_M * 1e3
    );
    println!(
        "schedule: {schedule_len} airspeeds from {}\n",
        file_name(schedule_path)
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

pub fn print_footer(table_path: &Path) {
    println!("margin table written to {}\n", file_name(table_path));
}

/// The file name of a path (not the absolute manifest path), so the recorded reference output is
/// portable across machines and checkouts.
fn file_name(p: &Path) -> std::borrow::Cow<'_, str> {
    p.file_name().map(|n| n.to_string_lossy()).unwrap_or_default()
}
