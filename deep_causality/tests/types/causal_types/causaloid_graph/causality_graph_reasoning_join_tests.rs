/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Fan-in (reconvergence-join) behaviour of `evaluate_subgraph_from_cause`.
//!
//! These are the multi-fired diamond cases that the existing suite lacked: a reconvergence
//! node reached by two fired parents from a common root. See the blast-radius scan in the
//! `comonoid-graph-join` change: every pre-existing reconvergent graph is single-fired at
//! run time, so this file is where the labeled-join path is actually exercised.

use deep_causality::{
    CausableGraph, CausalityError, CausalityErrorEnum, Causaloid, CausaloidGraph,
    MonadicCausableGraphReasoning, ParentEffects, PropagatingEffect,
};

// ----- node mechanisms -----

fn identity(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x)
}

fn add_one(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 1.0)
}

fn add_ten(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 10.0)
}

fn erroring(_: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
        "A failed".into(),
    )))
}

// ----- join mechanisms -----

/// Symmetric sum over the labeled parents.
fn sum_join(parents: &ParentEffects<f64>) -> PropagatingEffect<f64> {
    let mut acc = 0.0;
    for (_, effect) in parents.iter() {
        if let Some(v) = effect.value() {
            acc += *v;
        }
    }
    PropagatingEffect::from_value(acc)
}

/// Asymmetric: `2·parent[1] − parent[2]`. Distinguishes the two parents by key.
fn asym_join(parents: &ParentEffects<f64>) -> PropagatingEffect<f64> {
    let p1 = parents
        .get(1)
        .and_then(|e| e.value())
        .copied()
        .unwrap_or(0.0);
    let p2 = parents
        .get(2)
        .and_then(|e| e.value())
        .copied()
        .unwrap_or(0.0);
    PropagatingEffect::from_value(2.0 * p1 - p2)
}

// ----- diamond builder: root(0) -> A(1), B(2); A -> C(3), B -> C(3) -----

type Diamond = CausaloidGraph<Causaloid<f64, f64, (), ()>>;

/// Builds the diamond with the given node `c` at index 3 (the reconvergence).
/// `a_fn` / `b_fn` are the two branch mechanisms. Node indices are the insertion order:
/// root=0, A=1, B=2, C=3.
fn build_diamond(
    a_fn: fn(f64) -> PropagatingEffect<f64>,
    b_fn: fn(f64) -> PropagatingEffect<f64>,
    c: Causaloid<f64, f64, (), ()>,
) -> Diamond {
    let mut g = CausaloidGraph::new(0);
    let root = g
        .add_causaloid(Causaloid::new(0, identity, "root"))
        .unwrap();
    let a = g.add_causaloid(Causaloid::new(1, a_fn, "A")).unwrap();
    let b = g.add_causaloid(Causaloid::new(2, b_fn, "B")).unwrap();
    let cc = g.add_causaloid(c).unwrap();
    g.add_edge(root, a).unwrap();
    g.add_edge(root, b).unwrap();
    g.add_edge(a, cc).unwrap();
    g.add_edge(b, cc).unwrap();
    g.freeze();
    g
}

#[test]
fn diamond_root_start_declared_sum_join() {
    // root(id)=0 -> A(+1)=1, B(+10)=10 -> C sum-join = 11 -> C identity = 11.
    let c = Causaloid::new_join(3, identity, sum_join, "C");
    let g = build_diamond(add_one, add_ten, c);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok());
    assert_eq!(res.value(), Some(&11.0));
}

#[test]
fn diamond_root_start_undeclared_join_errors() {
    // C has no join mechanism; two fired parents -> descriptive error (D4).
    let c = Causaloid::new(3, identity, "C");
    let g = build_diamond(add_one, add_ten, c);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_err());
    let msg = res.error().unwrap().to_string();
    assert!(
        msg.contains("no join mechanism"),
        "unexpected message: {msg}"
    );
}

#[test]
fn diamond_single_fired_is_identity_without_join() {
    // Start at B(2): A(1) is not a descendant of B, so C(3) is single-fired -> identity,
    // no join declaration required.
    let c = Causaloid::new(3, identity, "C");
    let g = build_diamond(add_one, add_ten, c);
    let res = g.evaluate_subgraph_from_cause(2, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok());
    // B(+10)=10; C identity from B = 10.
    assert_eq!(res.value(), Some(&10.0));
}

#[test]
fn diamond_asymmetric_join_distinguishes_parents() {
    // 2·A − B = 2·1 − 10 = −8. Parent identity is preserved by the keys.
    let c = Causaloid::new_join(3, identity, asym_join, "C");
    let g = build_diamond(add_one, add_ten, c);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok());
    assert_eq!(res.value(), Some(&-8.0));
}

#[test]
fn diamond_parent_error_short_circuits() {
    // A errors -> the whole traversal short-circuits before the join runs (left-zero).
    let c = Causaloid::new_join(3, identity, sum_join, "C");
    let g = build_diamond(erroring, add_ten, c);
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("A failed"));
}

#[test]
fn diamond_join_of_one_never_invokes_join() {
    // A join that panics if ever called; with a single fired parent it must not run.
    fn poison_join(_: &ParentEffects<f64>) -> PropagatingEffect<f64> {
        panic!("join must not run for a single fired parent");
    }
    let c = Causaloid::new_join(3, identity, poison_join, "C");
    let g = build_diamond(add_one, add_ten, c);
    // Start at B(2) -> C single-fired -> identity, poison_join never invoked.
    let res = g.evaluate_subgraph_from_cause(2, &PropagatingEffect::from_value(0.0));
    assert!(res.is_ok());
    assert_eq!(res.value(), Some(&10.0));
}

#[test]
fn diamond_determinism_under_index_relabeling() {
    // Same diamond, branch insertion order swapped, so the reconvergence's parents get the
    // opposite indices. A symmetric join must agree — schedule invariance modulo relabeling.
    let c1 = Causaloid::new_join(3, identity, sum_join, "C");
    let g1 = build_diamond(add_one, add_ten, c1);
    let r1 = g1.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));

    // Swap the branch mechanisms (and thus which index carries which value).
    let c2 = Causaloid::new_join(3, identity, sum_join, "C");
    let g2 = build_diamond(add_ten, add_one, c2);
    let r2 = g2.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(0.0));

    assert_eq!(r1.value(), Some(&11.0));
    assert_eq!(r2.value(), r1.value());
}
