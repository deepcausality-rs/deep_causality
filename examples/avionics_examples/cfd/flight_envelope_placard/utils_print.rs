/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering (the `inspect` seam). The gating sequence lives in `model`; all printing
//! lives here, main.rs stays the study expression.

use crate::model::PlacardRow;
use std::path::Path;

pub fn print_intro(matrix_path: &Path) {
    println!(
        "=== Flight-envelope placard table: pointwise study over a Mach-altitude matrix ===\n"
    );
    println!("Matrix: {}", file_name(matrix_path));
}

pub fn print_rows(rows: &[PlacardRow]) {
    println!("\n{} grid points computed:\n", rows.len());
    println!("     M    alt(km)    q(kPa)    T0(K)    qdot(W/cm2)");
    for r in rows {
        println!(
            "  {:>4.2}  {:>7.1}  {:>8.1}  {:>7.1}  {:>11.2}",
            r.mach, r.alt_km, r.q_kpa, r.t0_k, r.qdot_w_cm2
        );
    }
    println!();
}

pub fn print_footer(out_path: &Path) {
    println!("\nPlacard table written: {}", file_name(out_path));
}

/// The file name of a path (not the absolute manifest path), so the recorded reference output is
/// portable across machines and checkouts.
fn file_name(p: &Path) -> std::borrow::Cow<'_, str> {
    p.file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default()
}
