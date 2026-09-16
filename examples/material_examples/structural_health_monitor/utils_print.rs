/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the hull monitor.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{
    BONDS, DEFAULT_MODULUS_GPA, ENHANCED_MODULUS_GPA, Hull, IMPACT_PLATE, N_PLATES,
    WARNING_THRESHOLD_MPA, YIELD_STRENGTH_MPA, breached_plates, strain,
};
use deep_causality_num::lower;

pub fn print_header() {
    println!("=== Decentralised structural health monitoring ===\n");
    println!(
        "Precision:          {}",
        core::any::type_name::<FloatType>()
    );
    println!("Yield strength:     {:.0} MPa", lower(YIELD_STRENGTH_MPA));
    println!(
        "Warning threshold:  {:.0} MPa\n",
        lower(WARNING_THRESHOLD_MPA)
    );
}

/// The hull as built: how many plates, how they are bonded, and what each plate carries.
pub fn print_hull(hull: &Hull) {
    println!("Hull section");
    println!("  plates              {N_PLATES}");
    println!("  structural bonds    {}", BONDS.len());

    let bonds: Vec<String> = BONDS.iter().map(|(a, b)| format!("{a}-{b}")).collect();
    println!("  topology            {}", bonds.join(", "));

    let stress = hull.data().as_slice();
    let nominal: Vec<String> = stress.iter().map(|&s| format!("{:.0}", lower(s))).collect();
    println!("  nominal load        {} MPa\n", nominal.join(", "));
}

/// The sensor reading, and what the intervention substituted for it.
pub fn print_reading(observed: FloatType, intervened: FloatType) {
    println!("Micrometeoroid strike on plate {IMPACT_PLATE}");
    println!("  sensor reading      {:.0} MPa", lower(observed));

    if lower(observed) > lower(WARNING_THRESHOLD_MPA) {
        println!("  status              past the warning threshold, and past yield");
        println!(
            "  intervention        do(stress := {:.0} MPa), recorded",
            lower(intervened)
        );
    } else {
        println!("  status              within limits, no intervention");
    }

    println!("\n  Strain at the reading, by Hooke's law");
    for (label, modulus) in [
        ("as manufactured", DEFAULT_MODULUS_GPA),
        ("reinforcement engaged", ENHANCED_MODULUS_GPA),
    ] {
        println!(
            "    {:<22} E = {:>4.0} GPa   strain {:.3e}",
            label,
            lower(modulus),
            lower(strain(observed, modulus))
        );
    }
    println!("  Raising the modulus leaves the same stress deforming the plate less.\n");
}

/// One step of the cascade: what every plate now carries, and which are over yield.
pub fn print_cascade_step(step: usize, hull: &Hull, yielding: &[usize]) {
    let stress = hull.data().as_slice();
    let row: Vec<String> = stress
        .iter()
        .map(|&s| format!("{:>6.1}", lower(s)))
        .collect();

    let marker = if step == 0 {
        "  impact ".to_string()
    } else {
        format!("  step {step} ")
    };

    let note = if yielding.is_empty() {
        "no plate over yield".to_string()
    } else {
        format!("over yield: {yielding:?}")
    };

    println!("{marker}  {}   {note}", row.join(" "));
}

pub fn print_verdict(without: &Hull, with: &Hull) {
    let breached_without = breached_plates(without).len();
    let breached_with = breached_plates(with).len();

    println!("Counterfactual");
    println!(
        "  without intervention   {breached_without} of {N_PLATES} plates breached{}",
        if breached_without == N_PLATES {
            ", the section is lost"
        } else {
            ""
        }
    );
    println!("  with intervention      {breached_with} of {N_PLATES} plates breached");
    println!();
    println!("Both runs used the same hull and the same redistribution rule. The only difference");
    println!("is the value the struck plate carried into the cascade, which is what the recorded");
    println!("intervention substituted.");
}
