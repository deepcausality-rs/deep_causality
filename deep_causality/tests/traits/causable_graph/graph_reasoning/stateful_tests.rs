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
    obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let val = obs.into_value().unwrap_or(0);
    state.count += 1;
    let mut logs = EffectLog::new();
    logs.add_entry(&format!("node_increment count={}", state.count));
    PropagatingProcess::new(Ok(CausalEffect::value(val)), state, ctx, logs)
}

fn node_failing(
    _obs: CausalEffect<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    let mut logs = EffectLog::new();
    logs.add_entry("node_failing: invoked");
    PropagatingProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::Custom(
            "node_failing: deliberate".into(),
        ))),
        state,
        ctx,
        logs,
    )
}

fn node_relay_to_two(
    _obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    // Emit a RelayTo pointing at index 2 with an inner stateless effect.

    let mut logs = EffectLog::new();
    logs.add_entry("node_relay_to_two: emitted RelayTo(2)");
    PropagatingProcess::new(
        Ok(CausalEffect::relay_to(2, CausalEffect::value(99u64))),
        state,
        ctx,
        logs,
    )
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
    PropagatingProcess::new(
        Ok(CausalEffect::value(7)),
        CounterState::default(),
        Some(ConfigCtx {}),
        EffectLog::new(),
    )
}

#[test]
fn evaluate_subgraph_from_cause_stateful_threads_state_across_three_nodes() {
    let g = build_three_node_path();
    let initial = build_initial();

    let out = g.evaluate_subgraph_from_cause_stateful(0, &initial);

    assert!(out.is_ok(), "expected success, got {:?}", out.error());
    assert_eq!(
        out.state().count,
        3,
        "state must reflect three counter increments across the BFS path"
    );

    let log_text = format!("{:?}", out.logs());
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

    assert!(out.is_err());
    assert!(out.value().is_none(), "an errored carrier holds no value");
    // State must reflect node 0's increment only — node 1 failed before
    // mutating state, node 2 must not execute.
    assert_eq!(out.state().count, 1);
    let log_text = format!("{:?}", out.logs());
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

    assert!(out.is_ok(), "got {:?}", out.error());
    // Node 0 increments to 1; relays to node 2 which increments to 2.
    // Node 1 must not execute.
    assert_eq!(out.state().count, 2);
    let log_text = format!("{:?}", out.logs());
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

    assert!(out.is_ok());
    assert_eq!(out.state().count, 1);
}

#[test]
fn evaluate_shortest_path_between_causes_stateful_works() {
    let g = build_three_node_path();
    let initial = build_initial();

    let out = g.evaluate_shortest_path_between_causes_stateful(0, 2, &initial);

    assert!(out.is_ok(), "got {:?}", out.error());
    assert_eq!(out.state().count, 3);
}

/// A `PropagatingProcess` that already carries an error (drives the short-circuit arms).
fn build_errored() -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess::new(
        Err(CausalityError::new(CausalityErrorEnum::Custom(
            "pre-existing".into(),
        ))),
        CounterState { count: 5 },
        Some(ConfigCtx {}),
        EffectLog::new(),
    )
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
    assert!(single.is_err());
    assert!(single.value().is_none());
    assert_eq!(single.state().count, 5, "incoming state is preserved");

    let subgraph = g.evaluate_subgraph_from_cause_stateful(0, &errored);
    assert!(subgraph.is_err());

    let path = g.evaluate_shortest_path_between_causes_stateful(0, 2, &errored);
    assert!(path.is_err());
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
        assert_eq!(out.error(), Some(&CausalityError::GraphNotFrozen()));
    }
}

// --- index / containment guards ---

#[test]
fn evaluate_single_cause_stateful_rejects_a_missing_index() {
    let g = build_three_node_path();
    let initial = build_initial();
    let out = g.evaluate_single_cause_stateful(99, &initial);
    assert_eq!(out.error(), Some(&CausalityError::CausaloidNotFound(99)));
}

#[test]
fn evaluate_subgraph_stateful_rejects_a_start_index_not_in_the_graph() {
    let g = build_three_node_path();
    let initial = build_initial();
    let out = g.evaluate_subgraph_from_cause_stateful(99, &initial);
    assert_eq!(out.error(), Some(&CausalityError::CausaloidNotFound(99)));
}

fn node_relay_to_one(
    _obs: CausalEffect<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess::new(
        Ok(CausalEffect::relay_to(1, CausalEffect::value(1u64))),
        state,
        ctx,
        EffectLog::new(),
    )
}

fn node_relay_to_zero(
    _obs: CausalEffect<u64>,
    state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess::new(
        Ok(CausalEffect::relay_to(0, CausalEffect::value(0u64))),
        state,
        ctx,
        EffectLog::new(),
    )
}

#[test]
fn evaluate_subgraph_stateful_cuts_a_relay_cycle_with_the_fuel_bound() {
    // 0 relays to 1 and 1 relays back to 0 — an unbounded adaptive-reasoning loop that no
    // edge-level acyclicity check can see. The relay fuel (`MAX_RELAY_ROUNDS`) makes the handler
    // total: evaluation terminates with a loud budget error
    // (`core.causal_effect.relay_termination`) instead of hanging.
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_relay_to_one, ConfigCtx {}, "ping");
    let n1 = Causaloid::new_with_context(1, node_relay_to_zero, ConfigCtx {}, "pong");
    let i0 = g.add_root_causaloid(n0).expect("root");
    let _i1 = g.add_causaloid(n1).expect("n1");
    g.freeze();

    let out = g.evaluate_subgraph_from_cause_stateful(i0, &build_initial());
    assert_eq!(
        out.error(),
        Some(&CausalityError::MaxStepsExceeded()),
        "the relay cycle errors instead of hanging"
    );
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
    // The relayer names index 2; the graph holds only 0 and 1.
    assert_eq!(out.error(), Some(&CausalityError::CausaloidNotFound(2)));
    // The errored carrier holds no value: the stale relay value is not preserved.
    assert!(out.value().is_none());
}

// --- shortest-path specific branches ---

#[test]
fn evaluate_shortest_path_stateful_start_equals_stop_runs_only_that_node() {
    let g = build_three_node_path();
    let out = g.evaluate_shortest_path_between_causes_stateful(1, 1, &build_initial());
    assert!(out.is_ok(), "got {:?}", out.error());
    assert_eq!(out.state().count, 1, "exactly one node runs");
}

#[test]
fn evaluate_shortest_path_stateful_errors_when_no_path_exists() {
    let g = build_two_unconnected();
    let out = g.evaluate_shortest_path_between_causes_stateful(0, 1, &build_initial());
    assert!(out.is_err(), "no path between disconnected nodes");
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
    assert!(out.is_err(), "a failing node aborts the path walk");
    assert_eq!(out.state().count, 1, "only node 0 advanced state");
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
    assert!(out.is_ok(), "got {:?}", out.error());
    assert!(
        out.command_target() == Some(2),
        "the walk returns the relaying node's process"
    );
}

#[test]
fn evaluate_subgraph_stateful_multi_fired_reconvergence_errors_loudly() {
    // Stateful diamond: root(0) -> A(1), B(2); A,B -> C(3). Starting at the root fires both
    // A and B into C, a multi-fired reconvergence. The merge (∇) is undefined (assumption #2),
    // so this must fail loudly rather than silently pick one parent.
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let n0 = Causaloid::new_with_context(0, node_increment, ConfigCtx {}, "root");
    let n1 = Causaloid::new_with_context(1, node_increment, ConfigCtx {}, "A");
    let n2 = Causaloid::new_with_context(2, node_increment, ConfigCtx {}, "B");
    let n3 = Causaloid::new_with_context(3, node_increment, ConfigCtx {}, "C");
    let i0 = g.add_root_causaloid(n0).expect("root");
    let i1 = g.add_causaloid(n1).expect("A");
    let i2 = g.add_causaloid(n2).expect("B");
    let i3 = g.add_causaloid(n3).expect("C");
    g.add_edge(i0, i1).expect("edge 0->1");
    g.add_edge(i0, i2).expect("edge 0->2");
    g.add_edge(i1, i3).expect("edge 1->3");
    g.add_edge(i2, i3).expect("edge 2->3");
    g.freeze();

    let out = g.evaluate_subgraph_from_cause_stateful(0, &build_initial());
    let err = out.error().expect("a multi-fired join errors");
    assert!(
        matches!(&err.0, deep_causality::CausalityErrorEnum::UnsupportedOperation(m) if m.contains("reconvergence merge (∇) is not")),
        "expected UnsupportedOperation: {:?}",
        err
    );
}

// --- permanent positions: a removed index below a live causaloid ---

fn node_add_one(
    obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    let val = obs.into_value().unwrap_or(0);
    PropagatingProcess::new(
        Ok(CausalEffect::value(val + 1)),
        state,
        ctx,
        EffectLog::new(),
    )
}

fn node_times_hundred(
    obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    let val = obs.into_value().unwrap_or(0);
    PropagatingProcess::new(
        Ok(CausalEffect::value(val * 100)),
        state,
        ctx,
        EffectLog::new(),
    )
}

fn node_double(
    obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    let val = obs.into_value().unwrap_or(0);
    PropagatingProcess::new(
        Ok(CausalEffect::value(val * 2)),
        state,
        ctx,
        EffectLog::new(),
    )
}

fn node_add_ten(
    obs: CausalEffect<u64>,
    mut state: CounterState,
    ctx: Option<ConfigCtx>,
) -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    state.count += 1;
    let val = obs.into_value().unwrap_or(0);
    PropagatingProcess::new(
        Ok(CausalEffect::value(val + 10)),
        state,
        ctx,
        EffectLog::new(),
    )
}

/// A process carrying the value 3 and a zeroed counter.
fn build_initial_three() -> PropagatingProcess<u64, CounterState, ConfigCtx> {
    PropagatingProcess::new(
        Ok(CausalEffect::value(3)),
        CounterState::default(),
        Some(ConfigCtx {}),
        EffectLog::new(),
    )
}

/// Causaloids 0 (`+1`), 1 (`×100`) and 2 (`×2`) with edges 0 → 1, 1 → 2 and 0 → 2; index 1
/// removed; frozen. Only the edge 0 → 2 survives the removal.
fn build_removed_middle() -> CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> {
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let i0 = g
        .add_causaloid(Causaloid::new_with_context(
            0,
            node_add_one,
            ConfigCtx {},
            "n0",
        ))
        .expect("n0");
    let i1 = g
        .add_causaloid(Causaloid::new_with_context(
            1,
            node_times_hundred,
            ConfigCtx {},
            "n1",
        ))
        .expect("n1");
    let i2 = g
        .add_causaloid(Causaloid::new_with_context(
            2,
            node_double,
            ConfigCtx {},
            "n2",
        ))
        .expect("n2");
    assert_eq!([i0, i1, i2], [0, 1, 2]);
    g.add_edge(0, 1).expect("edge 0->1");
    g.add_edge(1, 2).expect("edge 1->2");
    g.add_edge(0, 2).expect("edge 0->2");
    g.remove_causaloid(1).expect("remove 1");
    g.freeze();
    g
}

/// The chain 0 (`+1`) → 1 (`×100`) → 2 (`×2`) → 3 (`+10`) with 0 → 2 added and index 1 removed;
/// frozen. The live causaloid 3 sits at an index >= `number_nodes()` = 3.
fn build_chain_with_removed_middle() -> CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>>
{
    let mut g: CausaloidGraph<Causaloid<u64, u64, CounterState, ConfigCtx>> =
        CausaloidGraph::new(0u64);
    let i0 = g
        .add_causaloid(Causaloid::new_with_context(
            0,
            node_add_one,
            ConfigCtx {},
            "n0",
        ))
        .expect("n0");
    let i1 = g
        .add_causaloid(Causaloid::new_with_context(
            1,
            node_times_hundred,
            ConfigCtx {},
            "n1",
        ))
        .expect("n1");
    let i2 = g
        .add_causaloid(Causaloid::new_with_context(
            2,
            node_double,
            ConfigCtx {},
            "n2",
        ))
        .expect("n2");
    let i3 = g
        .add_causaloid(Causaloid::new_with_context(
            3,
            node_add_ten,
            ConfigCtx {},
            "n3",
        ))
        .expect("n3");
    assert_eq!([i0, i1, i2, i3], [0, 1, 2, 3]);
    g.add_edge(0, 1).expect("edge 0->1");
    g.add_edge(1, 2).expect("edge 1->2");
    g.add_edge(2, 3).expect("edge 2->3");
    g.add_edge(0, 2).expect("edge 0->2");
    g.remove_causaloid(1).expect("remove 1");
    g.freeze();
    g
}

#[test]
fn evaluate_subgraph_stateful_crosses_a_removed_index() {
    let g = build_removed_middle();
    // The index `add_causaloid` returned still names causaloid 2 after the removal and the freeze.
    assert_eq!(g.number_nodes(), 2);
    assert!(!g.contains_causaloid(1));
    assert!(g.contains_causaloid(2));
    assert_eq!(g.get_causaloid(2).map(|c| c.id()), Some(2));

    // Hand derivation: 0 turns 3 into 3 + 1 = 4; the removed 1 never fires; 2 doubles 4 to 8.
    // Two causaloids ran, so the counter reads 2. Through 1 the value would have been 800.
    let out = g.evaluate_subgraph_from_cause_stateful(0, &build_initial_three());
    assert!(out.is_ok(), "got {:?}", out.error());
    assert_eq!(out.value(), Some(&8));
    assert_eq!(out.state().count, 2);
}

#[test]
fn evaluate_subgraph_stateful_starts_at_a_live_index_at_or_above_number_nodes() {
    let g = build_removed_middle();
    assert!(2 >= g.number_nodes());

    // Hand derivation: 2 doubles 3 to 6; one causaloid ran.
    let out = g.evaluate_subgraph_from_cause_stateful(2, &build_initial_three());
    assert!(out.is_ok(), "got {:?}", out.error());
    assert_eq!(out.value(), Some(&6));
    assert_eq!(out.state().count, 1);
}

#[test]
fn evaluate_subgraph_stateful_reaches_a_causaloid_at_an_index_above_number_nodes() {
    let g = build_chain_with_removed_middle();
    assert_eq!(g.number_nodes(), 3);
    assert!(3 >= g.number_nodes());
    assert!(g.contains_causaloid(3));

    // Hand derivation: (3 + 1) · 2 + 10 = 18 over three causaloids. Without causaloid 3 firing
    // the result would be the output of 2, which is 8 with a count of 2.
    let out = g.evaluate_subgraph_from_cause_stateful(0, &build_initial_three());
    assert!(out.is_ok(), "got {:?}", out.error());
    assert_eq!(out.value(), Some(&18));
    assert_eq!(out.state().count, 3);
}

#[test]
fn evaluate_subgraph_stateful_from_a_removed_index_is_causaloid_not_found() {
    let g = build_removed_middle();
    let out = g.evaluate_subgraph_from_cause_stateful(1, &build_initial_three());
    assert_eq!(out.error(), Some(&CausalityError::CausaloidNotFound(1)));
}
