/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the virtual epilepsy surgery planner.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{
    BASE_FREQUENCY, COUPLING_STRENGTH, FREQUENCY_SPREAD, SEIZURE_THRESHOLD, SIMULATION_STEPS,
    TIME_STEP_S,
};
use deep_causality_num::lower;

pub fn print_header(regions: usize) {
    println!("=== Virtual Epilepsy Surgery Planning ===\n");
    println!(
        "Precision:           {}",
        core::any::type_name::<FloatType>()
    );
    println!("Brain regions:       {regions}  (region 0 is the hub, wired to every other)");
    println!("Coupling K:          {COUPLING_STRENGTH:.1} rad/s");
    println!("Natural frequency:   {BASE_FREQUENCY:.1} rad/s, spread {FREQUENCY_SPREAD:.1}");
    println!(
        "Simulated time:      {:.0} s  ({SIMULATION_STEPS} steps of {TIME_STEP_S} s)",
        SIMULATION_STEPS as f64 * TIME_STEP_S
    );
    println!("Seizure threshold:   synchronisation above {SEIZURE_THRESHOLD:.2}\n");
}

pub fn print_baseline(sync: FloatType) {
    let value = lower(sync);
    println!("Baseline, before surgery");
    println!("  synchronisation    {value:.4}");
    if value > SEIZURE_THRESHOLD {
        println!("  status             SEIZURE, the network is locked in step\n");
        println!("Virtual resection: disconnect one region, then re-simulate.");
        println!("  region   synchronisation   outcome");
    } else {
        println!("  status             within normal limits\n");
    }
}

pub fn print_resection_row(region: usize, sync: FloatType, seizing: bool) {
    let outcome = if seizing {
        "seizure persists"
    } else {
        "CURATIVE"
    };
    println!("  {:>6}   {:>15.4}   {}", region, lower(sync), outcome);
}

pub fn print_verdict(curative: &[(usize, FloatType)]) {
    println!();
    match curative {
        [] => println!("No single resection stops the seizure in this connectome."),
        [(region, sync)] => {
            println!(
                "Resecting region {region} drops synchronisation to {:.4}, well under the {SEIZURE_THRESHOLD:.2} threshold.",
                lower(*sync)
            );
            println!(
                "It is the one curative target here, and it is the hub: the region wired to every other."
            );
        }
        many => {
            println!("{} resections stop the seizure:", many.len());
            for (region, sync) in many {
                println!(
                    "  region {region} leaves synchronisation at {:.4}",
                    lower(*sync)
                );
            }
        }
    }
}
