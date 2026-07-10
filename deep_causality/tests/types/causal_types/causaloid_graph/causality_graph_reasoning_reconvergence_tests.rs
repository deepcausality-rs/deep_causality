/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Reconvergence (multi-parent fan-in) behaviour of `evaluate_subgraph_from_cause`.
//!
//! The reconvergence **merge (∇)** landed with the Stage-4 graph algebra (the documented
//! behavior change of `causaloid-formalization-stages-2-5`; previously the engine failed loudly
//! here): converging values fuse as `join = ∇ ∘ (Λ₁ ⊗ Λ₂)` — each incoming edge's Λ decoration
//! applied by intrinsic edge identity, then the commutative `∇ = Verdict::join` (f64: max) — so
//! the result is invariant under every schedule consistent with the causal order
//! (`core.causaloid.graph_fold_order_invariant`, `lean/DeepCausalityFormal/Core/GraphAlgebra.lean`;
//! closes tracker #2 Q1). A reconvergence reached by a *single* fired parent is not a merge — it
//! is the identity, and evaluates normally.

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, MonadicCausableGraphReasoning, PropagatingEffect,
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

/// Diamond: root(0) -> A(1), B(2); A -> C(3), B -> C(3). Node indices are insertion order.
fn build_diamond() -> CausaloidGraph<Causaloid<f64, f64, (), ()>> {
    let mut g = CausaloidGraph::new(0);
    let root = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let a = g.add_causaloid(Causaloid::new(1, add_one, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(2, add_ten, "B")).unwrap();
    let c = g.add_causaloid(Causaloid::new(3, identity, "C")).unwrap();
    g.add_edge(root, a).unwrap();
    g.add_edge(root, b).unwrap();
    g.add_edge(a, c).unwrap();
    g.add_edge(b, c).unwrap();
    g.freeze();
    g
}

#[test]
fn reconvergence_multi_fired_merges_with_nabla_join() {
    // Start at the root: both A and B fire into C. Stage-4 behavior change (documented): the
    // former loud error ("the reconvergence merge (∇) is not yet defined") is now the defined
    // merge — with no Λ decorations, C receives ∇(A, B) = Verdict::join = max(1.0, 10.0) = 10.0,
    // and C (identity) returns it.
    let g = build_diamond();
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&10.0));
}

#[test]
fn reconvergence_single_fired_is_identity() {
    // Start at B(2): A(1) is not a descendant of B, so C(3) is reached by a single fired parent.
    // That is the identity (no merge), and evaluates normally.
    let g = build_diamond();
    let res = g.evaluate_subgraph_from_cause(2, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok(), "got {:?}", res.error());
    // B(+10)=10; C identity from B = 10.
    assert_eq!(res.value(), Some(&10.0));
}
