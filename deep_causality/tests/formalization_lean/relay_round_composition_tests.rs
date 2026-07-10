/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the relay-round composition theorems in
//! `lean/DeepCausalityFormal/Core/CausalEffect.lean` — multi-round adaptive evaluation is the
//! sequential (Kleisli) composition of its rounds (graph-reasoning-formalization, task 3.6). Lean
//! proves the rounds compose (`rounds_add`), an answer is stable under further rounds
//! (`run_monotone_add`), the fuel-bounded run splits at a round boundary (`run_rounds_compose`,
//! `run_relay_peel`), and a relay cycle is cut (`run_self_relay_none`). These tests pin the real
//! graph-reasoning engine's `'rounds` loop to those statements.

use deep_causality::{
    CausableGraph, CausalEffect, Causaloid, CausaloidGraph, MonadicCausableGraphReasoning,
    PropagatingEffect,
};

type G = CausaloidGraph<Causaloid<bool, bool, (), ()>>;

fn identity(o: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(o)
}
fn inverter(o: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(!o)
}
/// Round 1 ends here: relay to node 2 with the sub-program `value(false)`.
fn relay_to_2(_: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect(CausalEffect::relay_to(2, CausalEffect::value(false)))
}

/// THEOREM_MAP: core.causal_effect.relay_round_composition
///
/// Lean: `run_relay_peel`, `run_rounds_compose`, `rounds_add` (`Core/CausalEffect.lean`). A two-round
/// adaptive graph run equals the Kleisli composite of its rounds: round 1 ends in `RelayTo(2, false)`,
/// and the whole run equals running round 2 from the target (node 2) seeded with the relayed sub
/// (`value(false)`) — exactly `run (n+1) (relay t sub) g = run n (g t sub) g`. Logs concatenate
/// across the round boundary (the full run carries round 1's entries too).
#[test]
fn test_relay_round_composition() {
    // Graph: [0] Id -> [1] RelayTo(2, false); node [2] Inverter (NO edge to 2 — the relay jumps).
    let mut g: G = CausaloidGraph::new(1);
    let n0 = g.add_causaloid(Causaloid::new(0, identity, "id")).unwrap();
    let n1 = g
        .add_causaloid(Causaloid::new(1, relay_to_2, "relayer"))
        .unwrap();
    let n2 = g
        .add_causaloid(Causaloid::new(2, inverter, "inverter"))
        .unwrap();
    assert_eq!(n2, 2, "relay target must be index 2");
    g.add_edge(n0, n1).unwrap();
    g.freeze();

    // The full two-round run from node 0.
    let full = g.evaluate_subgraph_from_cause(n0, &PropagatingEffect::pure(true));
    assert!(!full.is_err(), "full run errored: {:?}", full.error());
    let full_val = *full.value().expect("full run yields a value");
    assert!(
        full_val,
        "flow [0]true -> [1]RelayTo(2,false) -> [2]invert(false)=true"
    );

    // Round 2 in isolation: run from the relay TARGET (node 2) with the relayed sub-program as the
    // seed — the continuation the RelayTo hands off to (`run n (g t sub) g` of `run_relay_peel`).
    let round2 = g.evaluate_subgraph_from_cause(
        n2,
        &PropagatingEffect::from_effect(CausalEffect::value(false)),
    );
    let round2_val = *round2.value().expect("round 2 yields a value");

    // Two rounds compose as one arrow: the full run equals its round-2 continuation.
    assert_eq!(
        full_val, round2_val,
        "full run != Kleisli composite of the rounds"
    );

    // Logs concatenate across the boundary: the full run carries round 1's entries (nodes 0, 1) plus
    // round 2's, so strictly more than round 2 alone (which starts from an empty log).
    let full_logs = full.logs().messages().count();
    let round2_logs = round2.logs().messages().count();
    assert!(
        full_logs > round2_logs,
        "logs not concatenated across the round boundary: full={full_logs} round2={round2_logs}"
    );
}

/// A causaloid that relays to node 1 (one half of a relay cycle).
fn relay_to_1(_: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect(CausalEffect::relay_to(1, CausalEffect::value(true)))
}
/// A causaloid that relays back to node 0 (the other half of the cycle).
fn relay_to_0(_: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect(CausalEffect::relay_to(0, CausalEffect::value(true)))
}

/// THEOREM_MAP: core.causal_effect.relay_round_composition
///
/// Lean: `run_self_relay_none`, `run_monotone_add` (`Core/CausalEffect.lean`). The fuel bound
/// composes across rounds with no new termination argument (inherits
/// `core.causal_effect.relay_termination`): a relay CYCLE — two causaloids relaying to each other —
/// is cut at `MAX_RELAY_ROUNDS` with a specific "Relay budget exhausted" error rather than looping
/// forever.
#[test]
fn test_relay_round_fuel_bound_composes() {
    // No edges: the relay jumps by index, so the EDGE graph is acyclic (freeze passes); the cycle is
    // a runtime relay phenomenon, cut by the fuel bound.
    let mut g: G = CausaloidGraph::new(1);
    let n0 = g
        .add_causaloid(Causaloid::new(0, relay_to_1, "relay->1"))
        .unwrap();
    let n1 = g
        .add_causaloid(Causaloid::new(1, relay_to_0, "relay->0"))
        .unwrap();
    assert_eq!((n0, n1), (0, 1));
    g.freeze();

    let out = g.evaluate_subgraph_from_cause(n0, &PropagatingEffect::pure(true));
    assert!(out.is_err(), "a relay cycle must be cut, not loop forever");
    let msg = out.error().unwrap().to_string();
    assert!(
        msg.contains("Relay budget exhausted") && msg.contains("relay_termination"),
        "not the fuel-bound cut error: {msg}"
    );
}
