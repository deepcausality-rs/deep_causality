/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the protein-folding simulation.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::model::{MEMORY_DEPTH, N_STATES, STATE_LABELS, native_fraction};
use deep_causality_num::lower;
use deep_causality_physics::Probability;

/// Width of the printed probability bar, in characters.
const BAR_WIDTH: f64 = 24.0;

/// The native-state probability above which the chain counts as folded.
const FOLDED_THRESHOLD: f64 = 0.5;
/// The native-state probability above which the chain counts as underway.
const UNDERWAY_THRESHOLD: f64 = 0.2;

pub fn print_header(steps: usize) {
    println!("=== Protein folding: the generalized master equation ===\n");
    println!("Precision:        {}", core::any::type_name::<FloatType>());
    println!(
        "States:           {N_STATES}  ({})",
        STATE_LABELS.join(", ")
    );
    println!("Memory depth:     {MEMORY_DEPTH} past distributions");
    println!("Steps:            {steps}\n");
}

pub fn print_distribution(step: usize, state: &[Probability<FloatType>]) {
    println!("  step {step:>2}");
    for (label, p) in STATE_LABELS.iter().zip(state) {
        let value = lower(p.value());
        let filled = (value * BAR_WIDTH).round().max(0.0) as usize;
        println!(
            "    {:<16} {:>6.2}%  {}",
            label,
            value * 100.0,
            "#".repeat(filled)
        );
    }
    println!();
}

pub fn print_summary(state: &[Probability<FloatType>]) {
    let native = lower(native_fraction(state));
    println!("Native-state probability: {:.4}", native);

    let verdict = if native > FOLDED_THRESHOLD {
        "the chain has reached its native state"
    } else if native > UNDERWAY_THRESHOLD {
        "folding is underway"
    } else {
        "folding is still in its early stages"
    };
    println!("{verdict}.");
}
