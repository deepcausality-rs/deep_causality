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
//! The row the table exists for is **navigation precision versus weather**. Two real mechanisms
//! couple them:
//!
//! * The atmosphere sets the ionization, the ionization sets the blackout window, and the
//!   window sets how long the dead-reckoning drift integrates. Weather moves the window.
//! * The accelerometer bias departs from its calibration point with temperature (a labeled
//!   tactical-grade thermal coefficient), while the navigation filter keeps its standard-day
//!   priors in every world. On a cold enough day the INS does not behave the way the filter
//!   assumes; the table measures by how much.
//!
//! Each dispersion world carries the `!!ContextAlternation!!` audit marker naming exactly what
//! it is a counterfactual of, so every row of the table has a provenance trail back to the
//! baseline. The run self-verifies and exits nonzero on regression (`exit(1)`) or setup failure
//! (`exit(2)`).
//!
//! ```bash
//! cargo run --release -p avionics_examples --example plasma_blackout_weather
//! ```

mod constants;
mod model;

use avionics_examples::blackout::{support, world};
use deep_causality_cfd::CfdFlow;
use deep_causality_core::AlternatableContext;
use deep_causality_par::scoped_map;
use std::process::exit;
use std::time::Instant;

/// The working precision, shared with the corridor (see `avionics_examples::blackout`).
pub type FloatType = avionics_examples::blackout::FloatType;

fn main() {
    let clock = Instant::now();
    println!("=== Plasma-blackout weather-dispersion table: six counterfactual atmospheres ===\n");
    println!(
        "Baseline: the corridor's validated descent | {} coupled steps per world, {} s of flight each",
        constants::STEPS,
        constants::STEPS as f64 * avionics_examples::blackout::constants::DT_FLIGHT,
    );
    println!(
        "IMU thermal model: bias departure 1 + {:.3}/K away from the calibration point; filter priors stay standard-day\n",
        constants::IMU_THERMAL_COEFF_PER_K,
    );

    // One world per condition. The first is the baseline; the five dispersions are alternated
    // from it, so each carries the audit marker naming its counterfactual origin.
    let worlds: Vec<_> = constants::WEATHER
        .iter()
        .map(|&(name, d_temp, rho_scale)| {
            model::weather_world(name, d_temp, rho_scale).unwrap_or_else(|e| support::stop(&e))
        })
        .collect();

    // Fly all six concurrently: the scoped fan-out gives the whole table one descent of
    // wall-clock. Each world gets its own coupling because the IMU it flies is part of the
    // condition (the thermal bias departure), and each run is data-independent.
    let indices: Vec<usize> = (0..constants::WEATHER.len()).collect();
    let pauses: Vec<_> = scoped_map(&indices, |&i| {
        let (_, d_temp, _) = constants::WEATHER[i];
        let run = CfdFlow::compressible_march(&worlds[0]);
        let run = if i == 0 {
            run
        } else {
            run.alternate_context(&worlds[i])
        };
        run.run_until(
            world::corridor_coupling(model::bias_departure(d_temp)),
            world::initial_field(),
            support::trigger(),
            support::ft(0.0),
            |_, _| false,
        )
    })
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .unwrap_or_else(|e| support::stop(&e));

    let rows: Vec<model::WorldRow> = constants::WEATHER
        .iter()
        .zip(&pauses)
        .map(|(&(name, d_temp, rho_scale), pause)| model::world_row(name, d_temp, rho_scale, pause))
        .collect();

    // ── The dispersion table (the digital-twin deliverable).
    println!(
        "  world          dT K   rho    IMU dep  onset s  exit s  dwell s   peak n_e     peak q      drift(dark) m  terminal m"
    );
    for r in &rows {
        println!(
            "  {:<13} {:>5.0}  {:>5.2}  {:>7.2}  {:>7.1}  {:>6.1}  {:>7.1}  {:>10.3e}  {:>10.3e}  {:>13.4}  {:>10.4}",
            r.name,
            r.d_temp,
            r.rho_scale,
            r.bias_departure,
            r.onset_s,
            r.exit_s,
            r.dwell_s,
            r.ne_max,
            r.q_max,
            r.drift_denied_max_m,
            r.terminal_err_m,
        );
    }
    println!();

    // ── The gates.
    println!("--- Weather-table validation gates ---");
    let mut all = true;
    let mut gate = |label: &str, pass: bool, detail: String| {
        println!(
            "  [{}] {label}: {detail}",
            if pass { "PASS" } else { "FAIL" }
        );
        all &= pass;
    };

    gate(
        "(0) table integrity",
        rows.iter().all(|r| !r.errored),
        "all six worlds completed without a captured step error".into(),
    );

    gate(
        "(1) counterfactual audit trail",
        rows.iter().skip(1).all(|r| r.has_alternation_marker),
        "every dispersion world carries the !!ContextAlternation!! marker naming its baseline"
            .into(),
    );

    let dt = avionics_examples::blackout::constants::DT_FLIGHT;
    let horizon_s = support::ft((constants::STEPS - 1) as f64 * dt);
    gate(
        "(2) flow-resolved windows in every weather",
        rows.iter().all(|r| {
            r.onset_s > support::ft(0.0)
                && r.dwell_s > support::ft(0.0)
                && r.exit_s >= r.onset_s
                && r.exit_s < horizon_s
        }),
        "each world found onset, a nonzero dwell, and a recovered link before the horizon".into(),
    );

    let onset_hi = rows
        .iter()
        .map(|r| r.onset_s)
        .fold(support::ft(f64::MIN), |a, x| if x > a { x } else { a });
    let onset_lo = rows
        .iter()
        .map(|r| r.onset_s)
        .fold(support::ft(f64::MAX), |a, x| if x < a { x } else { a });
    let dwell_hi = rows
        .iter()
        .map(|r| r.dwell_s)
        .fold(support::ft(f64::MIN), |a, x| if x > a { x } else { a });
    let dwell_lo = rows
        .iter()
        .map(|r| r.dwell_s)
        .fold(support::ft(f64::MAX), |a, x| if x < a { x } else { a });
    gate(
        "(3) weather moves the blackout window",
        onset_hi - onset_lo >= support::ft(constants::MIN_ONSET_SPREAD_S),
        format!(
            "onset spread {:.1} s across the table (gate requires {:.0} s); dwell spread {:.1} s",
            onset_hi - onset_lo,
            constants::MIN_ONSET_SPREAD_S,
            dwell_hi - dwell_lo,
        ),
    );

    let standard = &rows[0];
    let polar = rows
        .iter()
        .find(|r| r.name == "polar_winter")
        .expect("polar_winter row");
    gate(
        "(4) the INS does not behave as assumed in the cold",
        polar.drift_denied_max_m
            >= standard.drift_denied_max_m * support::ft(constants::COLD_DRIFT_FACTOR_MIN),
        format!(
            "polar-winter blackout drift {:.4} m vs standard-day {:.4} m ({:.2}x; gate requires \
             {:.1}x from the thermal bias departure and the widened window)",
            polar.drift_denied_max_m,
            standard.drift_denied_max_m,
            polar.drift_denied_max_m / standard.drift_denied_max_m,
            constants::COLD_DRIFT_FACTOR_MIN,
        ),
    );

    gate(
        "(5) every weather reacquires",
        rows.iter()
            .all(|r| r.terminal_err_m < support::ft(constants::REACQ_ERR_MAX_M)),
        format!(
            "terminal navigation error under {:.1} m in all six worlds",
            constants::REACQ_ERR_MAX_M,
        ),
    );

    let elapsed_s = clock.elapsed().as_secs_f64();
    gate(
        "(6) wall-clock budget",
        elapsed_s < constants::WALL_CLOCK_BUDGET_S,
        format!(
            "{elapsed_s:.1} s for the whole six-world table (budget {:.0} s)",
            constants::WALL_CLOCK_BUDGET_S,
        ),
    );

    println!();
    if all {
        println!("=== Weather-dispersion table complete. All gates passed. ===");
    } else {
        println!("=== Weather-table gate REGRESSION: see the FAIL lines above. ===");
        exit(1);
    }
}
