/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`StatefulMonadicCausableGraphReasoning`] over `CausaloidGraph`.
//!
//! Coverage note: a few arms guard against a `get_causaloid` / `outbound_edges` failure *inside*
//! the BFS / path walk, where the index originates from the graph's own frozen adjacency. On a
//! frozen, validated graph those calls cannot fail, so the arms are defensive and left uncovered
//! rather than forced through a corrupted graph.

use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;
use deep_causality_haft::LogAddEntry;

#[derive(Debug, Default, Clone, PartialEq)]
struct CounterState {
    count: u64,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct ConfigCtx {}

fn node_increment(
    obs: EffectValue<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let val = obs.into_value().unwrap_or(0);
    state.count += 1;
    let mut p = PropagatingProcess {
        value: EffectValue::Value(val),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs
        .add_entry(&format!("node_increment count={}", p.state.count));
    p
}

fn node_failing(
    _obs: EffectValue<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let mut p = PropagatingProcess {
        value: EffectValue::None,
        state,
        context: ctx,
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            "node_failing: deliberate".into(),
        ))),
        logs: EffectLog::new(),
    };
    p.logs.add_entry("node_failing: invoked");
    p
}

fn node_relay_to_two(
    _obs: EffectValue<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    // Emit a RelayTo pointing at index 2 with an inner stateless effect.
    let inner = PropagatingEffect::from_value(99u64);
    let mut p = PropagatingProcess {
        value: EffectValue::RelayTo(2, Box::new(inner)),
        state,
        context: ctx,
        error: None,
        logs: EffectLog::new(),
    };
    p.logs.add_entry("node_relay_to_two: emitted RelayTo(2)");
    p
}

fn build_three_node_path() -> CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);

    let n0: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    let n1: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(1, node_increment, ConfigCtx {}, "n1");
    let n2: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "n2");

    let i0 = g.add_root_causaloid(n0).expect("add root");
    let i1 = g.add_causaloid(n1).expect("add n1");
    let i2 = g.add_causaloid(n2).expect("add n2");

    g.add_edge(i0, i1).expect("edge 0->1");
    g.add_edge(i1, i2).expect("edge 1->2");

    g.freeze();
    g
}

fn build_initial() -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess {
        value: EffectValue::Value(7),
        state: CounterState::default(),
        context: Some(ConfigCtx {}),
        error: None,
        logs: EffectLog::new(),
    }
}

#[test]
fn evaluate_subgraph_from_cause_stateful_threads_state_across_three_nodes() {
    let g = build_three_node_path();
    let initial = build_initial();

    let out = g.evaluate_subgraph_from_cause_stateful(0, &initial);

    assert!(out.error.is_none(), "expected success, got {:?}", out.error);
    assert_eq!(
        out.state.count, 3,
        "state must reflect three counter increments across the BFS path"
    );

    let log_text = format!("{:?}", out.logs);
    assert!(log_text.contains("count=1"));
    assert!(log_text.contains("count=2"));
    assert!(log_text.contains("count=3"));
}

#[test]
fn evaluate_subgraph_stateful_short_circuits_on_node_error() {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);

    let n0: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    let n1: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(1, node_failing, ConfigCtx {}, "n1");
    let n2: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "n2");

    let i0 = g.add_root_causaloid(n0).expect("add root");
    let i1 = g.add_causaloid(n1).expect("add n1");
    let i2 = g.add_causaloid(n2).expect("add n2");
    g.add_edge(i0, i1).expect("edge 0->1");
    g.add_edge(i1, i2).expect("edge 1->2");
    g.freeze();

    let initial = build_initial();
    let out = g.evaluate_subgraph_from_cause_stateful(0, &initial);

    assert!(out.error.is_some());
    // State must reflect node 0's increment only — node 1 failed before
    // mutating state, node 2 must not execute.
    assert_eq!(out.state.count, 1);
    let log_text = format!("{:?}", out.logs);
    assert!(log_text.contains("node_failing"));
    assert!(
        !log_text.contains("count=2"),
        "node 2 must not have executed: {log_text}"
    );
}

#[test]
fn evaluate_subgraph_stateful_relayto_preserves_state() {
    // Layout: 0 -> 1 -> 2.  Node 0 emits RelayTo(2). Node 2 must observe
    // the state node 0 advanced; node 1 must not execute.
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);

    let n0: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(0, node_relay_to_two, ConfigCtx {}, "relayer");
    let n1: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(1, node_increment, ConfigCtx {}, "skipped");
    let n2: Causaloid<u64, u64, CounterState, ConfigCtx> =
        Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "target");

    let i0 = g.add_root_causaloid(n0).expect("root");
    let i1 = g.add_causaloid(n1).expect("n1");
    let i2 = g.add_causaloid(n2).expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.add_edge(i1, i2).expect("edge");
    g.freeze();

    let initial = build_initial();
    let out = g.evaluate_subgraph_from_cause_stateful(0, &initial);

    assert!(out.error.is_none(), "got {:?}", out.error);
    // Node 0 increments to 1; relays to node 2 which increments to 2.
    // Node 1 must not execute.
    assert_eq!(out.state.count, 2);
    let log_text = format!("{:?}", out.logs);
    assert!(
        log_text.contains("RelayTo(2)"),
        "expected relayer log entry: {log_text}"
    );
    assert!(
        log_text.contains("count=2"),
        "expected target node's increment log (count=2): {log_text}"
    );
    let _ = (i0, i1, i2);
}

#[test]
fn evaluate_single_cause_stateful_works() {
    let g = build_three_node_path();
    let initial = build_initial();

    let out = g.evaluate_single_cause_stateful(1, &initial);

    assert!(out.error.is_none());
    assert_eq!(out.state.count, 1);
}

#[test]
fn evaluate_shortest_path_between_causes_stateful_works() {
    let g = build_three_node_path();
    let initial = build_initial();

    let out = g.evaluate_shortest_path_between_causes_stateful(0, 2, &initial);

    assert!(out.error.is_none(), "got {:?}", out.error);
    assert_eq!(out.state.count, 3);
}

/// A `PropagatingProcess` that already carries an error (drives the short-circuit arms).
fn build_errored() -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess {
        value: EffectValue::None,
        state: CounterState { count: 5 },
        context: Some(ConfigCtx {}),
        error: Some(CausalityError::new(CausalityErrorEnum::Custom(
            "pre-existing".into(),
        ))),
        logs: EffectLog::new(),
    }
}

/// A frozen graph with two unconnected nodes (no edge), so there is no path between them.
fn build_two_unconnected() -> CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    let n1 = Causaloid::new_with_context(1, node_increment, ConfigCtx {}, "n1");
    g.add_root_causaloid(n0).expect("root");
    g.add_causaloid(n1).expect("n1");
    g.freeze();
    g
}

// --- short-circuit on an incoming error (all three methods) ---

#[test]
fn stateful_methods_short_circuit_on_incoming_error() {
    let g = build_three_node_path();
    let errored = build_errored();

    let single = g.evaluate_single_cause_stateful(0, &errored);
    assert!(single.error.is_some());
    assert_eq!(single.state.count, 5, "incoming state is preserved");

    let subgraph = g.evaluate_subgraph_from_cause_stateful(0, &errored);
    assert!(subgraph.error.is_some());

    let path = g.evaluate_shortest_path_between_causes_stateful(0, 2, &errored);
    assert!(path.error.is_some());
}

// --- not-frozen guard (all three methods) ---

#[test]
fn stateful_methods_require_a_frozen_graph() {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    g.add_root_causaloid(n0).expect("root");
    // Deliberately NOT frozen.
    let initial = build_initial();

    for out in [
        g.evaluate_single_cause_stateful(0, &initial),
        g.evaluate_subgraph_from_cause_stateful(0, &initial),
        g.evaluate_shortest_path_between_causes_stateful(0, 0, &initial),
    ] {
        let err = out.error.expect("must reject an unfrozen graph");
        assert!(format!("{err:?}").contains("frozen"));
    }
}

// --- index / containment guards ---

#[test]
fn evaluate_single_cause_stateful_rejects_a_missing_index() {
    let g = build_three_node_path();
    let initial = build_initial();
    let out = g.evaluate_single_cause_stateful(99, &initial);
    let err = out.error.expect("missing index errors");
    assert!(format!("{err:?}").contains("not found"));
}

#[test]
fn evaluate_subgraph_stateful_rejects_a_start_index_not_in_the_graph() {
    let g = build_three_node_path();
    let initial = build_initial();
    let out = g.evaluate_subgraph_from_cause_stateful(99, &initial);
    let err = out.error.expect("missing start errors");
    assert!(format!("{err:?}").contains("does not contain"));
}

#[test]
fn evaluate_subgraph_stateful_rejects_a_relay_to_a_missing_target() {
    // A two-node graph whose root relays to index 2, which does not exist.
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_relay_to_two, ConfigCtx {}, "relayer");
    let n1 = Causaloid::new_with_context(1, node_increment, ConfigCtx {}, "n1");
    let i0 = g.add_root_causaloid(n0).expect("root");
    let i1 = g.add_causaloid(n1).expect("n1");
    g.add_edge(i0, i1).expect("edge");
    g.freeze();

    let out = g.evaluate_subgraph_from_cause_stateful(0, &build_initial());
    let err = out.error.expect("relay to a missing target errors");
    assert!(format!("{err:?}").contains("RelayTo target"));
}

// --- shortest-path specific branches ---

#[test]
fn evaluate_shortest_path_stateful_start_equals_stop_runs_only_that_node() {
    let g = build_three_node_path();
    let out = g.evaluate_shortest_path_between_causes_stateful(1, 1, &build_initial());
    assert!(out.error.is_none(), "got {:?}", out.error);
    assert_eq!(out.state.count, 1, "exactly one node runs");
}

#[test]
fn evaluate_shortest_path_stateful_errors_when_no_path_exists() {
    let g = build_two_unconnected();
    let out = g.evaluate_shortest_path_between_causes_stateful(0, 1, &build_initial());
    assert!(out.error.is_some(), "no path between disconnected nodes");
}

#[test]
fn evaluate_shortest_path_stateful_short_circuits_on_a_failing_node() {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    let n1 = Causaloid::new_with_context(1, node_failing, ConfigCtx {}, "n1");
    let n2 = Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "n2");
    let i0 = g.add_root_causaloid(n0).expect("root");
    let i1 = g.add_causaloid(n1).expect("n1");
    let i2 = g.add_causaloid(n2).expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.add_edge(i1, i2).expect("edge");
    g.freeze();

    let out = g.evaluate_shortest_path_between_causes_stateful(0, 2, &build_initial());
    assert!(out.error.is_some(), "a failing node aborts the path walk");
    assert_eq!(out.state.count, 1, "only node 0 advanced state");
}

#[test]
fn evaluate_shortest_path_stateful_returns_on_a_relay() {
    // Path 0 -> 1 -> 2 where node 1 emits a RelayTo: the walk returns that process verbatim.
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "n0");
    let n1 = Causaloid::new_with_context(1, node_relay_to_two, ConfigCtx {}, "relayer");
    let n2 = Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "n2");
    let i0 = g.add_root_causaloid(n0).expect("root");
    let i1 = g.add_causaloid(n1).expect("n1");
    let i2 = g.add_causaloid(n2).expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.add_edge(i1, i2).expect("edge");
    g.freeze();

    let out = g.evaluate_shortest_path_between_causes_stateful(0, 2, &build_initial());
    assert!(out.error.is_none(), "got {:?}", out.error);
    assert!(
        matches!(out.value, EffectValue::RelayTo(2, _)),
        "the walk returns the relaying node's process"
    );
}
