/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Rust witnesses for `lean/DeepCausalityFormal/Core/ContextGraph.lean` — the Context hypergraph:
//! parent-set (hyperedge) semantics keyed by identity, hyperedge threading = the causal monad
//! `bind` (so encapsulation = flat, inherited from `core.causal_monad.assoc`), and acyclicity as a
//! SEPARABLE, freeze-enforceable parameter. Lean proves the laws ∀; these tests pin the real
//! `Context` graph, the real `CausalEffectPropagationProcess::bind`, and the real `freeze_dag`
//! acyclicity gate to the same statements.

use deep_causality::utils_test::test_utils;
use deep_causality::{
    BaseContext, CausableGraph, CausalEffect, CausaloidGraph, Context, Contextoid, ContextoidType,
    ContextuableGraph, PropagatingEffect, RelationKind, Root,
};

/// THEOREM_MAP: core.context_graph.threading_bind
///
/// Lean: the `Pa` parent-set map keyed by identity (`Core/ContextGraph.lean`). A contextoid's
/// parents are named by IDENTITY (`fired[child][parent]` wire slots / `LambdaEdges` `(source,
/// target)` keys), so overlapping parent sets are expressible: here one root (id 10) is a parent of
/// two distinct children (ids 20, 30), and identity → index resolution round-trips.
#[test]
fn test_context_parent_set_keyed_by_identity() {
    let mut ctx: BaseContext = Context::with_capacity(1, "hypergraph", 8);

    let root = ctx
        .add_node(Contextoid::new(10, ContextoidType::Root(Root::new(10))))
        .expect("add root");
    let child_a = ctx
        .add_node(Contextoid::new(20, ContextoidType::Root(Root::new(20))))
        .expect("add child a");
    let child_b = ctx
        .add_node(Contextoid::new(30, ContextoidType::Root(Root::new(30))))
        .expect("add child b");

    // Hyperedges {10} -> 20 and {10} -> 30: the shared parent is an OVERLAPPING parent set.
    ctx.add_edge(root, child_a, RelationKind::Temporal)
        .expect("edge 10->20");
    ctx.add_edge(root, child_b, RelationKind::Temporal)
        .expect("edge 10->30");

    // Parents keyed by identity: the external u64 id resolves to its internal index.
    assert_eq!(ctx.get_node_index_by_id(20), Some(child_a));
    assert_eq!(ctx.get_node_index_by_id(30), Some(child_b));

    // The same parent (id 10) is in both children's parent sets — overlap is expressible.
    assert!(ctx.contains_edge(root, child_a));
    assert!(ctx.contains_edge(root, child_b));
    // ...and it is a directed hyperedge: no reverse edge was created.
    assert!(!ctx.contains_edge(child_a, root));
}

// Two contextoid mechanisms, as fn items (Copy — reusable on both sides of the assoc law).
fn mech_add_one(eff: CausalEffect<i64>, _s: (), _c: Option<()>) -> PropagatingEffect<i64> {
    PropagatingEffect::pure(eff.into_value().unwrap_or(0) + 1)
}
fn mech_double(eff: CausalEffect<i64>, _s: (), _c: Option<()>) -> PropagatingEffect<i64> {
    PropagatingEffect::pure(eff.into_value().unwrap_or(0) * 2)
}

/// THEOREM_MAP: core.context_graph.threading_bind
///
/// Lean: `thread_is_bind`, `encapsulation_flat` (`Core/ContextGraph.lean`). Hyperedge threading of a
/// node's parents IS the causal monad `bind`, and "encapsulate a sub-hypergraph as one contextoid
/// vs inline it" is exactly bind ASSOCIATIVITY (`core.causal_monad.assoc`): threading the running
/// value through `f` then `g` equals binding it once through the encapsulated composite
/// `|x| bind (f x) g`. Checked on the real `CausalEffectPropagationProcess::bind`.
#[test]
fn test_context_encapsulation_is_bind_assoc() {
    for seed in [0i64, 3, -7, 42] {
        let m = PropagatingEffect::pure(seed);

        // Flat: thread through the two parent mechanisms in sequence (bind then bind).
        let flat = m.clone().bind(mech_add_one).bind(mech_double);

        // Encapsulated: bind once through the composite mechanism `|x| bind (f x) g` — the wrapped
        // sub-hypergraph. Associativity (core.causal_monad.assoc) makes the two equal.
        let encapsulated = m.bind(|eff, s, c| mech_add_one(eff, s, c).bind(mech_double));

        assert_eq!(
            flat, encapsulated,
            "encapsulation != flat at seed {seed}: bind associativity violated"
        );
        // Sanity: the shared value is (seed + 1) * 2, so threading really composes the mechanisms.
        assert_eq!(flat.value(), Some(&((seed + 1) * 2)));
    }
}

/// THEOREM_MAP: core.context_graph.acyclicity_separable
///
/// Lean: `self_parent_not_acyclic`, `apparatus_acyclicity_agnostic` (`Core/ContextGraph.lean`).
/// Acyclicity is a SEPARABLE, freeze-enforceable parameter over the SAME parent-set graph: the
/// acyclic case passes `freeze_dag` (the rank certificate exists — `ultragraph::has_cycle` is
/// false); a directed cycle is rejected at `freeze_dag` and rolled back; yet the plain `freeze`
/// accepts the very same cyclic graph — the definitions are unchanged, only the acyclicity GATE
/// differs (enabling the deferred cyclic case: quantum switch / indefinite causal order).
#[test]
fn test_context_acyclicity_freeze_gate() {
    // Acyclic parent-set a -> b: freeze_dag succeeds (a rank certificate exists).
    let mut acyclic = CausaloidGraph::new(0);
    let a = acyclic
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("add a");
    let b = acyclic
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("add b");
    acyclic.add_edge(a, b).expect("edge a->b");
    assert!(
        acyclic.freeze_dag().is_ok(),
        "acyclic graph must freeze as a DAG"
    );
    assert!(acyclic.is_frozen());

    // The SAME apparatus with a directed cycle a -> b -> a: freeze_dag rejects it (has_cycle),
    // rolling the graph back to unfrozen.
    let mut cyclic = CausaloidGraph::new(0);
    let a = cyclic
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("add a");
    let b = cyclic
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("add b");
    cyclic.add_edge(a, b).expect("edge a->b");
    cyclic.add_edge(b, a).expect("edge b->a");

    let rejected = cyclic.freeze_dag();
    assert!(
        rejected.is_err(),
        "a directed cycle must be rejected at freeze_dag"
    );
    assert!(
        rejected.unwrap_err().to_string().contains("cycle"),
        "the freeze-gate error must name the cycle"
    );
    assert!(
        !cyclic.is_frozen(),
        "rejected graph is rolled back to unfrozen"
    );

    // Acyclicity is SEPARABLE: the plain `freeze` accepts the identical cyclic graph — the parent-set
    // definitions are unchanged; only the acyclicity gate (`freeze_dag`) enforces the DAG constraint.
    cyclic.freeze();
    assert!(
        cyclic.is_frozen(),
        "the same cyclic graph is admissible without the DAG gate"
    );
}
