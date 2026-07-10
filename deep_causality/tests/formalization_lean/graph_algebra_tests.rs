/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/GraphAlgebra.lean` — the schedule-invariant
//! graph fold with `∇ ∘ (Λ₁ ⊗ Λ₂)` at reconvergent joins (Stage 4; assumption #2 Q1). Lean proves
//! ∀ (every consistent schedule computes the denotational value); these tests pin the real Kahn
//! engine to the same statements at representative inputs.

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, LambdaEdges, MonadicCausableGraphReasoning,
    PropagatingEffect, Verdict,
};

fn identity(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x)
}
fn add_one(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 1.0)
}
fn add_ten(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 10.0)
}

fn double_lambda(v: f64) -> f64 {
    v * 2.0
}
fn negate_lambda(v: f64) -> f64 {
    -v
}

type DiamondGraph = CausaloidGraph<Causaloid<f64, f64, (), ()>>;

/// A diamond root -> {branch₁, branch₂} -> join, with the branch insertion order chosen by the
/// caller. Different insertion orders give different node indices, hence a DIFFERENT engine
/// schedule (the ready-set pops ascending indices) over the same causal structure. Returns the
/// graph and the (root, first-branch, second-branch, join) indices where "first" is `add_one`.
fn build_diamond(add_one_first: bool) -> (DiamondGraph, [usize; 4]) {
    let mut g = CausaloidGraph::new(0);
    let root = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let (a, b) = if add_one_first {
        let a = g.add_causaloid(Causaloid::new(1, add_one, "A")).unwrap();
        let b = g.add_causaloid(Causaloid::new(2, add_ten, "B")).unwrap();
        (a, b)
    } else {
        let b = g.add_causaloid(Causaloid::new(2, add_ten, "B")).unwrap();
        let a = g.add_causaloid(Causaloid::new(1, add_one, "A")).unwrap();
        (a, b)
    };
    let c = g.add_causaloid(Causaloid::new(3, identity, "C")).unwrap();
    g.add_edge(root, a).unwrap();
    g.add_edge(root, b).unwrap();
    g.add_edge(a, c).unwrap();
    g.add_edge(b, c).unwrap();
    g.freeze();
    (g, [root, a, b, c])
}

/// THEOREM_MAP: core.causaloid.graph_fold_order_invariant
///
/// Lean: `exec_computes_val`, `schedule_invariant`, `fuse_perm` (`Core/GraphAlgebra.lean`). The
/// fold's result is a property of the graph, not of the schedule: two diamonds with opposite
/// branch insertion order — hence opposite engine schedules over the same causal structure —
/// produce the same join value, undecorated and Λ-decorated alike, because the fuse
/// `∇ = Verdict::join` is commutative and each Λ is keyed by intrinsic edge identity.
#[test]
fn test_graph_fold_order_invariant() {
    let effect = PropagatingEffect::from_value(0.0f64);

    // ∇ commutativity — the hypothesis the Lean theorem consumes (hcomm), on the real carrier.
    for (x, y) in [(1.0f64, 10.0), (0.0, 0.5), (-3.0, 2.0)] {
        assert_eq!(x.join(y), y.join(x));
    }

    // Undecorated: join = ∇(A, B) = max(1, 10) = 10 under BOTH schedules.
    let (g1, _) = build_diamond(true);
    let (g2, _) = build_diamond(false);
    let r1 = g1.evaluate_subgraph_from_cause(0, &effect);
    let r2 = g2.evaluate_subgraph_from_cause(0, &effect);
    assert!(r1.is_ok() && r2.is_ok());
    assert_eq!(r1.value(), Some(&10.0));
    assert_eq!(r1.value(), r2.value());

    // Λ-decorated: Λ(A→C) doubles, Λ(B→C) negates — keyed by intrinsic edge identity, so each
    // Λ lands on its own edge under either schedule: ∇(2·1, −10) = max(2, −10) = 2.
    let (g1, [_, a1, b1, c1]) = build_diamond(true);
    let (g2, [_, a2, b2, c2]) = build_diamond(false);
    let lambdas1 = LambdaEdges::new()
        .with_lambda(a1, c1, double_lambda)
        .with_lambda(b1, c1, negate_lambda);
    let lambdas2 = LambdaEdges::new()
        .with_lambda(a2, c2, double_lambda)
        .with_lambda(b2, c2, negate_lambda);
    let d1 = g1.evaluate_subgraph_from_cause_with_lambda_edges(0, &effect, &lambdas1);
    let d2 = g2.evaluate_subgraph_from_cause_with_lambda_edges(0, &effect, &lambdas2);
    assert!(d1.is_ok() && d2.is_ok());
    assert_eq!(d1.value(), Some(&2.0));
    assert_eq!(d1.value(), d2.value());

    // A single-parent decorated edge applies its Λ too (connection data, independent of fan-in):
    // starting at B, C is reached by one fired parent through the negating edge: -(10) = -10.
    let single = g1.evaluate_subgraph_from_cause_with_lambda_edges(b1, &effect, &lambdas1);
    assert!(single.is_ok());
    assert_eq!(single.value(), Some(&-10.0));
}

/// THEOREM_MAP: core.causaloid.graph_fold_order_invariant
///
/// The freeze-time preconditions of the fold (Lean deviation notes 1–2): acyclicity and the
/// single-writer invariant. A two-writer diamond is rejected at freeze naming the join; a single
/// writer passes; a writer ABOVE the fork (in every branch's cone) passes — it is seen
/// identically by both branches and cannot conflict.
#[test]
fn test_two_writer_diamond_rejected_at_freeze() {
    // Two branch-exclusive writers (A = 1, B = 2) -> freeze fails, naming join node 3.
    let (mut g, [_, a, b, _]) = build_diamond(true);
    g.unfreeze();
    let err = g
        .freeze_verified(&[a, b])
        .expect_err("two writing branches must be rejected");
    assert!(
        err.to_string()
            .contains("Single-writer violation at join node 3"),
        "unexpected error: {err}"
    );
    assert!(!g.is_frozen(), "a failed check must roll the freeze back");

    // One writing branch -> fine.
    let (mut g, [_, a, _, _]) = build_diamond(true);
    g.unfreeze();
    g.freeze_verified(&[a]).expect("one writer is allowed");
    assert!(g.is_frozen());

    // A writer above the fork (the root, in both cones) -> fine: pre-fork writes cannot conflict.
    let (mut g, [root, _, _, _]) = build_diamond(true);
    g.unfreeze();
    g.freeze_verified(&[root])
        .expect("a pre-fork writer is allowed");
    assert!(g.is_frozen());

    // The level-specific hook is the extension point: a rejecting hook fails the freeze too.
    let (mut g, _) = build_diamond(true);
    g.unfreeze();
    let err = g
        .freeze_verified_with_check(&[], |_| {
            Err(deep_causality::CausalityGraphError(
                "level check rejected".into(),
            ))
        })
        .expect_err("hook rejection must fail the freeze");
    assert!(err.to_string().contains("level check rejected"));
    assert!(!g.is_frozen());
}

/// A declared writer index that names no node (`>= number_nodes`) is caller error: the cone scan
/// only queries indices in `0..number_nodes`, so an out-of-range index would be silently dropped.
/// The freeze rejects it (naming the index) and rolls back, rather than appearing to satisfy a
/// writer declaration it never applied.
#[test]
fn test_out_of_range_state_writer_rejected() {
    let (mut g, _) = build_diamond(true); // 4 nodes: valid writer indices are 0..4.
    g.unfreeze();
    let err = g
        .freeze_verified(&[99])
        .expect_err("an out-of-range writer index must be rejected");
    assert!(
        err.to_string().contains("Invalid state-writer index 99"),
        "unexpected error: {err}"
    );
    assert!(!g.is_frozen(), "a failed check must roll the freeze back");
}
