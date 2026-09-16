/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the TTFields optimisation.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{INVASION_BIAS, Report, TUMOR_RADIUS_CM, TumorVolume};
use deep_causality_num::lower;

pub fn print_header() {
    println!("=== Glioblastoma TTFields: aiming the transducer arrays ===\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

pub fn print_tumor(tumor: &TumorVolume) {
    println!("Tumour");
    println!("  voxels            {}", tumor.voxels.len());
    println!("  extent            {TUMOR_RADIUS_CM:.1} cm");
    println!("  invasion bias     {INVASION_BIAS:.1} toward +z\n");
}

pub fn print_trace(report: &Report) {
    println!("Gradient ascent, following the exact gradient from the tangent functor");
    println!("  step   efficacy      θ        φ         ∇θ         ∇φ");

    for entry in &report.trace {
        println!(
            "  {:>4}   {:>8.4}   {:>6.3}   {:>6.3}   {:>+8.3}   {:>+8.3}",
            entry.step,
            lower(entry.score),
            lower(entry.theta),
            lower(entry.phi),
            lower(entry.gradient[0]),
            lower(entry.gradient[1])
        );
    }
    println!();
}

pub fn print_outcome(report: &Report) {
    let gain = lower(report.final_score) - lower(report.initial_score);
    println!("Result");
    println!(
        "  orientation       θ = {:.3} rad, φ = {:.3} rad",
        lower(report.theta),
        lower(report.phi)
    );
    println!(
        "  efficacy          {:.4} -> {:.4}   (+{gain:.4} over {} steps)",
        lower(report.initial_score),
        lower(report.final_score),
        report.trace.len()
    );
    println!();
    println!(
        "The ascent read {} exact gradients, one per step.",
        report.trace.len()
    );
    println!("A difference quotient would have cost two extra objective sweeps per step.");
}
