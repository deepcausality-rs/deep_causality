/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the SCUBA decompression planner.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone. Every function renders a value the planner already computed, so the dive is
//! simulated once and shown twice.

use crate::FloatType;
use crate::model::{
    ASCENT_RATE, COMPARTMENTS, DESCENT_RATE, DiveProfile, DiveTableRow, GF_HIGH, HALF_TIME,
    HALF_TIMES, P_INSPIRED, SchreinerCurve, TEN, ZERO, ambient_pressure, half_time_of,
    inspired_n2_pp, oxygen_pp, saturation_percent,
};
use deep_causality_algebra::Real;
use deep_causality_num::lower;

/// The CNS oxygen clock reading above which a dive plan carries a warning, in percent.
const CNS_CAUTION_PERCENT: f64 = 50.0;
const CNS_WARNING_PERCENT: f64 = 80.0;

pub fn print_header() {
    println!("=== SCUBA Decompression Planner (Bühlmann ZH-L16C) ===\n");
    println!(
        "Precision:            {}",
        core::any::type_name::<FloatType>()
    );
    println!(
        "Tissue compartments:  {COMPARTMENTS}  (half-times {:.0} to {:.0} min)",
        lower(HALF_TIMES[0]),
        lower(HALF_TIMES[COMPARTMENTS - 1])
    );
    // A full planner interpolates from GF_low at depth to GF_high at the surface. This one holds
    // GF_high for the whole ascent, so that is the number the ceiling is computed against.
    println!(
        "Gradient factor:      {:.0}% held for the whole ascent",
        lower(GF_HIGH) * 100.0
    );
    println!("Descent rate:         {:.0} m/min", lower(DESCENT_RATE));
    println!("Ascent rate:          {:.0} m/min\n", lower(ASCENT_RATE));
}

/// The dive table: one planned dive per depth.
pub fn print_dive_table(rows: &[DiveTableRow]) {
    println!("Dive table");
    println!("  depth   ppO2    NDL    ascent   CNS    safety stop   deco stops");
    println!("   (m)    (bar)  (min)   (min)     (%)");

    let ascent_rate = lower(ASCENT_RATE);

    for row in rows {
        let depth = lower(row.depth_m);
        let ascent_minutes = depth / ascent_rate;

        let safety = match row.profile.safety_stop {
            Some(stop) => format!(
                "{:.0} min @ {:.0} m",
                lower(stop.minutes),
                lower(stop.depth_m)
            ),
            None => "none".to_string(),
        };

        let deco = if row.profile.deco_stops.is_empty() {
            "none".to_string()
        } else {
            row.profile
                .deco_stops
                .iter()
                .map(|s| format!("{:.0} min @ {:.0} m", lower(s.minutes), lower(s.depth_m)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        println!(
            "  {:>5.0}   {:>5.2}  {:>5.0}   {:>5.1}   {:>5.1}   {:<13} {}",
            depth,
            lower(oxygen_pp(row.depth_m)),
            lower(row.ndl_minutes),
            ascent_minutes,
            lower(row.profile.cns_percent),
            safety,
            deco
        );
    }
    println!();
    println!("  NDL is the time at depth that keeps the ascent free of mandatory stops.");
    println!("  The CNS oxygen clock starts accruing once ppO2 passes 1.0 bar.\n");
}

/// The headline dive, rendered from the profile the chain produced.
pub fn print_simulation(profile: &DiveProfile) {
    let max_depth = lower(profile.max_depth_m);
    let descent_rate = lower(DESCENT_RATE);
    let ascent_rate = lower(ASCENT_RATE);
    println!(
        "Planned dive: {:.0} m for {:.0} min",
        max_depth,
        lower(profile.bottom_minutes)
    );

    println!("\n  Phases");
    println!(
        "    descent   0 m  ->  {:>3.0} m   {:>5.1} min at {descent_rate:.0} m/min",
        max_depth,
        max_depth / descent_rate
    );
    println!(
        "    bottom    {:>3.0} m           {:>5.1} min, inspired ppN2 {:.2} bar",
        max_depth,
        lower(profile.bottom_minutes),
        lower(inspired_n2_pp(profile.max_depth_m))
    );
    println!(
        "    ascent    {:>3.0} m  ->   0 m   {:>5.1} min at {ascent_rate:.0} m/min",
        max_depth,
        max_depth / ascent_rate
    );

    println!("\n  Controlling compartment at the bottom");
    println!(
        "    #{:<2}  half-time {:>5.1} min   ceiling {:.1} m",
        profile.controlling + 1,
        lower(half_time_of(profile.controlling)),
        lower(profile.ceiling_m)
    );

    print_decompression(profile);
    print_tissues(profile);
    print_oxygen(profile);
    print_bubble_risk(profile);

    println!(
        "\n  Total run time: {:.1} min\n",
        lower(profile.total_minutes)
    );
}

fn print_decompression(profile: &DiveProfile) {
    println!("\n  Decompression schedule");
    if profile.deco_stops.is_empty() {
        println!("    mandatory stops   none, the dive stays within its NDL");
    } else {
        for stop in &profile.deco_stops {
            println!(
                "    mandatory stop    {:.0} min @ {:.0} m",
                lower(stop.minutes),
                lower(stop.depth_m)
            );
        }
    }
    match profile.safety_stop {
        Some(stop) => println!(
            "    safety stop       {:.0} min @ {:.0} m",
            lower(stop.minutes),
            lower(stop.depth_m)
        ),
        None => println!("    safety stop       none, the dive stays shallow"),
    }
}

fn print_tissues(profile: &DiveProfile) {
    let tensions = profile.final_tensions.as_slice();
    println!("\n  Tissue tensions at the surface");
    println!("    compartment   half-time    tension   saturation");

    // The fastest compartment, the one controlling the ascent, and the slowest. The controlling
    // compartment is often one of the other two, so the list is deduplicated before printing.
    let mut shown = vec![0usize, profile.controlling, COMPARTMENTS - 1];
    shown.sort_unstable();
    shown.dedup();

    for compartment in shown {
        let marker = if compartment == profile.controlling {
            "  <- controlling"
        } else {
            ""
        };
        println!(
            "      #{:<10} {:>6.1} min  {:>6.2} bar   {:>6.1}%{}",
            compartment + 1,
            lower(half_time_of(compartment)),
            lower(tensions[compartment]),
            lower(saturation_percent(
                tensions[compartment],
                profile.max_depth_m
            )),
            marker
        );
    }
}

fn print_oxygen(profile: &DiveProfile) {
    let pp_o2 = lower(oxygen_pp(profile.max_depth_m));
    let cns = lower(profile.cns_percent);

    println!("\n  Oxygen exposure");
    println!("    ppO2 at depth     {pp_o2:.2} bar");
    println!("    CNS clock         {cns:.1}%");

    let status = if cns < CNS_CAUTION_PERCENT {
        "within limits"
    } else if cns < CNS_WARNING_PERCENT {
        "approaching the 80% ceiling"
    } else {
        "past the 80% ceiling"
    };
    println!("    status            {status}");
}

/// Gas dissolved at depth expands on the way up in inverse proportion to the ambient pressure, and
/// the last ten metres carry the steepest relative drop of the whole ascent.
fn print_bubble_risk(profile: &DiveProfile) {
    println!("\n  Bubble expansion on ascent");
    println!("    band          pressure        expansion");

    let mut upper = profile.max_depth_m;

    while upper > ZERO {
        let lower_edge = if upper > TEN { upper - TEN } else { ZERO };
        let p_deep = ambient_pressure(upper);
        let p_shallow = ambient_pressure(lower_edge);
        println!(
            "    {:>3.0} m -> {:>3.0} m   {:.1} -> {:.1} bar   x{:.2}",
            lower(upper),
            lower(lower_edge),
            lower(p_deep),
            lower(p_shallow),
            lower(p_deep / p_shallow)
        );
        upper = lower_edge;
    }
}

/// The gas-loading rate from the tangent functor, beside the analytic rate it reproduces.
pub fn print_gas_loading_rate(at: &[FloatType; 4], tension: FloatType, rate: FloatType) {
    let k = SchreinerCurve.rate_constant(at[HALF_TIME]);
    let analytic = k * (at[P_INSPIRED] - tension);

    println!("Gas-loading rate from one evaluation over Dual");
    println!("  compartment half-time  {:>8.1} min", lower(at[HALF_TIME]));
    println!("  tension p(t)           {:>8.4} bar", lower(tension));
    println!("  rate dp/dt             {:>8.5} bar/min", lower(rate));
    println!("  analytic k(p_insp - p) {:>8.5} bar/min", lower(analytic));
    println!(
        "  agreement              {:>8.1e}",
        lower(Real::abs(rate - analytic))
    );
}
