/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The nozzle operating map: a back-pressure sweep over a converging-diverging duct
//!
//! Dropping the back pressure on a converging-diverging duct walks it through its operating
//! regimes: choked with a normal shock in the diverging section, then choked with a
//! supersonic exit. Where the shock sits and what thrust the nozzle produces at each point is
//! the operating map, and sizing one is routine work in propulsion and test.
//!
//! The study reads its schedule as the case axis, binds one duct case per back pressure, marches
//! each, reduces every report to one row, records the operating map, and gates the rows against
//! gas-dynamics closed forms — the whole thing one `CfdFlow::study` expression. Configuration
//! lives in `model_config`, domain logic and the gating sequence in `model`, printing in
//! `utils_print`, tuned values in `constants`.
//!
//! The whole computation runs in the working precision [`FloatType`] (the shared example alias);
//! `f64` appears only at the display boundary. Exit codes: 0 all gates pass, 1 gate regression,
//! 2 setup failure.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example nozzle_operating_map
//! ```

mod constants;
mod model;
mod model_config;
mod utils_print;

use deep_causality_cfd::CfdFlow;
use std::path::PathBuf;
use std::process::ExitCode;

/// The working precision of the whole study, shared with the other examples. Switch the alias
/// in `avionics_examples::shared` to re-run every study at another precision; `f64` appears
/// only at the display boundary.
pub type FloatType = avionics_examples::shared::FloatType;

fn main() -> ExitCode {
    let path = in_path();

    match CfdFlow::study("nozzle operating map")
        .read::<FloatType>(&path, "p_back_over_p0")
        .inspect(|ratios| utils_print::print_intro(ratios.len(), &path))
        .case(model_config::duct_case)
        .march()
        .reduce(model::map_row)
        .inspect(utils_print::print_rows)
        .record(out_path())
        .inspect(|_| utils_print::print_footer(&out_path()))
        .gates(model::nozzle_gates())
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
            eprintln!("nozzle operating map setup failed: {e}");
            ExitCode::from(2)
        }
    }
}

fn in_path() -> PathBuf {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("cfd/nozzle_operating_map/back_pressures.csv")
        })
}

fn out_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cfd/nozzle_operating_map/operating_map.csv")
}
