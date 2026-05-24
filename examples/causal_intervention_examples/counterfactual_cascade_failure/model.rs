/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Flow solver, resolve stage, and cascade driver.

use crate::model_types::{
    CASCADE_MAX_ITERATIONS, Edge, FloatType, FlowSolution, NetworkConfig, NetworkProcess,
    NetworkState,
};
use causal_intervention_examples::math_utils;
use deep_causality_core::{EffectLog, EffectValue, Intervenable};
use deep_causality_haft::LogAddEntry;
use std::collections::HashSet;

/// A pragmatic, deterministic flow distributor for the cascade demo.
///
/// At each non-sink node, inflow is divided across the outgoing edges in
/// proportion to their remaining capacity (capacity if alive, zero if
/// failed). This is a simplification of a real network-flow solver. It
/// gets the qualitative cascade behaviour right: when an edge fails, its
/// share of flow is redistributed to its neighbours, which then exceed
/// their own capacity in turn.
pub fn solve_flow(cfg: &NetworkConfig, failed: &HashSet<u32>) -> FlowSolution {
    let mut flows = vec![0.0 as FloatType; cfg.edges.len()];
    let mut node_inflow = vec![0.0 as FloatType; cfg.n_nodes as usize];
    node_inflow[cfg.source as usize] = cfg.source_supply;

    // Topological order for this toy network is just node-id order (DAG by
    // construction). For each node, distribute its inflow across alive
    // outgoing edges proportionally to capacity.
    for node in 0..cfg.n_nodes {
        if node == cfg.sink {
            continue;
        }
        let outgoing: Vec<&Edge> = cfg
            .edges
            .iter()
            .filter(|e| e.from == node && !failed.contains(&e.id))
            .collect();
        if outgoing.is_empty() {
            continue;
        }
        let total_capacity: FloatType = outgoing.iter().map(|e| e.capacity).sum();
        if total_capacity <= 0.0 {
            continue;
        }
        let inflow = node_inflow[node as usize];
        for e in outgoing {
            let share = inflow * (e.capacity / total_capacity);
            flows[e.id as usize] = share;
            node_inflow[e.to as usize] += share;
        }
    }

    let mut overloaded = Vec::new();
    for e in &cfg.edges {
        if failed.contains(&e.id) {
            continue;
        }
        // 1% tolerance so a flow exactly at capacity doesn't cascade.
        if flows[e.id as usize] > e.capacity * 1.01 {
            overloaded.push(e.id);
        }
    }

    FlowSolution {
        flows,
        overloaded,
        delivered: node_inflow[cfg.sink as usize],
    }
}

/// The repeatedly-invoked cascade stage. The value channel carries the set
/// of failed edge ids (as a `Vec<u32>`, because `PropagatingProcess`
/// requires `Default + Clone`); the stage produces a fresh `FlowSolution`.
pub fn resolve_stage(
    value: EffectValue<Vec<u32>>,
    mut state: NetworkState,
    ctx: Option<NetworkConfig>,
) -> NetworkProcess<FlowSolution> {
    let cfg = ctx.clone().expect("NetworkConfig required");
    let intervened_failures = value.into_value().unwrap_or_default();

    // Merge intervened failures into accumulated state.
    for id in intervened_failures {
        state.failed_edges.insert(id);
    }

    let solution = solve_flow(&cfg, &state.failed_edges);

    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "resolve@step{}: failed={:?} delivered={:.2}/{:.2} overloaded={:?}",
        state.cascade_step,
        math_utils::sorted(&state.failed_edges),
        solution.delivered,
        cfg.source_supply,
        solution.overloaded,
    ));

    NetworkProcess::<FlowSolution> {
        value: EffectValue::Value(solution),
        state,
        context: ctx,
        error: None,
        logs,
    }
}

/// Run a cascade starting from the given trigger pipe.
///
/// The driver is intentionally small. A short loop picks the next pipe
/// to fail from the previous solve's overloaded list, calls
/// `intervene(...)` on the chain, and re-binds the solve stage.
/// Everything substantive lives in `resolve_stage`. The loop only
/// decides what to intervene on next.
pub fn run_cascade(trigger_edge: u32, cfg: NetworkConfig) -> NetworkProcess<FlowSolution> {
    let mut process: NetworkProcess<Vec<u32>> = NetworkProcess::<Vec<u32>> {
        value: EffectValue::Value(vec![trigger_edge]),
        state: NetworkState::default(),
        context: Some(cfg),
        error: None,
        logs: EffectLog::new(),
    };

    let mut result: NetworkProcess<FlowSolution> = process.bind(resolve_stage);

    for _ in 0..CASCADE_MAX_ITERATIONS {
        let overloaded = match &result.value {
            EffectValue::Value(s) => s.overloaded.clone(),
            _ => break,
        };
        if overloaded.is_empty() {
            // Stable footprint reached.
            break;
        }

        // Pick the highest-overload pipe to fail next (deterministic tie-break by id).
        let next_failure = overloaded[0];
        result.state.cascade_step += 1;

        // Re-bind: feed the previous solve's `cascade_step` forward by
        // rebuilding a Vec<u32>-valued process from the existing state/ctx
        // and intervening with the next failure on it.
        process = NetworkProcess::<Vec<u32>> {
            value: EffectValue::Value(Vec::new()),
            state: result.state.clone(),
            context: result.context.clone(),
            error: result.error.clone(),
            logs: result.logs.clone(),
        }
        .intervene(vec![next_failure]);

        result = process.bind(resolve_stage);
    }

    result
}
