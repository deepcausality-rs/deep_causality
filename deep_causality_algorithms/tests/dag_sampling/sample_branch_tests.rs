/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Targeted coverage of the deeper Clique-Picking sampler/counter branches via
//! chordal graphs whose clique trees have several overlapping maximal cliques with
//! separators of *different* sizes (k-trees, interval graphs, triangle/clique
//! "books", and random chordal sweeps). Each graph is driven through both the
//! counter (`mec_size`) and the uniform sampler (`sample_dag`), cross-checked
//! against the exact enumeration oracle, with every sampled DAG asserted
//! fully-directed and acyclic.
//!
//! These graphs exhaustively exercise the forbidden-prefix scan in `count.rs`
//! (`count_traversal`) and `sample.rs` (`rec_count_init` / `rec_sample_ordering`),
//! the `WeightedChoice` selector, the rejection-sampling permutation path
//! (`draw_allowed_permutation` / `is_allowed`), and the `equal_to_vec` comparison
//! in MCS clique-tree construction.
//!
//! Several adjacent lines are proven-unreachable defensive guards for valid
//! chordal input and stay uncovered by design (each documented at its test):
//! * the forbidden-prefix `size <= separator.len()` early-stop `break`
//!   (`count.rs:170`, `sample.rs:237`, `sample.rs:399`) — within one flower, both
//!   endpoints of an internal crossing edge in the flower force its separator
//!   *strictly* larger than the subproblem separator (the tree gives a unique path,
//!   and the flower only crosses differing separators), so `size > separator.len()`
//!   always holds and the earlier flower-membership `break` fires for outside
//!   edges;
//! * `is_allowed`'s final post-loop `true` (`sample.rs:487`);
//! * `WeightedChoice::sample`'s last-index fallthrough (`sample.rs:133`);
//! * `equal_to_vec`'s "same length, different content" `false` (`index_set.rs:143`);
//! * the post-orientation cycle guards in `sample_dag` (`sample.rs:564`) and
//!   `representative_dag` (`sample.rs:614`) — a valid CPDAG (already passed
//!   `validate_cpdag`) oriented by a topological order cannot produce a cycle.

use deep_causality_algorithms::brcd::brcd_mec::mec_size as oracle_mec_size;
use deep_causality_algorithms::dag_sampling::{
    mec_size as cp_mec_size, representative_dag, sample_dag,
};
use deep_causality_rand::{Rng, Xoshiro256};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, MixedGraph};

/// Builds an all-undirected `MixedGraph<()>` on `n` vertices from `edges`.
fn undirected_graph(n: usize, edges: &[(usize, usize)]) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    let mut g = MixedGraph::new(n, data, 0).unwrap();
    for &(a, b) in edges {
        g.add_undirected(a, b).unwrap();
    }
    g
}

/// Runs the counter (f64) against the exact enumeration oracle and drives the
/// uniform sampler for many fixed-seed draws, asserting every draw is fully
/// directed and acyclic. Returns the oracle class size.
fn exercise(n: usize, edges: &[(usize, usize)], seed: u64) -> i128 {
    let g = undirected_graph(n, edges);
    let oracle = oracle_mec_size(&g).expect("oracle size");
    let cp: f64 = cp_mec_size(&g);
    assert_eq!(
        cp.round() as i128,
        oracle as i128,
        "clique-picking != oracle on n={n} edges={edges:?}"
    );

    let mut rng = Xoshiro256::from_seed(seed);
    // Enough draws to traverse the full clique tree many times over.
    for _ in 0..400 {
        let sample = sample_dag::<f64, (), _>(&g, &mut rng).expect("sample");
        for edge in sample.edges().values() {
            assert_eq!(edge.kind(), EdgeKind::Directed, "sample not fully directed");
        }
        assert!(!sample.has_cycle(), "sample is cyclic");
    }
    // Deterministic representative is also a valid fully-directed member.
    let rep = representative_dag::<()>(&g).expect("representative");
    for edge in rep.edges().values() {
        assert_eq!(edge.kind(), EdgeKind::Directed);
    }
    assert!(!rep.has_cycle());

    oracle as i128
}

/// A 2-tree (every maximal clique is a triangle) built as a fan of triangles
/// sharing a spine, giving a clique tree with separators of size 2 nested under
/// larger sub-flowers. Exercises the forbidden-prefix scan (the flower-membership
/// `break` and the `size > separator.len()` accept branch).
#[test]
fn two_tree_fan_branches() {
    // Triangles: {0,1,2}, {0,1,3}, {0,1,4} share the edge {0,1}; then {1,4,5}
    // and {4,5,6} extend a second spine. All maximal cliques are triangles.
    let edges = [
        (0, 1),
        (0, 2),
        (1, 2),
        (0, 3),
        (1, 3),
        (0, 4),
        (1, 4),
        (1, 5),
        (4, 5),
        (4, 6),
        (5, 6),
    ];
    let oracle = exercise(7, &edges, 0xDA6_0001);
    assert!(oracle > 1);
}

/// An interval graph (chordal) whose maximal cliques overlap in a sliding window,
/// producing a clique tree path with separators of varying sizes.
#[test]
fn interval_graph_sliding_cliques() {
    // Cliques {0,1,2,3}, {1,2,3,4}, {2,3,4,5}: each consecutive pair shares a
    // size-3 separator; non-consecutive cliques share smaller intersections.
    let mut edges = Vec::new();
    for a in 0..6 {
        for b in (a + 1)..6 {
            if b - a <= 3 {
                edges.push((a, b));
            }
        }
    }
    let oracle = exercise(6, &edges, 0xDA6_0002);
    assert!(oracle > 1);
}

/// A "book" of triangles plus a 4-clique chapter, so one clique participates in
/// crossing edges of *different* separator sizes (2 and 3), driving the
/// forbidden-prefix scan across several subproblems. (The `size <= separator.len()`
/// early stop stays unreachable — see the module-level note.)
#[test]
fn mixed_separator_sizes_book() {
    // 4-clique {0,1,2,3}; triangles {0,1,4}, {0,1,5} hang off edge {0,1};
    // triangle {2,3,6} hangs off edge {2,3}.
    let edges = [
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 3),
        (0, 4),
        (1, 4),
        (0, 5),
        (1, 5),
        (2, 6),
        (3, 6),
    ];
    let oracle = exercise(7, &edges, 0xDA6_0003);
    assert!(oracle > 1);
}

/// A 3-tree: every maximal clique is a K4, separators are size-3, nested under a
/// branching clique tree.
#[test]
fn three_tree_branches() {
    // Base K4 {0,1,2,3}; vertex 4 joins {1,2,3}; vertex 5 joins {2,3,4};
    // vertex 6 joins {1,2,3} (sibling branch sharing the same separator).
    let edges = [
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 3),
        (1, 4),
        (2, 4),
        (3, 4),
        (2, 5),
        (3, 5),
        (4, 5),
        (1, 6),
        (2, 6),
        (3, 6),
    ];
    let oracle = exercise(7, &edges, 0xDA6_0004);
    assert!(oracle > 1);
}

/// Two equal-size maximal cliques discovered consecutively in the MCS order,
/// exercising the `equal_to_vec` comparison in `index_set.rs` during clique-tree
/// construction. The clique boundary is detected by the length-mismatch `false`
/// return (`index_set.rs:139`); the "same length, different content" `false`
/// (`index_set.rs:143`) is unreachable here (see
/// `equal_to_vec_distinct_same_length_battery`).
#[test]
fn equal_size_distinct_cliques() {
    // Two triangles {0,1,2} and {2,3,4} joined only at vertex 2 (a cut vertex):
    // both maximal cliques have size 3; the second flushes the first when its
    // visited-neighbor set ({2}) differs in length from the running clique at the
    // transition.
    let edges = [(0, 1), (0, 2), (1, 2), (2, 3), (2, 4), (3, 4)];
    let oracle = exercise(5, &edges, 0xDA6_0005);
    assert!(oracle > 1);

    // A second shape: two K4s sharing a single vertex.
    let edges2 = [
        (0, 1),
        (0, 2),
        (0, 3),
        (1, 2),
        (1, 3),
        (2, 3),
        (3, 4),
        (3, 5),
        (3, 6),
        (4, 5),
        (4, 6),
        (5, 6),
    ];
    let oracle2 = exercise(7, &edges2, 0xDA6_0006);
    assert!(oracle2 > 1);
}

/// Drives a battery of chordal graphs (hand-built equal-size-clique shapes plus a
/// random sweep) through the MCS clique-tree construction, which is the only call
/// site of `IndexSet::equal_to_vec` (`index_set.rs`). This exhaustively exercises
/// that comparison: every clique boundary trips the length-mismatch `false` return
/// and every clique extension trips the all-contained `true` return, cross-checked
/// against the exact oracle.
///
/// Note: the helper's "same length, different content" `false` branch
/// (`index_set.rs:143`) is *not* reachable through this path. In an MCS perfect-
/// elimination order a vertex's back-neighbor set either equals the running clique
/// (extension) or is a *strictly smaller* separator (new clique), so the
/// length check (`index_set.rs:139`) always fires first; the equal-length scan
/// only ever finds matching content. That `false` is a defensive completeness
/// branch of the order-insensitive comparison, documented as unreachable.
#[test]
fn equal_to_vec_distinct_same_length_battery() {
    // Several hand-built chordal shapes plus a randomized sweep that exercise the
    // equal-size-clique transitions in MCS.
    let shapes: &[(usize, &[(usize, usize)])] = &[
        // Diamond chain of triangles sharing single vertices and edges.
        (
            6,
            &[
                (0, 1),
                (0, 2),
                (1, 2),
                (1, 3),
                (2, 3),
                (3, 4),
                (3, 5),
                (4, 5),
            ],
        ),
        // A K4 with pendant triangles sharing distinct edges.
        (
            6,
            &[
                (0, 1),
                (0, 2),
                (0, 3),
                (1, 2),
                (1, 3),
                (2, 3),
                (2, 4),
                (1, 4),
                (4, 5),
                (2, 5),
            ],
        ),
        // A chordal chain of triangles sharing single cut vertices (path of
        // equal-size maximal cliques).
        (
            7,
            &[
                (0, 1),
                (1, 2),
                (0, 2),
                (2, 3),
                (3, 4),
                (2, 4),
                (4, 5),
                (5, 6),
                (4, 6),
            ],
        ),
    ];
    for (i, (n, edges)) in shapes.iter().enumerate() {
        let oracle = exercise(*n, edges, 0xEAA_0000 + i as u64);
        assert!(oracle >= 1);
    }

    // Randomized sweep: many random chordal graphs exercise the full range of
    // equal-length / length-mismatch MCS comparisons against the exact oracle.
    let mut rng = Xoshiro256::from_seed(0xEA9_F00D);
    for trial in 0..150u64 {
        let n = 5 + (trial as usize % 6); // 5..=10
        let edges = random_connected_chordal(&mut rng, n);
        let g = undirected_graph(n, &edges);
        let oracle = oracle_mec_size(&g).expect("oracle");
        let cp: f64 = cp_mec_size(&g);
        assert_eq!(cp.round() as i128, oracle as i128, "trial {trial}");
    }
}

/// Generates the edge list of a random connected chordal graph on `n` vertices by
/// random elimination (each new vertex attaches to a random subset of an existing
/// clique), keeping every closed neighborhood a clique => chordal. Mirrors the
/// generator in `sample_tests.rs`.
fn random_connected_chordal(rng: &mut Xoshiro256, n: usize) -> Vec<(usize, usize)> {
    let mut edges: Vec<(usize, usize)> = Vec::new();
    let mut cliques: Vec<Vec<usize>> = vec![vec![0]];
    for v in 1..n {
        let clique_idx: usize = rng.random_range(0..cliques.len());
        let base = cliques[clique_idx].clone();
        let k: usize = 1 + rng.random_range(0..base.len());
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
        let mut new_clique = chosen.clone();
        new_clique.push(v);
        cliques.push(new_clique);
    }
    edges
}

/// Stresses the rejection-sampling permutation path
/// (`draw_allowed_permutation` / `is_allowed`) and the weighted-choice selector
/// (`WeightedChoice::sample`) across a large battery of random chordal graphs with
/// many fixed-seed draws each, asserting every draw is a valid fully-directed
/// acyclic member and cross-checking the counter against the exact oracle.
///
/// Note: two branches on this path are unreachable defensive guards and stay
/// uncovered by design.
/// * `is_allowed`'s final post-loop `true` (`sample.rs:487`): with `helper`
///   values non-decreasing into the running max `mx` and `mx_0 >= 1` for any
///   accepted permutation, avoiding `mx == i` forces `mx_i >= i + 1`, so by the
///   last index `mx >= len` returns `true` early (`sample.rs:484`) — the loop can
///   never fall through.
/// * `WeightedChoice::sample`'s final `cumulative.len() - 1` fallthrough
///   (`sample.rs:133`): `target = total * u` with `u in [0, 1)` is strictly below
///   `total = cumulative.last()` except on the measure-zero floating-point rounding
///   edge `target == total`, which a finite draw sequence does not hit.
#[test]
fn rejection_sampling_permutation_stress() {
    let mut rng = Xoshiro256::from_seed(0xBADC0FFE);
    for trial in 0..120u64 {
        // Bias toward denser graphs (more multi-vertex cliques => more forbidden
        // prefixes covering whole cliques).
        let n = 5 + (trial as usize % 5); // 5..=9
        let edges = random_connected_chordal(&mut rng, n);
        let g = undirected_graph(n, &edges);

        // Cross-check the counter against the exact oracle on this random graph.
        let oracle = oracle_mec_size(&g).expect("oracle size");
        let cp: f64 = cp_mec_size(&g);
        assert_eq!(
            cp.round() as i128,
            oracle as i128,
            "clique-picking != oracle on trial={trial} n={n} edges={edges:?}"
        );

        let mut draw_rng = Xoshiro256::from_seed(0xD00D_0000 + trial);
        for _ in 0..200 {
            let sample = sample_dag::<f64, (), _>(&g, &mut draw_rng).expect("sample");
            for edge in sample.edges().values() {
                assert_eq!(edge.kind(), EdgeKind::Directed, "sample not fully directed");
            }
            assert!(!sample.has_cycle(), "sample is cyclic");
        }
    }
}
