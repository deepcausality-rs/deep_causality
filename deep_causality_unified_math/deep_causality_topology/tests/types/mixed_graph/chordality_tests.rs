/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Chordality of the undirected projection.
//!
//! Each expectation is read off the definition — no induced cycle of length four or more — for a
//! graph small enough to check by eye.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// `n` vertices joined in a cycle `0 — 1 — … — n-1 — 0`, with no chords.
fn chordless_cycle(n: usize) -> MixedGraph<()> {
    let mut g = graph(n);
    for i in 0..n {
        g.add_undirected(i, (i + 1) % n).unwrap();
    }
    g
}

#[test]
fn a_triangle_is_chordal() {
    // A 3-cycle has no cycle of length four or more, so the condition holds vacuously.
    let g = chordless_cycle(3);
    assert!(g.undirected_projection_is_chordal().is_ok());
    assert_eq!(g.find_chordless_cycle(), None);
}

#[test]
fn a_four_cycle_is_not_chordal() {
    let g = chordless_cycle(4);
    assert!(g.undirected_projection_is_chordal().is_err());
    let witness = g
        .find_chordless_cycle()
        .expect("the 4-cycle is the witness");
    assert_eq!(witness.len(), 4);
}

#[test]
fn a_five_cycle_is_not_chordal() {
    let g = chordless_cycle(5);
    assert!(g.undirected_projection_is_chordal().is_err());
    assert_eq!(g.find_chordless_cycle().map(|c| c.len()), Some(5));
}

#[test]
fn a_four_cycle_with_a_chord_is_chordal() {
    // 0-1-2-3-0 plus the chord 0-2 splits it into two triangles.
    let mut g = chordless_cycle(4);
    g.add_undirected(0, 2).unwrap();
    assert!(g.undirected_projection_is_chordal().is_ok());
    assert_eq!(g.find_chordless_cycle(), None);
}

#[test]
fn the_error_names_the_offending_cycle() {
    let g = chordless_cycle(4);
    let err = g.undirected_projection_is_chordal().unwrap_err();
    let text = format!("{err:?}");
    assert!(
        ["0", "1", "2", "3"].iter().all(|v| text.contains(v)),
        "the error must name the cycle's vertices, got: {text}"
    );
}

#[test]
fn directed_edges_take_no_part() {
    // The same four vertices, but the cycle is directed. The undirected projection is empty, so
    // the condition holds. This is the case where the two projections would be confused.
    let mut g = graph(4);
    for i in 0..4 {
        g.add_arc(i, (i + 1) % 4).unwrap();
    }
    assert!(g.undirected_projection_is_chordal().is_ok());
    assert_eq!(g.find_chordless_cycle(), None);
}

#[test]
fn a_directed_chord_does_not_repair_an_undirected_cycle() {
    // A chord that is directed is not in the projection, so the 4-cycle is still induced there.
    let mut g = chordless_cycle(4);
    g.add_arc(0, 2).unwrap();
    assert!(g.undirected_projection_is_chordal().is_err());
}

#[test]
fn two_components_are_both_checked() {
    // A chordal triangle on 0,1,2 and a chordless 4-cycle on 3,4,5,6. The failure must be found
    // even though the first component is fine.
    let mut g = graph(7);
    for (a, b) in [(0, 1), (1, 2), (0, 2)] {
        g.add_undirected(a, b).unwrap();
    }
    for (a, b) in [(3, 4), (4, 5), (5, 6), (6, 3)] {
        g.add_undirected(a, b).unwrap();
    }
    assert!(g.undirected_projection_is_chordal().is_err());
    let witness = g.find_chordless_cycle().expect("the second component");
    assert!(witness.iter().all(|&v| (3..=6).contains(&v)));
}

// `MixedGraph::new` refuses zero vertices, so the empty graph is not constructible and the
// smallest case is one vertex.
#[test]
fn a_single_vertex_is_chordal() {
    let g = graph(1);
    assert!(g.undirected_projection_is_chordal().is_ok());
}

#[test]
fn a_tree_is_chordal() {
    // No cycles at all.
    let mut g = graph(4);
    for (a, b) in [(0, 1), (0, 2), (2, 3)] {
        g.add_undirected(a, b).unwrap();
    }
    assert!(g.undirected_projection_is_chordal().is_ok());
}

#[test]
fn a_complete_graph_is_chordal() {
    // Every cycle of length four or more has every possible chord.
    let mut g = graph(5);
    for i in 0..5 {
        for j in (i + 1)..5 {
            g.add_undirected(i, j).unwrap();
        }
    }
    assert!(g.undirected_projection_is_chordal().is_ok());
}

#[test]
fn two_directed_diagonals_do_not_repair_an_undirected_four_cycle() {
    // The undirected projection of 0 — 1 — 2 — 3 — 0 with both diagonals directed (0 → 2 and
    // 1 → 3) is still the chordless 4-cycle: a directed edge is not a chord of the projection.
    let mut g = chordless_cycle(4);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 3).unwrap();
    assert!(g.undirected_projection_is_chordal().is_err());
    let witness = g
        .find_chordless_cycle()
        .expect("the projection is still the 4-cycle 0-1-2-3");
    assert_eq!(witness.len(), 4);
}

#[test]
fn a_bidirected_diagonal_is_not_a_chord() {
    // Same shape, with the diagonals bidirected instead of directed. Neither kind is in the
    // undirected projection, so the 4-cycle remains chordless there.
    let mut g = chordless_cycle(4);
    g.add_bidirected(0, 2).unwrap();
    g.add_bidirected(1, 3).unwrap();
    assert!(g.undirected_projection_is_chordal().is_err());
    assert_eq!(g.find_chordless_cycle().map(|c| c.len()), Some(4));
}
