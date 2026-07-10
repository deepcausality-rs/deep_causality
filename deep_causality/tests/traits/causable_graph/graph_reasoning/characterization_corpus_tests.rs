/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The #10 characterization corpus — the behavior-preservation gate for the Stage-4 graph-join
//! change (`openspec/notes/causal-algebra/algebraic-causaloid-assumptions.md` #10; design D7).
//!
//! Each test pins the EXACT observable output of `evaluate_subgraph_from_cause` — value channel,
//! error channel, and the full ordered log-message sequence (timestamp-free via
//! `EffectLog::messages`) — for one graph shape, captured against the pre-change engine:
//!
//! * **Chains, trees, fan-outs are bit-identical**: their snapshots below are the pre-change
//!   captures, unmodified — single-parent joins are the identity fuse before and after.
//! * **The reconvergent diamond is the ONE documented behavior change**: it previously failed
//!   loudly ("Node 3 is a reconvergence reached by 2 fired parents … the reconvergence merge (∇)
//!   is not yet defined"); it now computes the defined join `∇ ∘ (Λ₁ ⊗ Λ₂)` with
//!   `∇ = Verdict::join` and the identity Λ (no decorations), per
//!   `core.causaloid.graph_fold_order_invariant`. The diamond tests pin the NEW merged output.

use deep_causality::utils_test::{test_utils, test_utils_graph};
use deep_causality::{
    CausableGraph, CausaloidGraph, MonadicCausableGraphReasoning, PropagatingEffect,
};

/// The full observable snapshot of one evaluation: (value, error rendering, log messages).
fn snapshot(effect: &PropagatingEffect<f64>) -> (Option<f64>, Option<String>, Vec<String>) {
    (
        effect.value().copied(),
        effect.error().map(|e| e.to_string()),
        effect.logs().messages().map(|m| m.to_string()).collect(),
    )
}

/// The per-node log block of `get_test_causaloid_num_input_output` for one evaluation step.
fn node_block(id: usize, incoming: &str, evidence: &str) -> Vec<String> {
    vec![
        format!("Causaloid {id}: Incoming effect: Value({incoming})"),
        format!("Processing evidence: {evidence}"),
        format!("Evidence {evidence} >= threshold 0.55: 1"),
        format!("Causaloid {id}: Outgoing effect: Value(1.0)"),
    ]
}

#[test]
fn test_corpus_chain_bit_identical() {
    // Pre-change capture (2026-07-10): the linear chain 0 -> 1 -> 2 -> 3, input 0.99.
    let g = test_utils_graph::build_linear_graph(4);
    let effect = PropagatingEffect::from_value(0.99f64);
    let (value, error, messages) = snapshot(&g.evaluate_subgraph_from_cause(0, &effect));

    let mut expected = node_block(0, "0.99", "0.99");
    expected.extend(node_block(1, "1.0", "1"));
    expected.extend(node_block(2, "1.0", "1"));
    expected.extend(node_block(3, "1.0", "1"));

    assert_eq!(value, Some(1.0));
    assert_eq!(error, None);
    assert_eq!(messages, expected);
}

#[test]
fn test_corpus_fanout_tree_bit_identical() {
    // Pre-change capture (2026-07-10): root(0) -> {A(1), B(2)}; A -> D(3); B -> E(4); no
    // reconvergence. The returned effect is the last node under the ascending-index schedule
    // (E = 4), carrying only its own lineage root -> B -> E.
    let mut g = CausaloidGraph::new(0);
    let root = g
        .add_root_causaloid(test_utils::get_test_causaloid_num_input_output(0))
        .expect("root");
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_num_input_output(1))
        .expect("A");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_num_input_output(2))
        .expect("B");
    let d = g
        .add_causaloid(test_utils::get_test_causaloid_num_input_output(3))
        .expect("D");
    let e = g
        .add_causaloid(test_utils::get_test_causaloid_num_input_output(4))
        .expect("E");
    g.add_edge(root, a).expect("root->A");
    g.add_edge(root, b).expect("root->B");
    g.add_edge(a, d).expect("A->D");
    g.add_edge(b, e).expect("B->E");
    g.freeze();

    let effect = PropagatingEffect::from_value(0.99f64);
    let (value, error, messages) = snapshot(&g.evaluate_subgraph_from_cause(0, &effect));

    let mut expected = node_block(0, "0.99", "0.99");
    expected.extend(node_block(2, "1.0", "1"));
    expected.extend(node_block(4, "1.0", "1"));

    assert_eq!(value, Some(1.0));
    assert_eq!(error, None);
    assert_eq!(messages, expected);
}

#[test]
fn test_corpus_diamond_defined_merge() {
    // THE documented Stage-4 behavior change (BREAKING on this shape only). Pre-change, this
    // diamond — root(0) -> {A(1), B(2)} -> C(3) — failed loudly at C: "Node 3 is a reconvergence
    // reached by 2 fired parents (graph indices [1, 2]); the reconvergence merge (∇) is not yet
    // defined and multi-parent fan-in is unsupported." Post-change, C's incoming wires fuse as
    // `∇(Λ₁(a), Λ₂(b))` with `∇ = Verdict::join` (f64: max) and identity Λ: join(1.0, 1.0) = 1.0
    // feeds C, and the join concatenates both branches' logs in ascending parent-index order
    // (the multiset-at-join representative) — each branch carries its own copy of the root's
    // lineage, both are kept.
    let g = test_utils_graph::build_multi_cause_graph();
    let effect = PropagatingEffect::from_value(0.99f64);
    let (value, error, messages) = snapshot(&g.evaluate_subgraph_from_cause(0, &effect));

    let mut expected = node_block(0, "0.99", "0.99"); // branch A's lineage: root...
    expected.extend(node_block(1, "1.0", "1")); //         ... then A
    expected.extend(node_block(0, "0.99", "0.99")); // branch B's lineage: root again...
    expected.extend(node_block(2, "1.0", "1")); //          ... then B
    expected.extend(node_block(3, "1.0", "1")); // the join value feeds C

    assert_eq!(value, Some(1.0));
    assert_eq!(error, None);
    assert_eq!(messages, expected);
}

#[test]
fn test_corpus_layered_reconvergent_defined_merge() {
    // Also previously loud-failing (reconvergence at E(5): A,B and F(6): B,C). Post-change the
    // layered graph evaluates end-to-end; the returned effect is the last node under the
    // ascending-index schedule (G = 7), whose lineage root -> C(3) -> G(7) contains no join —
    // so its log is the plain single-parent chain.
    let g = test_utils_graph::build_multi_layer_cause_graph();
    let effect = PropagatingEffect::from_value(0.99f64);
    let (value, error, messages) = snapshot(&g.evaluate_subgraph_from_cause(0, &effect));

    let mut expected = node_block(0, "0.99", "0.99");
    expected.extend(node_block(3, "1.0", "1"));
    expected.extend(node_block(7, "1.0", "1"));

    assert_eq!(value, Some(1.0));
    assert_eq!(error, None);
    assert_eq!(messages, expected);
}
