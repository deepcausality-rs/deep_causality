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

use avionics_examples::blackout::{utils, world};
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
        "IMU thermal model: bias departure 1 + {:.3}/K away from the calibration point; filter priors stay standard-day",
        constants::IMU_THERMAL_COEFF_PER_K,
    );
    println!(
        "Monte Carlo: {} deterministic receiver-noise draws per condition ({} descents total)\n",
        constants::MC_DRAWS,
        constants::WEATHER.len() * constants::MC_DRAWS,
    );

    // One world per condition. The first is the baseline; the five dispersions are alternated
    // from it, so each carries the audit marker naming its counterfactual origin.
    let worlds: Vec<_> = constants::WEATHER
        .iter()
        .map(|&(name, d_temp, rho_scale)| {
            model::weather_world(name, d_temp, rho_scale).unwrap_or_else(|e| utils::stop(&e))
        })
        .collect();

    // Fly the whole campaign concurrently: six conditions times MC_DRAWS receiver-noise
    // realizations, one flat fan-out over (condition, draw). Each run gets its own coupling
    // because both the IMU it flies (the thermal bias departure) and the noise realization are
    // part of the case, and every run is data-independent. Draw 0 of each condition is the
    // reference realization the window columns come from.
    let cases: Vec<(usize, usize)> = (0..constants::WEATHER.len())
        .flat_map(|i| (0..constants::MC_DRAWS).map(move |draw| (i, draw)))
        .collect();
    let pauses: Vec<_> = scoped_map(&cases, |&(i, draw)| {
        let (_, d_temp, _) = constants::WEATHER[i];
        let run = CfdFlow::compressible_march(&worlds[0]);
        let run = if i == 0 {
            run
        } else {
            run.alternate_context(&worlds[i])
        };
        run.run_until(
            world::corridor_coupling(model::bias_departure(d_temp), draw),
            world::initial_field(),
            utils::trigger(),
            utils::ft(0.0),
            |_, _| false,
        )
    })
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .unwrap_or_else(|e| utils::stop(&e));

    // Group the flat results back into per-condition draw sets (cases are condition-major).
    let rows: Vec<model::WorldRow> = constants::WEATHER
        .iter()
        .enumerate()
        .map(|(i, &(name, d_temp, rho_scale))| {
            let draws = &pauses[i * constants::MC_DRAWS..(i + 1) * constants::MC_DRAWS];
            model::world_row(name, d_temp, rho_scale, draws)
        })
        .collect();

    // ── The dispersion table (the digital-twin deliverable): navigation cells carry error
    // bars over the receiver-noise draws.
    println!(
        "  world          dT K   rho    IMU dep  onset s  exit s  dwell s   peak n_e     peak q      drift(dark) m     terminal m"
    );
    for r in &rows {
        println!(
            "  {:<13} {:>5.0}  {:>5.2}  {:>7.2}  {:>7.1}  {:>6.1}  {:>7.1}  {:>10.3e}  {:>10.3e}  {:>7.2} +- {:>4.2}  {:>6.3} (max {:.3})",
            r.name,
            r.d_temp,
            r.rho_scale,
            r.bias_departure,
            r.onset_s,
            r.exit_s,
            r.dwell_s,
            r.ne_max,
            r.q_max,
            r.drift_mean_m,
            r.drift_sd_m,
            r.terminal_mean_m,
            r.terminal_max_m,
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
    let horizon_s = utils::ft((constants::STEPS - 1) as f64 * dt);
    gate(
        "(2) flow-resolved windows in every weather",
        rows.iter().all(|r| {
            r.onset_s > utils::ft(0.0)
                && r.dwell_s > utils::ft(0.0)
                && r.exit_s >= r.onset_s
                && r.exit_s < horizon_s
        }),
        "each world found onset, a nonzero dwell, and a recovered link before the horizon".into(),
    );

    let onset_hi = rows
        .iter()
        .map(|r| r.onset_s)
        .fold(utils::ft(f64::MIN), |a, x| if x > a { x } else { a });
    let onset_lo = rows
        .iter()
        .map(|r| r.onset_s)
        .fold(utils::ft(f64::MAX), |a, x| if x < a { x } else { a });
    let dwell_hi = rows
        .iter()
        .map(|r| r.dwell_s)
        .fold(utils::ft(f64::MIN), |a, x| if x > a { x } else { a });
    let dwell_lo = rows
        .iter()
        .map(|r| r.dwell_s)
        .fold(utils::ft(f64::MAX), |a, x| if x < a { x } else { a });
    gate(
        "(3) weather moves the blackout window",
        onset_hi - onset_lo >= utils::ft(constants::MIN_ONSET_SPREAD_S),
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
        polar.drift_mean_m >= standard.drift_mean_m * utils::ft(constants::COLD_DRIFT_FACTOR_MIN),
        format!(
            "polar-winter mean blackout drift {:.2} m vs standard-day {:.2} m ({:.2}x; gate \
             requires {:.1}x from the thermal bias departure and the widened window)",
            polar.drift_mean_m,
            standard.drift_mean_m,
            polar.drift_mean_m / standard.drift_mean_m,
            constants::COLD_DRIFT_FACTOR_MIN,
        ),
    );

    // The certification-grade form of gate (4): the cold effect must be resolved above the
    // receiver-noise scatter, not within it.
    let combined_sd = deep_causality_num::Real::sqrt(
        polar.drift_sd_m * polar.drift_sd_m + standard.drift_sd_m * standard.drift_sd_m,
    );
    let separation = polar.drift_mean_m - standard.drift_mean_m;
    gate(
        "(4b) the cold effect is statistically resolved",
        separation >= combined_sd * utils::ft(constants::DRIFT_SIGNIFICANCE_SIGMA),
        format!(
            "polar-standard separation {:.2} m vs combined sigma {:.2} m ({:.1} sigma; gate \
             requires {:.0})",
            separation,
            combined_sd,
            separation / combined_sd,
            constants::DRIFT_SIGNIFICANCE_SIGMA,
        ),
    );

    gate(
        "(5) every weather reacquires, in every draw",
        rows.iter()
            .all(|r| r.terminal_max_m < utils::ft(constants::REACQ_ERR_MAX_M)),
        format!(
            "worst-draw terminal navigation error under {:.1} m across all {} descents",
            constants::REACQ_ERR_MAX_M,
            constants::WEATHER.len() * constants::MC_DRAWS,
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
