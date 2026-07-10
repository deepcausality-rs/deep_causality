/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/Causaloid.lean` — the signature functor `F`,
//! the fixpoint `Causaloid ≅ μX.F(X)`, and the Hardy inversion (Stage 2 of the causaloid
//! formalization roadmap). Lean proves ∀; these tests pin the real `Causaloid` to the same
//! statements at representative inputs.

use deep_causality::utils_test::test_utils;
use deep_causality::{
    AggregateLogic, BaseCausaloid, Causable, Causaloid, CausaloidType, LambdaEdges,
    MonadicCausable, MonadicCausableCollection, MonadicCausableGraphReasoning, PropagatingEffect,
};
use std::sync::Arc;

/// Nesting depth over the real recursive structure, via the unroll projections (the getters).
/// Terminating on every constructible causaloid is the μ-witness: values are built bottom-up and
/// `Arc`-shared, so no cycle inhabits the type (Lean: `size` is total by structural recursion).
fn depth(c: &BaseCausaloid<bool, bool>) -> usize {
    if let Some(coll) = c.causal_collection() {
        1 + coll.iter().map(depth).max().unwrap_or(0)
    } else {
        1
    }
}

/// THEOREM_MAP: core.causaloid.fixpoint
///
/// Lean: `roll_unroll`, `unroll_roll`, `size_pos` (`Core/Causaloid.lean`). The three summands of
/// `F` correspond one-to-one to the sealed `CausaloidType` forms; constructing (roll) and then
/// projecting (unroll) returns exactly the parts rolled in; nesting is finite (well-founded μ).
#[test]
fn test_fixpoint_three_forms_and_roll_unroll() {
    // Summand 1: Atom ↔ Singleton. Unroll returns the element (the causal function present).
    let atom = test_utils::get_test_causaloid_deterministic_true();
    assert_eq!(*atom.causal_type(), CausaloidType::Singleton);
    assert!(atom.is_singleton());
    assert!(atom.causal_fn().is_some());
    assert!(atom.causal_collection().is_none());
    assert!(atom.causal_graph().is_none());

    // Summand 2: Coll(Bag X, AggLogic) ↔ Collection. Unroll returns the same bag (Arc identity)
    // and the same aggregation logic that was rolled in.
    let bag = Arc::new(vec![
        test_utils::get_test_causaloid_deterministic_true(),
        test_utils::get_test_causaloid_deterministic_false(),
    ]);
    let coll: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
        10,
        bag.clone(),
        "coll summand",
        AggregateLogic::Any,
        0.5,
    );
    assert_eq!(*coll.causal_type(), CausaloidType::Collection);
    assert!(Arc::ptr_eq(
        coll.causal_collection().expect("bag projects back"),
        &bag
    ));
    assert_eq!(
        *coll.coll_aggregate_logic().expect("logic projects back"),
        { AggregateLogic::Any }
    );
    assert!(coll.causal_fn().is_none());
    assert!(coll.causal_graph().is_none());

    // Summand 3: Graph(Hyper X, Λ-edges) ↔ Graph. Unroll returns the same hypergraph (Arc
    // identity); the Λ-edge store projects back too (see the inversion witness below).
    let g = deep_causality::utils_test::test_utils_graph::build_linear_graph(3);
    let graph_arc = Arc::new(g);
    let graph_causaloid = Causaloid::from_causal_graph(11, "graph summand", graph_arc.clone());
    assert_eq!(*graph_causaloid.causal_type(), CausaloidType::Graph);
    assert!(Arc::ptr_eq(
        graph_causaloid.causal_graph().expect("graph projects back"),
        &graph_arc
    ));
    assert!(graph_causaloid.causal_fn().is_none());
    assert!(graph_causaloid.causal_collection().is_none());

    // Well-foundedness (μ, not ν): a nested causaloid — a collection of collections of atoms —
    // has finite depth, and the recursive walk over the unroll projections terminates.
    let inner: BaseCausaloid<bool, bool> =
        Causaloid::from_causal_collection(12, bag.clone(), "inner bag", AggregateLogic::All, 0.5);
    let outer: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
        13,
        Arc::new(vec![
            inner,
            test_utils::get_test_causaloid_deterministic_true(),
        ]),
        "outer bag",
        AggregateLogic::All,
        0.5,
    );
    assert_eq!(depth(&outer), 3);
}

/// THEOREM_MAP: core.causaloid.inversion
///
/// Lean: `eval_factors`, `mapL_perm` (`Core/Causaloid.lean`). The element carries no ordering
/// asymmetry: (1) evaluating an atom equals applying its element semantics directly (the
/// factorization at the atom case), and (2) the bag fragment is order-free — permuting a
/// collection's members does not change the aggregated value.
#[test]
fn test_inversion_element_is_symmetric() {
    // (1) Atom factorization: evaluate = wiring ∘ element, and at an atom the wiring is the
    // identity — the singleton's evaluation value equals the raw causal function's value.
    let atom = test_utils::get_test_causaloid_deterministic_true();
    let input = PropagatingEffect::from_value(true);
    let via_evaluate = atom.evaluate(&input);
    let raw_fn = atom.causal_fn().expect("singleton has an element");
    let via_element = raw_fn(true);
    assert_eq!(via_evaluate.value(), via_element.value());

    // (2) Element symmetry: the element sees values with intrinsic identity, never a position —
    // so the bag order cannot influence the aggregate (Lean: `mapL_perm` + assumption #1's
    // order-invariance). [t, f] and [f, t] aggregate identically under Any and All.
    let t = test_utils::get_test_causaloid_deterministic_true();
    let f = test_utils::get_test_causaloid_deterministic_false();
    let bag_tf = [t.clone(), f.clone()];
    let bag_ft = [f, t];
    let effect = PropagatingEffect::from_value(true);

    for logic in [AggregateLogic::Any, AggregateLogic::All] {
        let r_tf = bag_tf.evaluate_collection(&effect, &logic, Some(0.5));
        let r_ft = bag_ft.evaluate_collection(&effect, &logic, Some(0.5));
        assert_eq!(r_tf.value(), r_ft.value(), "order leaked through {logic:?}");
    }
}

fn lambda_double(v: f64) -> f64 {
    v * 2.0
}

fn lambda_negate(v: f64) -> f64 {
    -v
}

/// THEOREM_MAP: core.causaloid.inversion
///
/// Lean: D-fix-3 in `Core/Causaloid.lean` — Λ-edges are the keyed function
/// `Nat → Nat → Option Λ`, intrinsically enumeration-order-free. The Rust realization
/// (`LambdaEdges`) is witnessed here: slots are keyed by intrinsic edge identity
/// (`(source, target)`), which Λ applies to which input of a reconvergent join is independent of
/// insertion order, and an undecorated edge is the identity Λ.
#[test]
fn test_lambda_edges_identity_keyed_and_order_free() {
    // A reconvergent join at node 2 with two decorated in-edges: (0→2) doubles, (1→2) negates.
    // Two stores, opposite insertion order.
    let mut ab = LambdaEdges::<f64>::new();
    ab.insert(0, 2, lambda_double);
    ab.insert(1, 2, lambda_negate);

    let ba = LambdaEdges::<f64>::new()
        .with_lambda(1, 2, lambda_negate)
        .with_lambda(0, 2, lambda_double);

    // Lookup is by edge identity: both stores apply the same Λ to the same edge regardless of
    // the order the decorations were added.
    for store in [&ab, &ba] {
        assert_eq!(store.apply(0, 2, 3.0), 6.0);
        assert_eq!(store.apply(1, 2, 3.0), -3.0);
        assert_eq!(store.len(), 2);
    }

    // An undecorated edge is the identity Λ; an empty store is identity everywhere.
    assert_eq!(ab.apply(5, 7, 3.5), 3.5);
    let empty = LambdaEdges::<f64>::new();
    assert!(empty.is_empty());
    assert_eq!(empty.apply(0, 2, 3.5), 3.5);

    // Re-decorating a slot returns the previous Λ (the slot is a keyed cell, not a list).
    let mut store = ab.clone();
    let previous = store.insert(0, 2, lambda_negate);
    assert!(previous.is_some());
    assert_eq!(store.apply(0, 2, 3.0), -3.0);
}

/// THEOREM_MAP: core.causaloid.fixpoint
///
/// The Λ-decorated graph constructor rolls the Λ-edge store into the `Graph` summand and unroll
/// projects it back; an undecorated graph (no store, or an empty store) evaluates exactly as
/// before — the Stage-2 engine is untouched and the default Λ is the identity.
#[test]
fn test_undecorated_graph_evaluates_identically() {
    let g = Arc::new(deep_causality::utils_test::test_utils_graph::build_linear_graph(4));

    // Same frozen graph, evaluated directly: the baseline.
    let effect = PropagatingEffect::from_value(0.99f64);
    let baseline = g.evaluate_subgraph_from_cause(0, &effect);
    assert!(!baseline.is_err());

    // Wrapped undecorated vs wrapped with an empty Λ store: same graph Arc, same evaluation.
    let plain = Causaloid::from_causal_graph(20, "undecorated", g.clone());
    let decorated = Causaloid::from_causal_graph_with_lambda_edges(
        21,
        "identity-decorated",
        g.clone(),
        Arc::new(LambdaEdges::new()),
    );
    assert!(plain.graph_lambda_edges().is_none());
    let store = decorated
        .graph_lambda_edges()
        .expect("the Λ store projects back");
    assert!(store.is_empty());

    let r_plain = plain
        .causal_graph()
        .expect("graph")
        .evaluate_subgraph_from_cause(0, &effect);
    let r_decorated = decorated
        .causal_graph()
        .expect("graph")
        .evaluate_subgraph_from_cause(0, &effect);
    assert_eq!(baseline.value(), r_plain.value());
    assert_eq!(r_plain.value(), r_decorated.value());
}
