/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The flight-envelope placard table
//!
//! A Mach-altitude test matrix goes in; one placard table comes out. Per grid point the study
//! interpolates the freestream state from a cited US-1976 atmosphere table, computes the
//! dynamic pressure `q = ½·ρ_∞·V²`, the exact Rankine-Hugoniot post-shock stagnation
//! temperature (isentropic below Mach 1, where there is no shock), and the Sutton-Graves
//! stagnation-point heating `q̇ = k·√(ρ_∞/R_n)·V³`. Every point is gated against the stated
//! q-max and stagnation-temperature placards, and any out-of-envelope point is named, not
//! averaged away.
//!
//! This is the pointwise study path on purpose: no march runs and no manifold exists. The
//! matrix rows go through [`sweep`] exactly like march cases would, which is the demonstration:
//! the study shape is the same whether the body per point is a solver run or a closed form.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example flight_envelope_placard
//! # the negative scenario: one point beyond the q-max placard, named, exit 1
//! cargo run --release -p avionics_examples --example flight_envelope_placard \
//!     examples/avionics_examples/cfd/flight_envelope_placard/mach_alt_matrix_exceeds.csv
//! ```
//!
//! Exit codes: 0 all gates pass, 1 gate regression, 2 setup or usage failure.

mod constants;
mod model;
mod model_config;
mod utils_print;

use deep_causality_cfd::{IoAction, sweep};
use deep_causality_file::{NumericTable, read_table, write_table};
use deep_causality_num::ToPrimitive;
use std::process::exit;
use std::time::Instant;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

/// Abort with `exit(2)` on a setup or usage failure, naming what failed and the fix path.
fn fail(what: &str, detail: impl std::fmt::Display) -> ! {
    eprintln!("flight_envelope_placard setup failed ({what}): {detail}");
    exit(2)
}

fn main() {
    let clock = Instant::now();

    // ── The matrix: the recorded corridor by default, or a caller-supplied file.
    let matrix_path = model_config::matrix_path();
    let matrix = read_table::<FloatType>(&matrix_path)
        .run()
        .unwrap_or_else(|e| fail("reading the Mach-altitude matrix", e));
    let mach_col = matrix.column_index("mach").unwrap_or_else(|| {
        fail(
            "locating the 'mach' column",
            format!(
                "the matrix {} carries no 'mach' column; the header row must name it",
                matrix_path.display()
            ),
        )
    });
    let alt_col = matrix.column_index("alt").unwrap_or_else(|| {
        fail(
            "locating the 'alt' column",
            format!(
                "the matrix {} carries no 'alt' column; the header row must name it",
                matrix_path.display()
            ),
        )
    });
    let points: Vec<(FloatType, FloatType)> = matrix
        .rows()
        .iter()
        .map(|r| (r[mach_col], r[alt_col]))
        .collect();
    if points.is_empty() {
        fail(
            "reading the Mach-altitude matrix",
            format!("the matrix {} has no data rows", matrix_path.display()),
        );
    }
    utils_print::print_intro(&matrix_path);

    // ── The pointwise study: sweep over matrix rows, no march, no manifold. Printing happens
    // after the sweep (the side-effect rule of the combinator).
    let shock =
        model_config::shock_model().unwrap_or_else(|e| fail("building the fitted normal shock", e));
    let rows = sweep(&points, |&(mach, alt_km)| {
        model::placard_point(&shock, mach, alt_km)
    })
    .unwrap_or_else(|e| fail("computing the placard grid", e));
    utils_print::print_rows(&rows);

    // ── The placard table, through the group-1 writer. Display boundary: the working
    // precision downcasts to raw f64 here and only here.
    let raw_rows: Vec<Vec<f64>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|v| v.to_f64().expect("placard values are finite f64"))
                .collect()
        })
        .collect();
    let table = NumericTable::from_columns(
        [
            ("mach", "-"),
            ("alt", "km"),
            ("q", "kPa"),
            ("t0_post_shock", "K"),
            ("qdot", "W/cm2"),
        ],
        raw_rows,
    )
    .unwrap_or_else(|| fail("assembling the placard table", "ragged rows"));
    let out_path = model_config::table_path();
    write_table(&out_path, table)
        .run()
        .unwrap_or_else(|e| fail("writing the placard table", e));

    let all_pass = utils_print::report(
        &rows,
        matrix.rows().len(),
        &out_path,
        clock.elapsed().as_secs_f64(),
    );
    if !all_pass {
        exit(1);
    }
}
