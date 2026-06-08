/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cascade-failure-specific display helpers. Shared printing and
//! collection plumbing come from `causal_counterfactual_examples::{print_utils, math_utils}`.

use crate::model_types::{FlowSolution, NetworkConfig, NetworkProcess};
use causal_counterfactual_examples::{math_utils, print_utils};
use deep_causality_core::EffectValue;

pub fn print_network_topology(cfg: &NetworkConfig) {
    println!(
        "Network: {} nodes, {} edges, source supply = {:.1}",
        cfg.n_nodes,
        cfg.edges.len(),
        cfg.source_supply
    );
    for e in &cfg.edges {
        println!(
            "  edge {:>2}: {} -> {}  capacity = {:.1}",
            e.id, e.from, e.to, e.capacity
        );
    }
    println!();
}

pub fn summary_line(label: &str, process: &NetworkProcess<FlowSolution>) {
    let (delivered, overloaded_count) = match &process.value {
        EffectValue::Value(s) => (s.delivered, s.overloaded.len()),
        _ => (f64::NAN, 0),
    };
    let cfg = process.context.as_ref().unwrap();
    println!(
        "  {label}: failed_edges={:>2}  delivered={:>5.2} / {:.2}  cascade_steps={}  unresolved_overloaded={}",
        process.state.failed_edges.len(),
        delivered,
        cfg.source_supply,
        process.state.cascade_step,
        overloaded_count,
    );
}

pub fn print_section(label: &str, process: &NetworkProcess<FlowSolution>) {
    print_utils::print_section_header(label);
    match &process.value {
        EffectValue::Value(s) => {
            println!(
                "  delivered: {:.2} / {:.2}  failed_edges: {:?}  cascade_steps: {}",
                s.delivered,
                process.context.as_ref().unwrap().source_supply,
                math_utils::sorted(&process.state.failed_edges),
                process.state.cascade_step,
            );
            if !s.overloaded.is_empty() {
                println!(
                    "  unresolved overloaded edges at termination: {:?}",
                    s.overloaded
                );
            }
            print_utils::print_trajectory("per-edge flows", &s.flows, |f| format!("{f:.2}"));
        }
        _ => println!("  (no value)"),
    }
    println!("  EffectLog (cascade timeline):");
    print_utils::print_effect_log_indented(&process.logs, "    ");
    print_utils::print_section_footer();
}
