/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The vortex-shedding resonance margin: an airspeed sweep over a circular member
//!
//! A circular cross-section in a stream sheds a von Karman street, and the street pushes on the
//! structure at the shedding frequency `f = St * V / D`. When that frequency approaches a
//! structural natural mode the member locks in and shakes itself apart; margin to the mode is a
//! standard placard check. This example runs that check as a computed study, not a handbook
//! lookup:
//!
//! * the airspeed schedule is read as the case axis (`airspeeds.csv`, column `airspeed`),
//! * `.case` binds one validated isolated-cylinder wake case per airspeed and `.march` marches
//!   each (concurrently under the `parallel` feature) on its own fresh grid,
//! * `.reduce` extracts the shedding frequency per airspeed from the wake probe's developed tail,
//! * `.record` writes the margin table, and the gating sequence verdicts the run.
//!
//! The whole thing is one `CfdFlow::study` expression. It runs in the working precision
//! [`FloatType`]; `f64` appears only at the display boundary. Exit codes: 0 all gates pass, 1
//! gate regression, 2 setup failure.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example viv_resonance_margin
//! ```

mod constants;
mod model;
mod model_config;
mod utils_print;

use deep_causality_cfd::CfdFlow;
use std::process::ExitCode;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

fn main() -> ExitCode {
    let schedule = model::example_file("airspeeds.csv");

    // The resonance-margin study, as one grammar expression: airspeeds in, one wake case per
    // airspeed, march, reduce to margin rows, record, gate, verdict.
    match CfdFlow::study("vortex-shedding resonance margin")
        .read::<FloatType>(&schedule, "airspeed")
        .inspect(|airspeeds| utils_print::print_intro(airspeeds.len(), &schedule))
        .case(model_config::wake_case)
        .march()
        .reduce(model::margin_row)
        .inspect(utils_print::print_rows)
        .record(model::example_file("viv_resonance_margin.csv"))
        .inspect(|_| utils_print::print_footer(&model::example_file("viv_resonance_margin.csv")))
        .gates(model::viv_gates())
        .verdict()
    {
        Ok(verdict) => {
            print!("{verdict}");
            if verdict.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("vortex-shedding resonance margin setup failed: {e}");
            ExitCode::from(2)
        }
    }
}
