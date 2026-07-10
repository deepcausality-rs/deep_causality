/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reconvergence-join behaviour of `MonadicCausableGraphReasoning` over a **Boolean** verdict
//! carrier (`∇ = Verdict::join = ||`), plus the acyclicity precondition of the fan-in evaluator.
//!
//! * A frozen graph carrying a directed cycle is rejected before any node runs.
//! * A reconvergent diamond fuses its two value-carrying parents with the commutative `∇`
//!   (`bool` OR) — a genuinely different carrier from the `f64` (max) witnesses in the
//!   `formalization_lean` suite.
//! * A per-edge Λ decoration transforms the value flowing along that edge before the merge, in
//!   both the multi-parent join and the single-parent identity fuse.

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, LambdaEdges, MonadicCausableGraphReasoning,
    PropagatingEffect,
};

fn pass(x: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_value(x)
}
fn emit_true(_x: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_value(true)
}
fn emit_false(_x: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_value(false)
}
fn negate(v: bool) -> bool {
    !v
}

type BoolGraph = CausaloidGraph<Causaloid<bool, bool, (), ()>>;

/// Diamond root(0) -> A(1), B(2); A -> D(3), B -> D(3). `a_fn`/`b_fn` are the branch emitters; D
/// passes its merged input through. Returns the graph and the `[root, a, b, d]` indices.
fn build_bool_diamond(
    a_fn: fn(bool) -> PropagatingEffect<bool>,
    b_fn: fn(bool) -> PropagatingEffect<bool>,
) -> (BoolGraph, [usize; 4]) {
    let mut g = CausaloidGraph::new(0);
    let root = g.add_causaloid(Causaloid::new(0, pass, "root")).unwrap();
    let a = g.add_causaloid(Causaloid::new(1, a_fn, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(2, b_fn, "B")).unwrap();
    let d = g.add_causaloid(Causaloid::new(3, pass, "D")).unwrap();
    g.add_edge(root, a).unwrap();
    g.add_edge(root, b).unwrap();
    g.add_edge(a, d).unwrap();
    g.add_edge(b, d).unwrap();
    g.freeze();
    (g, [root, a, b, d])
}

#[test]
fn test_cyclic_graph_is_rejected_before_any_node_runs() {
    // A 2-cycle A(0) -> B(1) -> A(0). Plain `freeze()` performs no acyclicity check, so the cycle
    // survives into the frozen graph; the fan-in evaluator must reject it up front.
    let mut g: BoolGraph = CausaloidGraph::new(0);
    let a = g.add_causaloid(Causaloid::new(0, pass, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(1, pass, "B")).unwrap();
    g.add_edge(a, b).unwrap();
    g.add_edge(b, a).unwrap();
    g.freeze();
    assert!(g.is_frozen());

    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    let msg = res
        .error()
        .expect("a cyclic graph must resolve to an error")
        .to_string();
    assert!(msg.contains("directed cycle"), "unexpected error: {msg}");
}

#[test]
fn test_diamond_or_join_true() {
    // Both A and B fire into D. ∇ = Verdict::join = OR: join(true, false) = true.
    let (g, _) = build_bool_diamond(emit_true, emit_false);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(false));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&true));
}

#[test]
fn test_diamond_or_join_false() {
    // Both branches emit false, so the OR-merge at D is false: join(false, false) = false.
    let (g, _) = build_bool_diamond(emit_false, emit_false);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(false));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&false));
}

#[test]
fn test_lambda_decorated_edge_transforms_value_into_the_join() {
    // Decorate edge A -> D with the negating Λ. A emits true, negated on its edge to false;
    // B emits false; the merge is join(false, false) = false — flipping the undecorated true.
    let (g, [_, a, _, d]) = build_bool_diamond(emit_true, emit_false);
    let lambdas = LambdaEdges::new().with_lambda(a, d, negate);

    let res = g.evaluate_subgraph_from_cause_with_lambda_edges(
        0,
        &PropagatingEffect::from_value(false),
        &lambdas,
    );
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&false));

    // Sanity: without the decoration the same diamond merges to true.
    let undecorated = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(false));
    assert_eq!(undecorated.value(), Some(&true));
}

#[test]
fn test_lambda_applies_on_single_parent_edge() {
    // Start at A(1): B is not a descendant of A, so D is reached by the single fired parent A.
    // The single-input fuse is the identity, but the edge's Λ still applies (connection data,
    // independent of fan-in).
    let (g, [_, a, _, d]) = build_bool_diamond(emit_true, emit_false);
    let lambdas = LambdaEdges::new().with_lambda(a, d, negate);

    // Undecorated single parent: identity passthrough of A's true.
    let identity_run = g.evaluate_subgraph_from_cause(a, &PropagatingEffect::from_value(false));
    assert!(identity_run.is_ok(), "got {:?}", identity_run.error());
    assert_eq!(identity_run.value(), Some(&true));

    // Decorated single parent: !true = false.
    let negated_run = g.evaluate_subgraph_from_cause_with_lambda_edges(
        a,
        &PropagatingEffect::from_value(false),
        &lambdas,
    );
    assert!(negated_run.is_ok(), "got {:?}", negated_run.error());
    assert_eq!(negated_run.value(), Some(&false));
}
