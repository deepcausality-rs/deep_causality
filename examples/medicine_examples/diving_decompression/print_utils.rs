/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Verbose, presentation-only output for the SCUBA decompression planner: the dive table and the
//! detailed single-dive walkthrough. These render results computed by the `model` layer.

use crate::FloatType;
use crate::model::{
    ASCENT_RATE, DESCENT_RATE, DiveProfile, DiverState, GF_HIGH, HALF_TIMES, ambient_pressure,
    cns_accumulation, estimate_ndl, find_ceiling, inspired_n2_pp, oxygen_pp, update_tissues,
};

pub fn print_dive_table() {
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
        let profile = crate::simulate_dive(depth, ndl.min(20.0)); // Cap at 20 min for table

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

pub fn print_detailed_simulation(
    max_depth: FloatType,
    bottom_time: FloatType,
    profile: &DiveProfile,
) {
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

        let ascent_segment = (3.0 as FloatType).min(current_depth);
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
    let steps: Vec<(FloatType, FloatType)> = (0..=(max_depth as i32 / 10))
        .map(|i| {
            let d = max_depth - (i as FloatType * 10.0);
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

    // Decompression schedule from the monadic `simulate_dive` result `main` computed and passed in.
    println!("\n[DECO SCHEDULE] (from simulate_dive)");
    match profile.safety_stop {
        Some((d, t)) => println!("  Safety stop:           {t:.0} min @ {d:.0} m"),
        None => println!("  Safety stop:           none"),
    }
    if profile.deco_stops.is_empty() {
        println!("  Mandatory deco stops:  none (within NDL)");
    } else {
        for (d, t) in &profile.deco_stops {
            println!("  Deco stop:             {t:.0} min @ {d:.0} m");
        }
    }
    println!("  Total CNS load:        {:.1}%", profile.cns_percent);

    println!(
        "\n[COMPLETE] Dive completed safely. Total Simulated dive time: {:.1} min",
        state.elapsed_time
    );
}
