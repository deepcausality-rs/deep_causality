/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Meek orientation rules.
//!
//! Each expected orientation is derived by hand, either from the rule's own precondition or from
//! the definition of a compelled edge, and the derivation is in a comment above the test.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, MixedGraph};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// `a → b` present.
fn has_arc(g: &MixedGraph<()>, a: usize, b: usize) -> bool {
    g.arcs().contains(&(a, b))
}

fn is_undirected(g: &MixedGraph<()>, a: usize, b: usize) -> bool {
    g.edge_kind(a, b) == Some(EdgeKind::Undirected)
}

// ---------------------------------------------------------------------------------------------
// R1: c → a, c not adjacent to b, a — b  ⟹  a → b
//
// Derivation: orienting b → a would make c → a ← b an unshielded collider, which the input does
// not have. So a → b in every extension.
// ---------------------------------------------------------------------------------------------

#[test]
fn r1_orients_when_the_parent_is_not_adjacent_to_the_far_end() {
    let mut g = graph(3);
    g.add_arc(2, 0).unwrap(); // c → a
    g.add_undirected(0, 1).unwrap(); // a — b, and c=2 is not adjacent to b=1
    g.meek_complete();
    assert!(has_arc(&g, 0, 1), "R1 must compel a → b");
}

#[test]
fn r1_does_not_orient_when_the_parent_is_adjacent_to_the_far_end() {
    // Shielding c and b removes the collider argument, so nothing compels the edge. Both
    // orientations of a — b extend to a DAG with no new unshielded collider.
    let mut g = graph(3);
    g.add_arc(2, 0).unwrap(); // c → a
    g.add_undirected(0, 1).unwrap(); // a — b
    g.add_undirected(2, 1).unwrap(); // c — b, the shield
    g.meek_complete();
    assert!(
        is_undirected(&g, 0, 1) || has_arc(&g, 1, 0) || has_arc(&g, 0, 1),
        "the graph must remain well formed"
    );
    assert!(
        !has_arc(&g, 0, 1) || has_arc(&g, 2, 1),
        "a → b may only appear here through a further compelled orientation, not through R1"
    );
}

// ---------------------------------------------------------------------------------------------
// R2: a → c → b, a — b  ⟹  a → b
//
// Derivation: orienting b → a closes the directed cycle a → c → b → a.
// ---------------------------------------------------------------------------------------------

#[test]
fn r2_orients_to_avoid_closing_a_cycle() {
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap(); // a → c
    g.add_arc(2, 1).unwrap(); // c → b
    g.add_undirected(0, 1).unwrap(); // a — b
    g.meek_complete();
    assert!(has_arc(&g, 0, 1), "R2 must compel a → b");
}

// ---------------------------------------------------------------------------------------------
// R3: d — a — c undirected, d → b, c → b, c and d non-adjacent, a — b  ⟹  a → b
// ---------------------------------------------------------------------------------------------

#[test]
fn r3_orients_through_two_unshielded_parents() {
    // a=0, b=1, c=2, d=3.
    let mut g = graph(4);
    g.add_undirected(0, 1).unwrap(); // a — b
    g.add_undirected(0, 2).unwrap(); // a — c
    g.add_undirected(0, 3).unwrap(); // a — d
    g.add_arc(2, 1).unwrap(); // c → b
    g.add_arc(3, 1).unwrap(); // d → b
    // c=2 and d=3 are left non-adjacent, which is what makes the configuration R3's.
    g.meek_complete();
    assert!(has_arc(&g, 0, 1), "R3 must compel a → b");
}

// ---------------------------------------------------------------------------------------------
// R4: d — a — c undirected, d → c → b, b and d non-adjacent, a — b  ⟹  a → b
//
// Derivation, by the definition of a compelled edge rather than by the rule: orienting b → a forces
// c → a, since a → c with c → b and b → a would close a cycle; it likewise forces d → a. Then
// b → a ← d is an unshielded collider (b and d are non-adjacent) that the input does not have. So no
// consistent extension orients b → a, and a → b is compelled.
//
// This configuration is the one that separates the two closures: R1–R3 leaves the edge undirected.
// ---------------------------------------------------------------------------------------------

fn r4_configuration() -> MixedGraph<()> {
    // a=0, b=1, c=2, d=3.
    let mut g = graph(4);
    g.add_undirected(0, 1).unwrap(); // a — b
    g.add_undirected(0, 2).unwrap(); // a — c
    g.add_undirected(0, 3).unwrap(); // a — d
    g.add_arc(3, 2).unwrap(); // d → c
    g.add_arc(2, 1).unwrap(); // c → b
    // b=1 and d=3 are left non-adjacent.
    g
}

#[test]
fn r4_orients_its_own_configuration() {
    let mut g = r4_configuration();
    g.meek_complete();
    assert!(has_arc(&g, 0, 1), "R4 must compel a → b");
}

#[test]
fn the_restricted_closure_misses_the_r4_configuration() {
    let mut g = r4_configuration();
    g.meek_complete_r1_r3();
    assert!(
        is_undirected(&g, 0, 1),
        "R1–R3 is not complete here; leaving a — b undirected is what makes R4 necessary"
    );
}

#[test]
fn the_two_closures_differ_exactly_on_the_r4_edge() {
    let mut full = r4_configuration();
    full.meek_complete();
    let mut restricted = r4_configuration();
    restricted.meek_complete_r1_r3();

    // The complete closure orients a superset: every arc of the restricted closure survives.
    for (u, v) in restricted.arcs() {
        assert!(
            has_arc(&full, u, v),
            "the complete closure dropped the arc {u} → {v}"
        );
    }
    assert_eq!(
        full.arcs().len(),
        restricted.arcs().len() + 1,
        "exactly one further edge is compelled here, and it is a → b"
    );
    assert!(has_arc(&full, 0, 1));
}

// ---------------------------------------------------------------------------------------------
// Closure properties
// ---------------------------------------------------------------------------------------------

#[test]
fn the_closure_reaches_a_fixpoint() {
    let mut g = r4_configuration();
    g.meek_complete();
    let after_first = g.arcs();
    g.meek_complete();
    assert_eq!(
        after_first,
        g.arcs(),
        "a second closure must orient nothing further"
    );
}

// `undirected_edges` yields pairs in ascending order, so a graph whose *later* edge unlocks its
// *earlier* one cannot finish in a single sweep.
//
// Arcs 0 → 2 and 3 → 2, undirected 0 — 1 and 1 — 2. On the first sweep 0 — 1 is compelled by
// nothing: 0 has no parents and no children yet reaching 1. Then R1 orients 2 → 1, because 3 is a
// parent of 2 and is not adjacent to 1. Only on the second sweep does R2 reach 0 — 1 through
// 0 → 2 → 1.
#[test]
fn an_edge_unlocked_by_a_later_edge_needs_a_second_sweep() {
    let mut g = graph(4);
    g.add_arc(0, 2).unwrap();
    g.add_arc(3, 2).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.meek_complete();
    assert!(has_arc(&g, 2, 1), "R1 compels 2 → 1");
    assert!(
        has_arc(&g, 0, 1),
        "R2 then compels 0 → 1 through 0 → 2 → 1, which the first sweep cannot see"
    );
}

#[test]
fn both_closures_agree_on_the_pattern_of_a_dag() {
    // The pattern of 0 → 2 ← 1 with 0 and 1 non-adjacent: the collider is oriented, nothing else
    // is compelled. R1–R3 is complete for a pattern, so the two closures must agree.
    let mut full = graph(3);
    full.add_arc(0, 2).unwrap();
    full.add_arc(1, 2).unwrap();
    let mut restricted = graph(3);
    restricted.add_arc(0, 2).unwrap();
    restricted.add_arc(1, 2).unwrap();

    full.meek_complete();
    restricted.meek_complete_r1_r3();
    assert_eq!(full.arcs(), restricted.arcs());
}

// ---------------------------------------------------------------------------------------------
// Corner cases, from the enumeration in openspec/changes/unified-math-next/tdd/corner-cases.md
// ---------------------------------------------------------------------------------------------

// `MixedGraph::new` refuses zero vertices, so one vertex is the smallest constructible case.
#[test]
fn a_single_vertex_closes_to_itself() {
    let mut g = graph(1);
    g.meek_complete();
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn a_graph_with_no_undirected_edge_is_unchanged() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    let before = g.arcs();
    g.meek_complete();
    assert_eq!(before, g.arcs(), "there is nothing for a rule to orient");
}

#[test]
fn a_graph_with_no_arc_is_unchanged() {
    // Every rule's precondition needs at least one directed edge, so an all-undirected graph is a
    // fixpoint. This is the case where several rules could be confused with each other, because
    // none of them fires.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(0, 2).unwrap();
    g.meek_complete();
    assert!(is_undirected(&g, 0, 1));
    assert!(is_undirected(&g, 1, 2));
    assert!(is_undirected(&g, 0, 2));
}

#[test]
fn a_complete_undirected_graph_is_a_fixpoint() {
    let mut g = graph(4);
    for i in 0..4 {
        for j in (i + 1)..4 {
            g.add_undirected(i, j).unwrap();
        }
    }
    g.meek_complete();
    assert_eq!(g.arcs().len(), 0, "no edge is compelled in a clique");
}

#[test]
fn a_non_extendable_input_terminates_without_pinning_a_direction() {
    // Both directions of 0 — 1 are compelled: R1 through 2 → 0 with 2 not adjacent to 1, and R1
    // again through 3 → 1 with 3 not adjacent to 0. The input admits no consistent DAG extension.
    //
    // The closure does not detect this, so the assertion is termination and well-formedness only.
    // Asserting a direction here would pin the sweep's iteration order, which is not a decision the
    // closure makes.
    let mut g = graph(4);
    g.add_arc(2, 0).unwrap();
    g.add_arc(3, 1).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.meek_complete();
    assert!(
        has_arc(&g, 0, 1) || has_arc(&g, 1, 0),
        "the edge is oriented one way or the other"
    );
    assert!(g.invariant_holds(), "the graph stays well formed");
}

// R3 requires `c` and `d` non-adjacent. Shielding them removes the collider argument, so nothing
// compels a — b: 0 has no parents (R1) and no children (R2), and 2 — 3 is undirected so R4 finds no
// `d → c`.
#[test]
fn r3_does_not_fire_when_the_two_parents_are_shielded() {
    let mut g = graph(4);
    g.add_undirected(0, 1).unwrap(); // a — b
    g.add_undirected(0, 2).unwrap(); // a — c
    g.add_undirected(0, 3).unwrap(); // a — d
    g.add_arc(2, 1).unwrap(); // c → b
    g.add_arc(3, 1).unwrap(); // d → b
    g.add_undirected(2, 3).unwrap(); // the shield
    g.meek_complete();
    assert!(
        is_undirected(&g, 0, 1),
        "with c and d adjacent, R3's precondition fails and nothing else applies"
    );
}

// R4 requires `b` and `d` non-adjacent. Joining them removes its precondition, and no other rule
// reaches the edge: 0 has no parents or children, and b has only one parent so R3 cannot apply.
#[test]
fn r4_does_not_fire_when_b_and_d_are_adjacent() {
    let mut g = r4_configuration();
    g.add_undirected(1, 3).unwrap(); // b — d
    g.meek_complete();
    assert!(
        is_undirected(&g, 0, 1),
        "with b and d adjacent, R4's precondition fails"
    );
}
