/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for the labeled reconvergence-join laws.
//!
//! Mirrors `lean/DeepCausalityFormal/Core/GraphJoin.lean`. The Lean proofs model the fan-in as an
//! acyclic labeled equation system `σ(n) = f_n(σ|Pa(n))`; these witnesses check the same laws on
//! the real engine (`evaluate_subgraph_from_cause`) and the `LinearJoin` kernel. One `#[test]` per
//! THEOREM_MAP id.

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, LinearJoin, MonadicCausableGraphReasoning,
    ParentEffects, PropagatingEffect, linear_join,
};
use std::collections::BTreeMap;

fn identity(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x)
}
fn add_one(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 1.0)
}
fn add_ten(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 10.0)
}

fn sum_join(parents: &ParentEffects<f64>) -> PropagatingEffect<f64> {
    let mut acc = 0.0;
    for (_, e) in parents.iter() {
        if let Some(v) = e.value() {
            acc += *v;
        }
    }
    PropagatingEffect::from_value(acc)
}

/// Diamond root(0) -> A(1), B(2); A,B -> C(3), with branch mechanisms `a_fn`/`b_fn` and join `c`.
fn diamond(
    a_fn: fn(f64) -> PropagatingEffect<f64>,
    b_fn: fn(f64) -> PropagatingEffect<f64>,
    c: Causaloid<f64, f64, (), ()>,
) -> CausaloidGraph<Causaloid<f64, f64, (), ()>> {
    let mut g = CausaloidGraph::new(0);
    let r = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let a = g.add_causaloid(Causaloid::new(1, a_fn, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(2, b_fn, "B")).unwrap();
    let cc = g.add_causaloid(c).unwrap();
    g.add_edge(r, a).unwrap();
    g.add_edge(r, b).unwrap();
    g.add_edge(a, cc).unwrap();
    g.add_edge(b, cc).unwrap();
    g.freeze();
    g
}

/// `core.graph_join.unique_valuation` — the acyclic labeled system has a single solution, so the
/// diamond evaluates to one determined value.
#[test]
fn test_core_graph_join_unique_valuation() {
    let g = diamond(
        add_one,
        add_ten,
        Causaloid::new_join(3, identity, sum_join, "C"),
    );
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    // Unique solution: A=1, B=10, C = sum = 11.
    assert_eq!(res.value(), Some(&11.0));
}

/// `core.graph_join.schedule_invariance` (command-free) — two schedules (here induced by swapping
/// the branch insertion order, which permutes the reconvergence's parent indices) agree.
#[test]
fn test_core_graph_join_schedule_invariance() {
    let g1 = diamond(
        add_one,
        add_ten,
        Causaloid::new_join(3, identity, sum_join, "C"),
    );
    let g2 = diamond(
        add_ten,
        add_one,
        Causaloid::new_join(3, identity, sum_join, "C"),
    );
    let r1 = g1.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    let r2 = g2.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert_eq!(r1.value(), r2.value());
}

/// `core.graph_join.union_comm` — the fired-parent map keys each parent once, so building it in
/// either insertion order yields the same join result (disjoint-key union is commutative).
#[test]
fn test_core_graph_join_union_comm() {
    let mut m1: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    m1.insert(1, PropagatingEffect::from_value(3.0));
    m1.insert(2, PropagatingEffect::from_value(5.0));

    let mut m2: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    m2.insert(2, PropagatingEffect::from_value(5.0));
    m2.insert(1, PropagatingEffect::from_value(3.0));

    let r1 = sum_join(&ParentEffects::new(m1));
    let r2 = sum_join(&ParentEffects::new(m2));
    assert_eq!(r1.value(), r2.value());
    assert_eq!(r1.value(), Some(&8.0));
}

/// `core.graph_join.classical_copy` — a fan-out node delivers the same value to every child.
#[test]
fn test_core_graph_join_classical_copy() {
    // root(0) -> S(1); S -> C1(2), S -> C2(3). Both children read S's output identically.
    let mut g: CausaloidGraph<Causaloid<f64, f64, (), ()>> = CausaloidGraph::new(0);
    let r = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let s = g.add_causaloid(Causaloid::new(1, add_ten, "S")).unwrap();
    let c1 = g.add_causaloid(Causaloid::new(2, identity, "C1")).unwrap();
    let c2 = g.add_causaloid(Causaloid::new(3, identity, "C2")).unwrap();
    g.add_edge(r, s).unwrap();
    g.add_edge(s, c1).unwrap();
    g.add_edge(s, c2).unwrap();
    g.freeze();

    // Evaluating to each child separately yields the same copied value from S (= 10).
    let to_c1 = g.evaluate_shortest_path_between_causes(1, 2, &PropagatingEffect::from_value(0.0));
    let to_c2 = g.evaluate_shortest_path_between_causes(1, 3, &PropagatingEffect::from_value(0.0));
    assert_eq!(to_c1.value(), Some(&10.0));
    assert_eq!(to_c1.value(), to_c2.value());
}

/// `core.graph_join.linear_surgery_locality` — cutting parent `p`'s wire drops exactly
/// `weights[p]·v_p` from the `LinearJoin` result (the kernel-level shadow of opening a mechanism).
#[test]
fn test_core_graph_join_linear_surgery_locality() {
    let config = LinearJoin::new([(1, 2.0), (2, 3.0)].into_iter().collect(), 1.0);

    let mut full: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    full.insert(1, PropagatingEffect::from_value(4.0));
    full.insert(2, PropagatingEffect::from_value(10.0));
    let full_res = linear_join(&ParentEffects::new(full), Some(&config));

    let mut cut: BTreeMap<usize, PropagatingEffect<f64>> = BTreeMap::new();
    cut.insert(2, PropagatingEffect::from_value(10.0));
    let cut_res = linear_join(&ParentEffects::new(cut), Some(&config));

    // full − cut == weights[1]·v1 == 2.0·4.0 == 8.0.
    assert_eq!(full_res.value().unwrap() - cut_res.value().unwrap(), 8.0);
}
