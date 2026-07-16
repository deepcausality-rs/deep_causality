/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The weather-dispersion table: counterfactual atmospheres for the blackout corridor
//!
//! When a new flight vehicle is built, the tables its flight computer flies are sourced from
//! somewhere: either a prototype flies into the ground until the envelope is mapped, or a
//! digital twin maps it in simulation. This example is the digital-twin table factory for the
//! plasma-blackout corridor: **six weather conditions, each a counterfactual world alternated
//! from one validated baseline description**, flown concurrently through the full coupled
//! physics (compressed compressible flow, evolved Park-2T ionization, flow-resolved GNSS
//! denial, IMU-driven navigation, the cybernetic envelope gate), and reduced to one dispersion
//! table.
//!
//! The whole campaign is one CfdFlow study expression:
//!
//! * `.baseline(standard_day).alternate(weather_world)` — the origin-form counterfactual: every
//!   condition is a whole atmosphere alternated from one declared baseline, each carrying the
//!   `!!ContextAlternation!!` audit marker naming what it is a counterfactual of.
//! * `.ensemble(MC_DRAWS).couple(..).march_for(STEPS, initial_field)` — each condition flies
//!   `MC_DRAWS` deterministic receiver-noise draws, concurrently over `(condition, draw)`, from a
//!   fresh initial field, through the per-draw coupling stack.
//! * `.reduce_ensemble(world_row)` — each condition's draw set collapses to one row (means,
//!   scatters, worst draws), then the table is recorded and gated.
//!
//! The row the table exists for is **navigation precision versus weather**. The DSL never exits or
//! prints: `main` maps the merged `Verdict` to an exit code (0 all gates pass, 1 regression, 2
//! setup failure).
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_weather
//! ```

mod constants;
mod model;
mod utils_print;

use avionics_examples::shared::{utils, world};
use deep_causality_cfd::{CfdFlow, PhysicsError, StudyError, StudyView, Verdict};
use std::process::ExitCode;
use std::time::Instant;

/// The working precision. Switch between `f64` and `deep_causality_num::Float106`
/// (106-bit double-double); the specification constants stay `f64` literals, which either type
/// represents exactly, and every derived number is computed in this type, so this alias is the
/// only line that changes.
pub type FloatType = f64;

/// The whole weather table is one study expression, resolved to a verdict, merged with the
/// caller's wall-clock gate (the study cannot see wall-clock — it times the whole program), then
/// mapped to an exit code.
fn main() -> ExitCode {
    // Where the dispersion table is recorded (the campaign's `record` seam).
    let table_path = model::get_table_path();

    // The audit-log base path: one file per (condition, draw) plus a main spawn/rejoin file land
    // under this directory (the campaign-level `save_log` verb). Run artifacts, git-ignored.
    let audit_dir = model::get_audit_dir();

    let outcome: Result<Verdict, StudyError> = (|| {
        let clock = Instant::now();
        utils_print::print_intro();
        std::fs::create_dir_all(&audit_dir).map_err(|e| {
            StudyError::in_stage(
                "save_log setup",
                PhysicsError::CalculationError(format!("audit dir: {e}")),
            )
        })?;

        let table = CfdFlow::study("weather-dispersion table")
            .save_log(audit_dir.join("weather.audit")) // one stepwise-flushed log per branch
            .cases(model::weather_cases())
            .baseline(model::standard_day) // the validated origin, built once
            .alternate(model::weather_world) // six counterfactual atmospheres, each marked
            .ensemble(constants::MC_DRAWS) // deterministic receiver-noise draws
            .couple(|case, draw| world::corridor_coupling(model::bias_departure(case.d_temp), draw))
            .march_for(constants::STEPS, world::initial_field) // fixed horizon, concurrent over (case, draw)
            .reduce_ensemble(model::world_row) // draw sets collapse to mean / scatter / worst
            .inspect(utils_print::print_rows)
            .record(&table_path)
            .gates(model::weather_gates())
            .verdict()?;

        // The wall-clock gate is the caller's: the study cannot see the wall clock, which times the
        // whole program. Merge it into the table verdict so the run still ends in one report.
        let elapsed = utils::ft(clock.elapsed().as_secs_f64());
        Ok(table.merge(model::runtime_gates().check(&StudyView::of(&[elapsed]))))
    })();

    match outcome {
        Ok(verdict) => {
            println!("\n{verdict}");
            if verdict.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("plasma-blackout weather table failed: {e}");
            ExitCode::from(2)
        }
    }
}
