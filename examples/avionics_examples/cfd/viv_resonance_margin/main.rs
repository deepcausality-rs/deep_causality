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
//! * the airspeed schedule is read from `airspeeds.csv` through the typed table reader,
//! * [`sweep`] runs one wake march per airspeed (concurrently under the `parallel` feature),
//!   each on the validated isolated-cylinder configuration via `CfdFlow::march(..).run_owned()`,
//! * the shedding frequency per airspeed comes from [`strouhal_number`] on the wake probe's
//!   developed tail,
//! * the margin table is written through the typed table writer, and [`Gates`] verdicts the run.
//!
//! The whole computation runs in the working precision [`FloatType`]; `f64` appears only at the
//! display and table-writing boundary. The example exits nonzero on any gate regression.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example viv_resonance_margin
//! ```

mod constants;
mod model;
mod model_config;
mod utils_print;

use deep_causality_cfd::fail;
use deep_causality_cfd::{IoAction, sweep};
use deep_causality_file::{NumericTable, read_table, write_table};
use std::process::exit;
use std::time::Instant;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

fn main() {
    let clock = Instant::now();

    // ── The airspeed schedule, from the typed table next to this file.
    let schedule_path = model::example_file("airspeeds.csv");
    let schedule = read_table::<FloatType>(&schedule_path)
        .run()
        .unwrap_or_else(|e| fail("airspeed schedule", e));
    let col = schedule
        .column_index("airspeed")
        .unwrap_or_else(|| fail("airspeed schedule", "no 'airspeed' column in airspeeds.csv"));
    let airspeeds: Vec<FloatType> = schedule.rows().iter().map(|r| r[col]).collect();
    utils_print::print_intro(airspeeds.len(), &schedule_path);

    // ── The sweep: one validated cylinder-wake march per airspeed. The bodies run concurrently
    // under the `parallel` feature and return in schedule order either way; printing stays out
    // here, after the sweep, per the sweep's side-effect rule.
    let rows =
        sweep(&airspeeds, |&v| model::margin_row(v)).unwrap_or_else(|e| fail("airspeed sweep", e));
    utils_print::print_rows(&rows);

    // The display boundary: the table writer takes `f64`. For `FloatType = f64` the downcast is
    // the identity; for another alias it is the one place working precision leaves the program.
    let table_rows: Vec<Vec<f64>> = rows
        .iter()
        .map(|r| {
            vec![
                Into::<f64>::into(r.airspeed),
                Into::<f64>::into(r.reynolds),
                Into::<f64>::into(r.strouhal),
                Into::<f64>::into(r.f_shed_hz),
                Into::<f64>::into(r.margin),
            ]
        })
        .collect();
    let table = NumericTable::from_columns(
        [
            ("airspeed", "m/s"),
            ("reynolds", "-"),
            ("strouhal", "-"),
            ("shedding_frequency", "Hz"),
            ("margin", "-"),
        ],
        table_rows,
    )
    .unwrap_or_else(|| fail("margin table", "ragged rows"));
    let table_path = model::example_file("viv_resonance_margin.csv");
    write_table(&table_path, table)
        .run()
        .unwrap_or_else(|e| fail("margin table write", e));

    let ok = utils_print::report(
        &rows,
        airspeeds.len(),
        &table_path,
        clock.elapsed().as_secs_f64(),
    );
    if !ok {
        exit(1);
    }
}
