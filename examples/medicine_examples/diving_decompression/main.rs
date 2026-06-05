/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCUBA Diving Decompression Planner
//!
//! Simulates nitrogen tissue loading and CNS oxygen toxicity for safe dive profiles.
//! Implements a simplified Bühlmann ZH-L16C decompression algorithm.
//!
//! ## Two DeepCausality abstractions, side by side
//! - **Monadic composition** — `simulate_dive` chains the descent → bottom → ascent phases with
//!   `CausalEffectPropagationProcess::bind`, threading the diver state through the dive.
//! - **The tangent functor (arrow)** — the Schreiner gas-loading rate `dp/dt` is obtained by
//!   `SchreinerLoading::derivative`, the differentiable counterpart of the tissue-loading curve.
//!
//! Code is organised across three files: `model` (types, constants, physics), `utils_print`
//! (verbose presentation), and this `main` (the workflow that wires both abstractions together).

mod model;
mod print_utils;

use deep_causality_calculus::DifferentiateExt;
use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_physics::Density;
use model::{
    ASCENT_RATE, DESCENT_RATE, DiveProfile, DiverState, GF_HIGH, GF_LOW, SchreinerLoading,
    cns_accumulation, find_ceiling, update_tissues,
};
use print_utils::{print_detailed_simulation, print_dive_table};

/// The working precision for the whole dive simulation. **This is the single alias to change**:
/// set it to `f32` for lower precision or `f64` for standard precision, or Float106 for high precision;
/// the entire model (constants, tissue tensions, ceilings, CNS clock) recomputes at that precision.
pub type FloatType = f64;

// Adjust the max diving depth and bottom time using these constants.
const MAX_DEPTH: FloatType = 30.0;
const BOTTOM_TIME: FloatType = 0.0;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║           SCUBA Decompression Planner (Bühlmann ZH-L16C)          ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝\n");

    println!("Algorithm: Bühlmann ZH-L16C with Gradient Factors");
    println!("Tissue Compartments: 16 (half-times: 5 - 635 minutes)");
    println!(
        "Gradient Factors: GF_low={:.0}%, GF_high={:.0}%",
        GF_LOW * 100.0,
        GF_HIGH * 100.0
    );
    println!("Descent Rate: {:.0} m/min", DESCENT_RATE);
    println!("Ascent Rate: {:.0} m/min", ASCENT_RATE);
    println!();

    // Print dive table
    print_dive_table();
    println!();

    // Monadic composition: run the dive as a `bind` chain, then hand the result to the printer.
    let profile = simulate_dive(MAX_DEPTH, BOTTOM_TIME);
    print_detailed_simulation(MAX_DEPTH, BOTTOM_TIME, &profile);
    println!();

    let p_initial: FloatType = 0.740_467; // (1 − P_H2O)·0.79 at the surface
    let p_inspired: FloatType = 3.110_467; // (1 + 30/10 − P_H2O)·0.79 at 30 m
    let half_time: FloatType = 5.0; // compartment 1 half-time (min)

    let loading = SchreinerLoading {
        p_initial,
        p_inspired,
        half_time,
    };

    let x: FloatType = 10.0;
    let ln_two: FloatType = 2.0f64.ln();

    // The tangent functor: the Schreiner gas-loading rate dp/dt calculated via `derivative`
    // over the closed-form p(t) using autodiff.
    let (p_t, dp_dt) = loading.value_and_derivative(x);

    let k = ln_two / loading.half_time;
    println!("=== Gas-Loading Rate (autodiff) ===\n");
    println!(
        "Compartment 1 (τ½={:.0} min) @ 30 m, t=10 min: p={p_t:.4} bar, \
         dp/dt={dp_dt:.5} bar/min  [analytic k·(p_insp−p)={:.5}]",
        loading.half_time,
        k * (loading.p_inspired - p_t)
    );

    Ok(())
}

// =============================================================================
// Dive Simulation (monadic composition)
// =============================================================================

/// Simulates a complete dive profile by chaining the descent, bottom, and ascent phases with
/// `CausalEffectPropagationProcess::bind`. Each phase reads the carried `DiverState`, advances the
/// tissue tensions and CNS clock, and returns the next state; the final state is summarised into a
/// [`DiveProfile`].
fn simulate_dive(max_depth: FloatType, bottom_time: FloatType) -> DiveProfile {
    let initial_state = DiverState::default();
    let two: FloatType = 2.0;

    // Phase 1: Descent
    let descent_time = max_depth / DESCENT_RATE;
    let avg_descent_depth = max_depth / two;

    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(()),
        initial_state,
        None::<Density<FloatType>>,
    )
    .bind(|_, state, _| {
        // Access the current state
        let new_tensions = update_tissues(&state.tissue_tensions, avg_descent_depth, descent_time);
        let cns = state.cns_percent + cns_accumulation(avg_descent_depth, descent_time);

        CausalEffectPropagationProcess::pure(DiverState {
            depth: max_depth,
            elapsed_time: descent_time,
            tissue_tensions: new_tensions,
            cns_percent: cns,
            phase: "At Depth".to_string(),
        })
    })
    // Phase 2: Bottom time
    .bind(|prev, _, _| {
        // Access the previous state
        let state = prev.into_value().unwrap();
        let new_tensions = update_tissues(&state.tissue_tensions, max_depth, bottom_time);
        let cns = state.cns_percent + cns_accumulation(max_depth, bottom_time);

        CausalEffectPropagationProcess::pure(DiverState {
            elapsed_time: state.elapsed_time + bottom_time,
            tissue_tensions: new_tensions,
            cns_percent: cns,
            phase: "Bottom Complete".to_string(),
            ..state
        })
    })
    // Phase 3: Ascent
    .bind(|prev, _, _| {
        let mut state = prev.into_value().unwrap();
        let mut current_depth = state.depth;
        let mut deco_stops: Vec<(FloatType, FloatType)> = Vec::new();

        // Ascent in 3m increments
        while current_depth > 0.0 {
            let (_, ceiling) = find_ceiling(&state.tissue_tensions, GF_HIGH);

            // Check if we need a deco stop
            if ceiling > current_depth - 3.0 && current_depth > 6.0 {
                // Stop at current depth rounded to 3m
                let stop_depth = (current_depth / 3.0).floor() * 3.0;
                let stop_time = 2.0; // Minimum 2 min stop

                state.tissue_tensions =
                    update_tissues(&state.tissue_tensions, stop_depth, stop_time);
                state.cns_percent += cns_accumulation(stop_depth, stop_time);
                state.elapsed_time += stop_time;
                deco_stops.push((stop_depth, stop_time));
            }

            // Ascend 3m
            let ascent_segment = current_depth.min(3.0);
            let ascent_time = ascent_segment / ASCENT_RATE;
            current_depth -= ascent_segment;

            let avg_depth = current_depth + ascent_segment / 2.0;
            state.tissue_tensions = update_tissues(&state.tissue_tensions, avg_depth, ascent_time);
            state.cns_percent += cns_accumulation(avg_depth, ascent_time);
            state.elapsed_time += ascent_time;
        }

        state.depth = 0.0;
        state.phase = "Surfaced".to_string();

        // Store deco stops in phase string for extraction
        let deco_str = deco_stops
            .iter()
            .map(|(d, t)| format!("{:.0}min@{:.0}m", t, d))
            .collect::<Vec<_>>()
            .join(",");
        state.phase = format!("Surfaced|{}", deco_str);

        CausalEffectPropagationProcess::pure(state)
    });

    // Extract results
    let final_state = match process.value() {
        EffectValue::Value(s) => s.clone(),
        _ => DiverState::default(),
    };

    // Parse deco stops from phase string
    let deco_stops: Vec<(FloatType, FloatType)> = if final_state.phase.contains('|') {
        let parts: Vec<&str> = final_state.phase.split('|').collect();
        if parts.len() > 1 && !parts[1].is_empty() {
            parts[1]
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        return None;
                    }
                    let parts: Vec<&str> = s.split('@').collect();
                    if parts.len() == 2 {
                        let time = parts[0].replace("min", "").parse::<FloatType>().ok()?;
                        let depth = parts[1].replace("m", "").parse::<FloatType>().ok()?;
                        Some((depth, time))
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Safety stop for depths >= 15m
    let safety_stop = if max_depth >= 15.0 {
        Some((5.0, 3.0))
    } else {
        None
    };

    DiveProfile {
        cns_percent: final_state.cns_percent,
        safety_stop,
        deco_stops,
    }
}
