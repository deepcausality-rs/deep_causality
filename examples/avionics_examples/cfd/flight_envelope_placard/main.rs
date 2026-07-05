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
//! This is the pointwise study path on purpose: no march runs and no manifold exists. The matrix
//! rows go through the grammar's `.prepare(shock).sweep(placard_point)` exactly like march cases
//! would go through `.case().march().reduce()` — the study shape is the same whether the body per
//! point is a solver run or a closed form.
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

use deep_causality_cfd::{CfdFlow, StudyError, Verdict};
use model::FlightPoint;
use std::process::ExitCode;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

/// The placard study, as one grammar expression: the Mach-altitude matrix is the case axis, the
/// fitted shock is the shared rig, and each point is a pointwise closed-form sweep (no march) to
/// one placard row, recorded and gated against the envelope placards.
fn placard_study() -> Result<Verdict, StudyError> {
    let matrix = model_config::matrix_path();
    CfdFlow::study("flight envelope placard")
        .matrix::<FlightPoint>(&matrix)
        .inspect(|_| utils_print::print_intro(&matrix))
        .prepare(model_config::shock_model)
        .sweep(model::placard_point)
        .inspect(utils_print::print_rows)
        .record(model_config::table_path())
        .inspect(|_| utils_print::print_footer(&model_config::table_path()))
        .gates(model::placard_gates())
        .verdict()
}

fn main() -> ExitCode {
    match placard_study() {
        Ok(verdict) => {
            print!("{verdict}");
            if verdict.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("flight envelope placard setup failed: {e}");
            ExitCode::from(2)
        }
    }
}
