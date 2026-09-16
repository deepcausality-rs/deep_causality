/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Presentation for the freeze-check scenarios.
//!
//! This is the display boundary: `lower` is called here and nowhere else, so `f64` appears in this
//! file alone.

use crate::FloatType;
use crate::constants::SHARED_LEG;
use deep_causality_num::lower;
use deep_causality_quantum::QuantumMarkovReport;

pub fn print_header() {
    println!("=== A quantum causal model, and the check that decides whether it is one ===\n");
    println!("Precision: {}", core::any::type_name::<FloatType>());
    println!("Graph:     two nodes, 0 -> 1, both factors on Hilbert leg {SHARED_LEG}");
    println!("Condition: factors sharing a leg must pairwise commute\n");
}

/// Which scenario is about to run.
pub fn print_scenario(number: usize, factors: &str, expectation: &str) {
    println!("[{number}] {factors}");
    println!("    expected: {expectation}");
}

/// A model that froze.
pub fn print_frozen(report: &QuantumMarkovReport<FloatType>, is_frozen: bool) {
    let margin = report
        .worst_margin()
        .map(|m| format!("{:.3e}", lower(m)))
        .unwrap_or_else(|| "no pair tested".to_string());

    println!("    froze cleanly");
    println!("      pairs tested       {}", report.tested_pairs());
    println!("      worst margin       {margin}");
    println!("      is_frozen()        {is_frozen}");
    println!();
    println!("    The margin is how far the worst pair sat from commuting. Two diagonal");
    println!("    operators commute exactly, so it is zero rather than merely small.");
}

/// A model whose freeze aborted.
pub fn print_abort(node_j: usize, node_k: usize, detail: &str, is_frozen: bool) {
    println!("    freeze aborted");
    println!("      offending pair     nodes {node_j} and {node_k}");
    println!("      reason             {detail}");
    println!("      is_frozen()        {is_frozen}");
    println!();
    println!("    The graph rolled back to dynamic. A model that cannot be frozen is never");
    println!("    left half-frozen, so the failure costs the caller a rebuild and never a");
    println!("    result computed on a structure that does not hold.");
    println!();
    println!("    The error names the pair. A check that reported only that something failed");
    println!("    would leave a real model of any size with nowhere to start looking.");
}
