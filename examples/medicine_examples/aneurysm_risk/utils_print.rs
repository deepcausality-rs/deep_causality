/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the aneurysm study.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone and the rest of the example stays in the working scalar.

use crate::FloatType;
use crate::model::{
    CYCLES_PER_EPOCH, DOME_RADIUS_M, FLOW_RATE_M3S, HEALTHY_RADIUS_M, LOW_SHEAR_THRESHOLD_PA,
    NECK_IN, NECK_OUT, RUPTURE_THRESHOLD, SEGMENT_LENGTH_M, radius_at,
};
use deep_causality_num::lower;
use deep_causality_topology::SimplicialManifold;

pub fn print_header() {
    println!("=== Aneurysm wall degeneration from the shear profile ===\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

pub fn print_geometry(nodes: usize) {
    println!("Vessel segment");
    println!(
        "  length              {:.0} mm",
        lower(SEGMENT_LENGTH_M) * 1000.0
    );
    println!("  centreline nodes    {nodes}");
    println!(
        "  healthy radius      {:.1} mm",
        lower(HEALTHY_RADIUS_M) * 1000.0
    );
    println!(
        "  dome radius         {:.1} mm",
        lower(DOME_RADIUS_M) * 1000.0
    );
    println!(
        "  flow rate           {:.1} mL/s\n",
        lower(FLOW_RATE_M3S) * 1.0e6
    );
}

pub fn print_profile(
    shear: &SimplicialManifold<FloatType, FloatType>,
    gradient: &SimplicialManifold<FloatType, FloatType>,
    nodes: usize,
) {
    let tau = shear.data().as_slice();
    let grad = gradient.data().as_slice();

    println!("Shear profile along the centreline");
    println!("  node   radius     shear      |dτ/dx|      state");
    println!("          (mm)      (Pa)       (Pa/m)");

    for node in [0usize, 10, NECK_IN, 17, 20, 23, NECK_OUT, 30, nodes - 1] {
        let state = if tau[node] < LOW_SHEAR_THRESHOLD_PA {
            "starved"
        } else {
            "healthy"
        };
        println!(
            "  {:>4}   {:>6.2}   {:>7.3}   {:>9.1}      {}",
            node,
            lower(radius_at(node)) * 1000.0,
            lower(tau[node]),
            lower(grad[node]),
            state
        );
    }
    println!();
}

pub fn print_risk_factors(dome_shear: FloatType, peak_gradient: FloatType) {
    println!("Risk factors");
    println!(
        "  lowest shear        {:.3} Pa   (threshold {:.1} Pa)",
        lower(dome_shear),
        lower(LOW_SHEAR_THRESHOLD_PA)
    );
    println!("  peak shear gradient {:.1} Pa/m\n", lower(peak_gradient));
}

pub fn print_history(history: &[FloatType]) {
    println!("Wall degeneration");
    println!("  epoch   cycles        index");
    for (i, value) in history.iter().enumerate() {
        let epoch = i + 1;
        let flag = if *value >= RUPTURE_THRESHOLD {
            "  <- rupture risk"
        } else {
            ""
        };
        println!(
            "  {:>5}   {:>10}   {:>6.3}{}",
            epoch,
            epoch as u64 * CYCLES_PER_EPOCH,
            lower(*value),
            flag
        );
    }
    println!();
}

pub fn print_verdict(history: &[FloatType]) {
    match history.iter().position(|v| *v >= RUPTURE_THRESHOLD) {
        Some(i) => {
            let epoch = i + 1;
            println!(
                "The index crosses {:.2} at epoch {epoch}, about {} cardiac cycles.",
                lower(RUPTURE_THRESHOLD),
                epoch as u64 * CYCLES_PER_EPOCH
            );
            println!("Sustained low shear in the dome is what carried it there.");
        }
        None => println!(
            "The index stays below {:.2} for the whole run.",
            lower(RUPTURE_THRESHOLD)
        ),
    }
}
