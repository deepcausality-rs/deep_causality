/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validation of the Clique-Picking AMO counter against the exact enumeration
//! oracle `brcd::brcd_mec::mec_size`.
//!
//! The counter is the polynomial-time Clique-Picking port; the oracle enumerates
//! AMOs exactly (capped at `MEC_ENUM_BOUND`). For an all-undirected chordal
//! `MixedGraph` they must agree exactly. These tests assert round-equality
//! (within 0.5, since both are integers) on the author's hard-coded anchors (54
//! and 108), a few hand-built chordal graphs, and at least 2000 random connected
//! chordal graphs (`n` in 3..=10) whose class fits under the oracle's enumeration
//! bound. A `Float106` instantiation is checked against the same oracle.

use deep_causality_algorithms::brcd::brcd_mec::{MEC_ENUM_BOUND, mec_size as oracle_mec_size};
use deep_causality_algorithms::dag_sampling::{
    Graph, count_amos, count_chordal, mec_size as cp_mec_size,
};
use deep_causality_num::Float106;
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

/// Builds an all-undirected `MixedGraph<()>` on `n` vertices from `edges`.
fn undirected_graph(n: usize, edges: &[(usize, usize)]) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    let mut g = MixedGraph::new(n, data, 0).unwrap();
    for &(a, b) in edges {
        g.add_undirected(a, b).unwrap();
    }
    g
}

/// Rounds a count held as `f64` to the nearest integer.
fn round(x: f64) -> i128 {
    x.round() as i128
}

// --- hard-coded author anchors ---------------------------------------------

#[test]
fn anchor_six_node_graph_counts_54() {
    // The author's 6-node anchor: a single connected chordal component with 54 AMOs.
    let edges = [
        (0, 1),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
        (3, 4),
        (4, 5),
    ];
    let g = undirected_graph(6, &edges);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 54);
    assert_eq!(oracle_mec_size(&g), Ok(54));
}

#[test]
fn anchor_nine_node_two_components_counts_108() {
    // The 6-node anchor plus a disjoint edge (7, 8) on 9 nodes: 54 * 2 = 108.
    let edges = [
        (0, 1),
        (0, 2),
        (1, 2),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
        (3, 4),
        (4, 5),
        (7, 8),
    ];
    let g = undirected_graph(9, &edges);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 108);
    assert_eq!(oracle_mec_size(&g), Ok(108));
}

// --- small deterministic chordal graphs ------------------------------------

#[test]
fn empty_graph_counts_one() {
    let g = undirected_graph(3, &[]);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 1);
    assert_eq!(oracle_mec_size(&g), Ok(1));
}

#[test]
fn single_edge_counts_two() {
    let g = undirected_graph(2, &[(0, 1)]);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 2);
    assert_eq!(oracle_mec_size(&g), Ok(2));
}

#[test]
fn path_of_three_counts_three() {
    let g = undirected_graph(3, &[(0, 1), (1, 2)]);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 3);
    assert_eq!(oracle_mec_size(&g), Ok(3));
}

#[test]
fn triangle_counts_six() {
    let g = undirected_graph(3, &[(0, 1), (1, 2), (0, 2)]);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 6);
    assert_eq!(oracle_mec_size(&g), Ok(6));
}

#[test]
fn complete_four_counts_twentyfour() {
    // K4 is chordal; 4! = 24 acyclic moral orientations.
    let g = undirected_graph(4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 24);
    assert_eq!(oracle_mec_size(&g), Ok(24));
}

#[test]
fn complete_five_counts_onetwenty() {
    // K5: 5! = 120 (exercises the num_possible_edges special case).
    let edges: Vec<(usize, usize)> = (0..5)
        .flat_map(|a| ((a + 1)..5).map(move |b| (a, b)))
        .collect();
    let g = undirected_graph(5, &edges);
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(round(cp), 120);
    assert_eq!(oracle_mec_size(&g), Ok(120));
}

#[test]
fn two_triangles_sharing_an_edge() {
    // Chordal: triangles {0,1,2} and {1,2,3} share edge 1-2. AMO count is 8.
    let edges = [(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)];
    let g = undirected_graph(4, &edges);
    let cp: f64 = cp_mec_size(&g);
    let oracle = oracle_mec_size(&g).unwrap();
    assert_eq!(round(cp), oracle as i128);
}

// --- direct count_amos / count_chordal API (public Graph input) ------------

#[test]
fn count_amos_direct_on_anchor() {
    // Single connected chordal component via the public Graph + count_amos path.
    let g = Graph::from_edge_list(
        vec![
            (0, 1),
            (0, 2),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
            (3, 4),
            (4, 5),
        ],
        6,
    );
    let cp: f64 = count_amos(&g);
    assert_eq!(round(cp), 54);
}

#[test]
fn count_chordal_direct_multi_component() {
    // count_chordal multiplies over components: anchor (54) x edge (2) = 108.
    let g = Graph::from_edge_list(
        vec![
            (0, 1),
            (0, 2),
            (1, 2),
            (1, 3),
            (1, 4),
            (1, 5),
            (2, 3),
            (2, 4),
            (2, 5),
            (3, 4),
            (4, 5),
            (7, 8),
        ],
        9,
    );
    let cp: f64 = count_chordal(&g);
    assert_eq!(round(cp), 108);
}

#[test]
fn count_amos_from_adjacency_list() {
    // K4 via the adjacency-list constructor: 4! = 24.
    let g = Graph::from_adjacency_list(vec![
        vec![1, 2, 3],
        vec![0, 2, 3],
        vec![0, 1, 3],
        vec![0, 1, 2],
    ]);
    let cp: f64 = count_amos(&g);
    assert_eq!(round(cp), 24);
}

// --- random connected chordal graphs vs the oracle -------------------------

/// Generates the edge list of a random connected chordal graph on `n` vertices.
///
/// Chordality is guaranteed by random elimination: vertices are added one at a
/// time, each new vertex made adjacent to a random clique among the already-added
/// vertices (a subset of one existing maximal-clique candidate). Connectivity is
/// guaranteed by always attaching the new vertex to at least one prior vertex.
fn random_connected_chordal(rng: &mut Xoshiro256, n: usize) -> Vec<(usize, usize)> {
    // `cliques` holds vertex sets known to be cliques; we attach each new vertex
    // to a random subset (of size >= 1) of a randomly chosen existing clique,
    // which keeps the closed neighborhood a clique => chordal.
    let mut edges: Vec<(usize, usize)> = Vec::new();
    // Maximal-clique-ish candidates; start with the singleton {0}.
    let mut cliques: Vec<Vec<usize>> = vec![vec![0]];

    for v in 1..n {
        // Pick an existing clique to attach to.
        let clique_idx: usize = rng.random_range(0..cliques.len());
        let base = cliques[clique_idx].clone();
        // Choose how many of its vertices v connects to: at least 1, at most all.
        let k: usize = 1 + rng.random_range(0..base.len());
        // Random subset of `base` of size k (simple partial Fisher-Yates).
        let mut pool = base.clone();
        let mut chosen = Vec::with_capacity(k);
        for i in 0..k {
            let j: usize = rng.random_range(i..pool.len());
            pool.swap(i, j);
            chosen.push(pool[i]);
        }
        for &u in &chosen {
            edges.push((u.min(v), u.max(v)));
        }
        // The new closed neighborhood {chosen ∪ v} is a clique.
        let mut new_clique = chosen.clone();
        new_clique.push(v);
        cliques.push(new_clique);
    }
    edges
}

#[test]
fn random_chordal_graphs_match_oracle() {
    let mut rng = Xoshiro256::from_seed(0xC119_E0FA_BCD1_2345);
    let target = 200usize;
    let mut checked = 0usize;
    let mut attempts = 0usize;
    let mut max_n = 0usize;
    let mut max_count: i128 = 0;

    while checked < target {
        attempts += 1;
        assert!(
            attempts < 200_000,
            "could not generate {target} usable graphs"
        );
        let n: usize = 3 + rng.random_range(0usize..8); // n in 3..=10
        let edges = random_connected_chordal(&mut rng, n);
        let g = undirected_graph(n, &edges);

        // Skip graphs whose class would cap the oracle.
        let oracle = match oracle_mec_size(&g) {
            Ok(size) => size,
            Err(_) => continue, // ClassTooLarge: regenerate
        };
        // Defensive: oracle never returns above the bound, but be explicit.
        if oracle > MEC_ENUM_BOUND {
            continue;
        }

        let cp: f64 = cp_mec_size(&g);
        assert_eq!(
            round(cp),
            oracle as i128,
            "mismatch on n={n} edges={edges:?}: clique-picking={cp} oracle={oracle}"
        );

        checked += 1;
        max_n = max_n.max(n);
        max_count = max_count.max(oracle as i128);
    }

    assert_eq!(checked, target);
    // Sanity: the fuzz run actually reached the larger sizes and non-trivial counts.
    assert!(max_n >= 8, "fuzzing never reached n>=8 (max_n={max_n})");
    assert!(
        max_count >= 24,
        "fuzzing never produced a non-trivial class (max_count={max_count})"
    );
}

// --- Float106 instantiation -------------------------------------------------

/// Reads the high `f64` limb of a `Float106` for comparison against the oracle.
fn f106_round(x: Float106) -> i128 {
    x.to_f64().round() as i128
}

#[test]
fn float106_matches_f64_and_oracle() {
    let cases: &[(usize, &[(usize, usize)])] = &[
        // K4
        (4, &[(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)]),
        // path of three
        (3, &[(0, 1), (1, 2)]),
        // the 6-node anchor
        (
            6,
            &[
                (0, 1),
                (0, 2),
                (1, 2),
                (1, 3),
                (1, 4),
                (1, 5),
                (2, 3),
                (2, 4),
                (2, 5),
                (3, 4),
                (4, 5),
            ],
        ),
        // two-triangle book
        (4, &[(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)]),
    ];

    for &(n, edges) in cases {
        let g = undirected_graph(n, edges);
        let oracle = oracle_mec_size(&g).unwrap() as i128;
        let cp_f64: f64 = cp_mec_size(&g);
        let cp_f106: Float106 = cp_mec_size(&g);
        assert_eq!(
            round(cp_f64),
            oracle,
            "f64 mismatch on n={n} edges={edges:?}"
        );
        assert_eq!(
            f106_round(cp_f106),
            oracle,
            "Float106 mismatch on n={n} edges={edges:?}"
        );
    }
}

/// Exercises the `count_amos` closed-form special case for a connected chordal
/// graph with `m == C(n, 2) - 2` (the complete graph minus two adjacent edges).
///
/// `K5` minus `(0,1)` and `(0,2)` is chordal (vertex 0 is simplicial: its
/// neighbours `{3,4}` are adjacent) and has `m = 10 - 2 = 8 != n, n-1`, so it
/// reaches the dedicated `num_possible_edges - 2` branch rather than the generic
/// clique-tree path. He–Jia–Yu Thm 3 gives `(n(n-1) - 4)·(n-3)! = 16·2 = 32`; the
/// enumeration oracle cross-checks it.
#[test]
fn closed_form_complete_minus_two_adjacent_edges() {
    let edges = &[
        (0, 3),
        (0, 4),
        (1, 2),
        (1, 3),
        (1, 4),
        (2, 3),
        (2, 4),
        (3, 4),
    ];
    let g = undirected_graph(5, edges);
    let oracle = oracle_mec_size(&g).unwrap();
    assert_eq!(
        oracle, 32,
        "enumeration oracle for K5 minus two adjacent edges"
    );
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(
        round(cp),
        oracle as i128,
        "clique-picking must match the oracle"
    );
}
