/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Console rendering and the placard gates. The boolean from [`report`] tells the caller
//! whether to exit nonzero; all printing lives here, main.rs stays logic.

use crate::FloatType;
use crate::constants::{Q_MAX_PLACARD_KPA, T0_MAX_PLACARD_K};
use avionics_examples::shared::utils::ft;
use deep_causality_cfd::Gates;
use std::path::Path;

pub fn print_intro(matrix_path: &Path) {
    println!(
        "=== Flight-envelope placard table: pointwise study over a Mach-altitude matrix ===\n"
    );
    println!("Matrix: {}", matrix_path.display());
}

pub fn print_rows(rows: &[[FloatType; 5]]) {
    println!("\n{} grid points computed:\n", rows.len());
    println!("     M    alt(km)    q(kPa)    T0(K)    qdot(W/cm2)");
    for r in rows {
        println!(
            "  {:>4.2}  {:>7.1}  {:>8.1}  {:>7.1}  {:>11.2}",
            r[0], r[1], r[2], r[3], r[4]
        );
    }
    println!();
}

/// The row with the largest value in `col` (rows checked non-empty by the caller).
fn peak(rows: &[[FloatType; 5]], col: usize) -> &[FloatType; 5] {
    rows.iter().fold(
        &rows[0],
        |best, r| if r[col] > best[col] { r } else { best },
    )
}

/// The placard gates: every point inside the envelope, offenders named. Returns false on any
/// regression; the caller exits nonzero.
pub fn report(rows: &[[FloatType; 5]], scheduled: usize, out_path: &Path, elapsed_s: f64) -> bool {
    let q_max = ft(Q_MAX_PLACARD_KPA);
    let t0_max = ft(T0_MAX_PLACARD_K);
    let peak_q = peak(rows, 2);
    let peak_t0 = peak(rows, 3);

    let q_offenders: Vec<String> = rows
        .iter()
        .filter(|r| r[2] > q_max)
        .map(|r| format!("q = {:.1} kPa at M {:.2} / {:.1} km", r[2], r[0], r[1]))
        .collect();
    let q_detail = if q_offenders.is_empty() {
        format!(
            "max q = {:.1} kPa at M {:.2} / {:.1} km, inside the {Q_MAX_PLACARD_KPA:.0} kPa placard",
            peak_q[2], peak_q[0], peak_q[1]
        )
    } else {
        format!(
            "{} exceeds the {Q_MAX_PLACARD_KPA:.0} kPa placard",
            q_offenders.join("; ")
        )
    };

    let t0_offenders: Vec<String> = rows
        .iter()
        .filter(|r| r[3] > t0_max)
        .map(|r| format!("T0 = {:.1} K at M {:.2} / {:.1} km", r[3], r[0], r[1]))
        .collect();
    let t0_detail = if t0_offenders.is_empty() {
        format!(
            "max T0 = {:.1} K at M {:.2} / {:.1} km, inside the {T0_MAX_PLACARD_K:.0} K placard",
            peak_t0[3], peak_t0[0], peak_t0[1]
        )
    } else {
        format!(
            "{} exceeds the {T0_MAX_PLACARD_K:.0} K placard",
            t0_offenders.join("; ")
        )
    };

    let all_pass = Gates::new("flight-envelope placard gates")
        .gate("q-max placard", q_offenders.is_empty(), q_detail)
        .gate(
            "stagnation-temperature placard",
            t0_offenders.is_empty(),
            t0_detail,
        )
        .gate(
            "matrix integrity",
            rows.len() == scheduled,
            format!("{} of {scheduled} matrix rows computed", rows.len()),
        )
        .finish();

    println!("\nPlacard table written: {}", out_path.display());
    println!("Wall clock: {elapsed_s:.3} s");
    all_pass
}
