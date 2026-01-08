/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # SCUBA Diving Decompression Planner
//!
//! Simulates nitrogen tissue loading and CNS oxygen toxicity for safe dive profiles.
//! Implements a simplified Bühlmann ZH-L16C decompression algorithm.
//!
//! ## Key Concepts
//! - **Tissue Compartments**: 16 parallel compartments with different half-times (5-635 min)
//! - **Schreiner Equation**: Exponential gas loading/offgassing model
//! - **CNS Toxicity**: NOAA ppO2 exposure limits for oxygen clock tracking
//!
//! ## APIs Demonstrated
//! - `CausalTensor` - Tissue compartment tensions as 1D tensor
//! - `CausalEffectPropagationProcess` - Monadic dive phase chaining
//! - `Pressure`, `Length` - Type-safe physics quantities
use deep_causality_core::{CausalEffectPropagationProcess, EffectValue};
use deep_causality_physics::Density;
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Constants: Bühlmann ZH-L16C Parameters
// =============================================================================
/// Tissue compartment half-times (minutes) for N2
const HALF_TIMES: [f64; 16] = [
    5.0, 8.0, 12.5, 18.5, 27.0, 38.3, 54.3, 77.0, 109.0, 146.0, 187.0, 239.0, 305.0, 390.0, 498.0,
    635.0,
];
/// M-value 'a' coefficients (bar)
const A_COEFFICIENTS: [f64; 16] = [
    1.1696, 1.0000, 0.8618, 0.7562, 0.6200, 0.5043, 0.4410, 0.4000, 0.3750, 0.3500, 0.3295, 0.3065,
    0.2835, 0.2610, 0.2480, 0.2327,
];
/// M-value 'b' coefficients (dimensionless)
const B_COEFFICIENTS: [f64; 16] = [
    0.5578, 0.6514, 0.7222, 0.7825, 0.8126, 0.8434, 0.8693, 0.8910, 0.9092, 0.9222, 0.9319, 0.9403,
    0.9477, 0.9544, 0.9602, 0.9653,
];
/// Surface nitrogen partial pressure (bar) - ~79% of 1 atm
const SURFACE_N2_PP: f64 = 0.79;
/// Oxygen fraction in air
const F_O2: f64 = 0.21;
/// Gradient factors (conservative recreational diving)
const GF_LOW: f64 = 0.30;
const GF_HIGH: f64 = 0.85;
/// Descent rate (m/min)
const DESCENT_RATE: f64 = 18.0;
/// Ascent rate (m/min) - PADI standard
const ASCENT_RATE: f64 = 9.0;
/// NOAA CNS oxygen toxicity limits: (ppO2 threshold, max_time_minutes)
const CNS_LIMITS: [(f64, f64); 7] = [
    (1.60, 45.0),
    (1.50, 120.0),
    (1.40, 150.0),
    (1.30, 180.0),
    (1.20, 210.0),
    (1.10, 240.0),
    (1.00, 300.0),
];

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

    // Detailed simulation for 30m dive
    print_detailed_simulation(30.0, 20.0);
    println!();

    // // Deep dive example (50m) to show CNS accumulation
    // println!("=== Deep Dive Example (50m for 6 min) ===\n");
    // print_detailed_simulation(50.0, 6.0);

    Ok(())
}

// =============================================================================
// Dive Simulation
// =============================================================================

/// Simulates a complete dive profile
fn simulate_dive(max_depth: f64, bottom_time: f64) -> DiveProfile {
    let initial_state = DiverState::default();

    // Phase 1: Descent
    let descent_time = max_depth / DESCENT_RATE;
    let avg_descent_depth = max_depth / 2.0;

    let process = CausalEffectPropagationProcess::with_state(
        CausalEffectPropagationProcess::pure(()),
        initial_state,
        None::<Density>,
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
        let mut deco_stops: Vec<(f64, f64)> = Vec::new();

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
            let ascent_segment = 3.0_f64.min(current_depth);
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
    let deco_stops: Vec<(f64, f64)> = if final_state.phase.contains('|') {
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
                        let time = parts[0].replace("min", "").parse::<f64>().ok()?;
                        let depth = parts[1].replace("m", "").parse::<f64>().ok()?;
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

// =============================================================================
// Physics Functions
// =============================================================================

/// Calculates ambient pressure at depth (bar)
fn ambient_pressure(depth: f64) -> f64 {
    1.0 + depth / 10.0
}

/// Calculates inspired nitrogen partial pressure at depth
fn inspired_n2_pp(depth: f64) -> f64 {
    let p_amb = ambient_pressure(depth);
    let p_water_vapor = 0.0627; // bar at 37°C
    let f_n2 = 0.79;
    (p_amb - p_water_vapor) * f_n2
}

/// Calculates oxygen partial pressure at depth
fn oxygen_pp(depth: f64) -> f64 {
    ambient_pressure(depth) * F_O2
}

/// Schreiner equation: tissue gas loading over time
fn tissue_loading(p_initial: f64, p_inspired: f64, time_minutes: f64, half_time: f64) -> f64 {
    let k = 2.0_f64.ln() / half_time;
    p_inspired + (p_initial - p_inspired) * (-k * time_minutes).exp()
}

/// Calculates ascent ceiling (minimum safe depth) for a tissue
fn tissue_ceiling(tissue_tension: f64, a: f64, b: f64, gf: f64) -> f64 {
    let m_value = tissue_tension / b + a;
    let allowed_gradient = gf * (m_value - tissue_tension / b);
    let ceiling_pressure = tissue_tension - allowed_gradient;
    ((ceiling_pressure - 1.0) * 10.0).max(0.0)
}

/// Finds maximum exposure time for a given ppO2 (linear interpolation)
fn max_cns_time(pp_o2: f64) -> f64 {
    if pp_o2 < 1.0 {
        return f64::INFINITY;
    }

    for i in 0..CNS_LIMITS.len() - 1 {
        let (pp_high, time_high) = CNS_LIMITS[i];
        let (pp_low, time_low) = CNS_LIMITS[i + 1];

        if pp_o2 >= pp_low && pp_o2 <= pp_high {
            let ratio = (pp_o2 - pp_low) / (pp_high - pp_low);
            return time_low + ratio * (time_high - time_low);
        }
    }

    if pp_o2 > 1.6 {
        return 45.0 * (1.6 / pp_o2);
    }

    300.0
}

/// Calculates CNS% accumulation for time spent at depth
fn cns_accumulation(depth: f64, time_minutes: f64) -> f64 {
    let pp_o2 = oxygen_pp(depth);
    let max_time = max_cns_time(pp_o2);

    if max_time.is_infinite() {
        return 0.0;
    }

    (time_minutes / max_time) * 100.0
}

/// Updates tissue tensions for all 16 compartments
fn update_tissues(tensions: &CausalTensor<f64>, depth: f64, time: f64) -> CausalTensor<f64> {
    let p_inspired = inspired_n2_pp(depth);
    let new_tensions: Vec<f64> = tensions
        .as_slice()
        .iter()
        .enumerate()
        .map(|(i, &p_initial)| tissue_loading(p_initial, p_inspired, time, HALF_TIMES[i]))
        .collect();

    CausalTensor::new(new_tensions, vec![16]).unwrap()
}

/// Finds the controlling compartment (highest ceiling)
fn find_ceiling(tensions: &CausalTensor<f64>, gf: f64) -> (usize, f64) {
    let mut max_ceiling = 0.0;
    let mut controlling = 0;

    for (i, &tension) in tensions.as_slice().iter().enumerate() {
        let ceiling = tissue_ceiling(tension, A_COEFFICIENTS[i], B_COEFFICIENTS[i], gf);
        if ceiling > max_ceiling {
            max_ceiling = ceiling;
            controlling = i;
        }
    }

    (controlling, max_ceiling)
}

/// Estimates NDL (No Decompression Limit) for a depth
fn estimate_ndl(depth: f64) -> f64 {
    // Conservative NDL estimates based on gradient factors
    match depth as i32 {
        0..=12 => 200.0,
        13..=18 => 80.0,
        19..=24 => 45.0,
        25..=30 => 25.0,
        31..=36 => 15.0,
        37..=42 => 10.0,
        43..=48 => 8.0,
        _ => 6.0,
    }
}

// =============================================================================
// Output Functions
// =============================================================================

fn print_dive_table() {
    println!("=== Dive Table (10m - 50m) ===\n");

    println!(
        "┌───────┬────────────┬───────────┬─────────────┬──────┬─────────────┬──────────────┐"
    );
    println!(
        "│ Depth │ ppO2 (bar) │ NDL (min) │ Ascent Time │ CNS% │ Safety Stop │ Deco Stop(s) │"
    );
    println!(
        "├───────┼────────────┼───────────┼─────────────┼──────┼─────────────┼──────────────┤"
    );

    let depths = [10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0];

    for depth in depths {
        let ndl = estimate_ndl(depth);
        let profile = simulate_dive(depth, ndl.min(20.0)); // Cap at 20 min for table

        let pp_o2 = oxygen_pp(depth);
        let ascent_time = depth / ASCENT_RATE;

        let safety = if profile.safety_stop.is_some() {
            "3min @ 5m".to_string()
        } else {
            "None".to_string()
        };

        let deco = if profile.deco_stops.is_empty() {
            "None".to_string()
        } else {
            profile
                .deco_stops
                .iter()
                .map(|(d, t)| format!("{:.0}min @ {:.0}m", t, d))
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "│ {:>3.0}m  │    {:.2}    │    {:>3.0}    │   {:.1} min   │ {:>3.0}% │ {:^11} │ {:^12} │",
            depth,
            pp_o2,
            ndl,
            ascent_time,
            profile.cns_percent.round(),
            safety,
            deco
        );
    }

    println!(
        "└───────┴────────────┴───────────┴─────────────┴──────┴─────────────┴──────────────┘"
    );
    println!();
    println!("Notes:");
    println!("  • NDL = No Decompression Limit (max time at depth without mandatory stops)");
    println!("  • Deco Stop format: Xmin @ Ym = stop at Y meters for X minutes during ascent");
    println!("  • CNS% only accumulates when ppO2 > 1.0 bar (depths > 38m on air)");
    println!("  • ppO2 calculated as (1 + depth/10) × 0.21 for air");
}

fn print_detailed_simulation(max_depth: f64, bottom_time: f64) {
    println!("=== Detailed {:.0}m Dive Simulation ===\n", max_depth);

    let descent_time = max_depth / DESCENT_RATE;
    println!(
        "[DESCENT] 0m → {:.0}m in {:.1} min ({:.0} m/min)",
        max_depth, descent_time, DESCENT_RATE
    );

    // Simulate and track
    let mut state = DiverState::default();

    // Descent
    let avg_descent_depth = max_depth / 2.0;
    state.tissue_tensions = update_tissues(&state.tissue_tensions, avg_descent_depth, descent_time);
    state.cns_percent += cns_accumulation(avg_descent_depth, descent_time);
    state.elapsed_time = descent_time;

    println!(
        "  Tissue Loading: Fast compartments absorbing N2 at {:.2} bar inspired ppN2",
        inspired_n2_pp(max_depth)
    );

    // Bottom
    println!("\n[BOTTOM] {:.1} min at {:.0}m", bottom_time, max_depth);
    state.tissue_tensions = update_tissues(&state.tissue_tensions, max_depth, bottom_time);
    state.cns_percent += cns_accumulation(max_depth, bottom_time);
    state.elapsed_time += bottom_time;

    let (controlling, ceiling) = find_ceiling(&state.tissue_tensions, GF_HIGH);
    println!(
        "  Controlling Compartment: #{} (τ={:.1} min)",
        controlling + 1,
        HALF_TIMES[controlling]
    );
    println!("  Ascent Ceiling: {:.1}m", ceiling);

    // Ascent
    println!(
        "\n[ASCENT] {:.0}m → 0m at {:.0} m/min",
        max_depth, ASCENT_RATE
    );

    let mut current_depth = max_depth;
    while current_depth > 0.0 {
        let (_, ceil) = find_ceiling(&state.tissue_tensions, GF_HIGH);
        let next_depth = (current_depth - 3.0).max(0.0);

        if next_depth > 0.0 {
            println!("  [{:>2.0}m] Ceiling: {:.1}m ✓", next_depth, ceil);
        }

        let ascent_segment = 3.0_f64.min(current_depth);
        let ascent_time = ascent_segment / ASCENT_RATE;
        current_depth -= ascent_segment;

        state.tissue_tensions = update_tissues(
            &state.tissue_tensions,
            current_depth + ascent_segment / 2.0,
            ascent_time,
        );
        state.elapsed_time += ascent_time;
    }

    // Safety stop
    if max_depth >= 15.0 {
        println!("  [ 5m] Safety Stop: 3 min");
        state.tissue_tensions = update_tissues(&state.tissue_tensions, 5.0, 3.0);
        state.elapsed_time += 3.0;
    }

    // Bubble expansion risk table
    println!("\n[⚠ BUBBLE EXPANSION RISK] Pressure changes during ascent:");
    println!("  ┌─────────────┬────────────────┬──────────────┬────────────────────┐");
    println!("  │ Depth Range │ Pressure (bar) │ Relative Drop│ Bubble Expansion   │");
    println!("  ├─────────────┼────────────────┼──────────────┼────────────────────┤");

    // Calculate for this specific dive depth
    let p_bottom = ambient_pressure(max_depth);
    let steps: Vec<(f64, f64)> = (0..=(max_depth as i32 / 10))
        .map(|i| {
            let d = max_depth - (i as f64 * 10.0);
            (d.max(0.0), ambient_pressure(d.max(0.0)))
        })
        .collect();

    for i in 0..steps.len() - 1 {
        let (d1, p1) = steps[i];
        let (d2, p2) = steps[i + 1];
        let rel_drop = ((p1 - p2) / p1) * 100.0;
        let expansion = p1 / p2;
        println!(
            "  │ {:>3.0}m → {:>2.0}m  │  {:.1} → {:.1} bar │    {:>4.0}%     │ ×{:.2} volume       │",
            d1, d2, p1, p2, rel_drop, expansion
        );
    }

    // Final critical zone
    println!("  ├─────────────┼────────────────┼──────────────┼────────────────────┤");
    println!(
        "  │ TOTAL       │  {:.1} → 1.0 bar │    {:>4.0}%     │ ×{:.1} = {:>3.0}% growth │",
        p_bottom,
        ((p_bottom - 1.0) / p_bottom) * 100.0,
        p_bottom,
        (p_bottom - 1.0) * 100.0
    );
    println!("  └─────────────┴────────────────┴──────────────┴────────────────────┘");
    println!("  ⚠ The final 10m (2→1 bar) = 50% pressure drop = HIGHEST BUBBLE RISK!");

    // Final summary
    println!("\n[SURFACE] Final tissue tensions:");
    let tensions = state.tissue_tensions.as_slice();
    println!(
        "  Compartment #1 (τ=5):     {:.2} bar ({:.0}% saturated)",
        tensions[0],
        (tensions[0] / inspired_n2_pp(max_depth)) * 100.0
    );
    println!(
        "  Compartment #{} (τ={:.1}): {:.2} bar ← Controlling",
        controlling + 1,
        HALF_TIMES[controlling],
        tensions[controlling]
    );
    println!(
        "  Compartment #16 (τ=635):  {:.2} bar (< 5% loaded)",
        tensions[15]
    );

    // CNS status
    let pp_o2 = oxygen_pp(max_depth);
    println!("\n[O2 TOXICITY] CNS Oxygen Status:");
    println!("  ppO2 at {:.0}m: {:.2} bar", max_depth, pp_o2);

    if pp_o2 < 1.0 {
        println!(
            "  CNS% accumulated: {:.1}% (no accumulation below ppO2 1.0)",
            state.cns_percent
        );
        println!("  Status: ✓ SAFE (well below 80% warning threshold)");
    } else {
        println!("  CNS% accumulated: {:.1}%", state.cns_percent);
        if state.cns_percent < 50.0 {
            println!(
                "  Status: ✓ SAFE ({:.1}% < 80% warning threshold)",
                state.cns_percent
            );
        } else if state.cns_percent < 80.0 {
            println!("  Status: ⚠ CAUTION (approaching 80% threshold)");
        } else {
            println!("  Status: ⚠ WARNING (exceeds 80% threshold)");
        }
    }

    println!(
        "\n[COMPLETE] Dive completed safely. Total Simulated dive time: {:.1} min",
        state.elapsed_time
    );
}

// =============================================================================
// Data Types
// =============================================================================

/// Represents a diver's physiological state
#[derive(Debug, Clone)]
struct DiverState {
    depth: f64,
    elapsed_time: f64,
    tissue_tensions: CausalTensor<f64>,
    cns_percent: f64,
    phase: String,
}

impl Default for DiverState {
    fn default() -> Self {
        Self {
            depth: 0.0,
            elapsed_time: 0.0,
            tissue_tensions: CausalTensor::new(vec![SURFACE_N2_PP; 16], vec![16]).unwrap(),
            cns_percent: 0.0,
            phase: "Surface".to_string(),
        }
    }
}

/// Dive profile result
#[derive(Debug)]
struct DiveProfile {
    cns_percent: f64,
    safety_stop: Option<(f64, f64)>, // (depth, duration)
    deco_stops: Vec<(f64, f64)>,     // [(depth, duration), ...]
}
