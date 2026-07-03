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
//! The study reads its schedule through the typed table reader, runs one
//! [`CfdFlow::duct_march`](deep_causality_cfd::CfdFlow) per back pressure through [`sweep`],
//! gates every row against gas-dynamics closed forms, and writes the map through the typed
//! table writer. Configuration lives in `model_config`, domain logic in `model`, printing and
//! gates in `utils_print`, tuned values in `constants`.
//!
//! The whole computation runs in the working precision [`FloatType`] (the shared example
//! alias); `f64` appears only at the display and table-writing boundary. Exit codes: 0 all
//! gates pass, 1 gate regression, 2 setup failure.
//!
//! ```bash
//! cargo run --release -p avionics_examples --example nozzle_operating_map
//! ```

mod constants;
mod model;
mod model_config;
mod utils_print;

use avionics_examples::shared::utils::ft;
use constants::NO_SHOCK_SENTINEL_M;
use deep_causality_cfd::{IoAction, fail, sweep};
use deep_causality_file::{NumericTable, read_table, write_table};
use model::MapRow;
use std::path::PathBuf;
use std::process::exit;
use std::time::Instant;

/// The working precision of the whole study, shared with the other examples. Switch the alias
/// in `avionics_examples::shared` to re-run every study at another precision; `f64` appears
/// only at the display and table-writing boundary.
pub type FloatType = avionics_examples::shared::FloatType;

fn main() {
    let clock = Instant::now();

    // The schedule: one column of p_back / p0 ratios, overridable for the wrong-usage demo.
    let schedule_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("cfd/nozzle_operating_map/back_pressures.csv")
        });
    let schedule = read_table::<FloatType>(&schedule_path)
        .run()
        .unwrap_or_else(|e| fail("back-pressure schedule", e));
    let ratios: Vec<FloatType> = schedule.rows().iter().map(|r| r[0]).collect();
    utils_print::print_intro(ratios.len(), &schedule_path);

    // One duct march per back pressure; concurrent under the parallel feature.
    let rows: Vec<MapRow> = sweep(&ratios, |&p_ratio| model::map_row(p_ratio))
        .unwrap_or_else(|e| fail("back-pressure sweep", e));
    utils_print::print_rows(&rows);

    // The analytic regime boundaries and per-row closed-form shock stations.
    let first_critical = model::subsonic_exit_pressure_ratio();
    let shock_at_exit = model::exit_shock_back_pressure_ratio();
    let analytic_shocks: Vec<Option<FloatType>> = rows
        .iter()
        .map(|row| {
            (row.p_ratio > shock_at_exit && row.p_ratio < first_critical)
                .then(|| model::analytic_shock_position(row.p_ratio))
        })
        .collect();

    // The operating-map table, written at the display boundary (FloatType -> f64 here).
    let table_rows: Vec<Vec<f64>> = rows
        .iter()
        .map(|r| {
            vec![
                r.p_ratio,
                r.mach_exit,
                r.shock_x.unwrap_or(ft(NO_SHOCK_SENTINEL_M)),
                r.cf,
            ]
        })
        .collect();
    let table = NumericTable::from_columns(
        [
            ("p_back_over_p0", "-"),
            ("mach_exit", "-"),
            ("shock_x", "m; -1 = none"),
            ("thrust_coefficient", "-"),
        ],
        table_rows,
    )
    .unwrap_or_else(|| fail("operating-map table", "ragged rows"));
    let out_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("cfd/nozzle_operating_map/operating_map.csv");
    write_table(&out_path, table)
        .run()
        .unwrap_or_else(|e| fail("operating-map write", e));
    utils_print::print_footer(
        &out_path,
        first_critical,
        shock_at_exit,
        clock.elapsed().as_secs_f64(),
    );

    let ok = utils_print::report(&utils_print::GateInputs {
        rows: &rows,
        scheduled: ratios.len(),
        first_critical,
        analytic_shocks: &analytic_shocks,
    });
    if !ok {
        exit(1);
    }
}
