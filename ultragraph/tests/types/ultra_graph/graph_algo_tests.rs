/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Algorithms on a frozen graph with tombstones (spec: ultragraph-stable-node-ids, "A tombstone
//! reads as absent in every state" and "Every algorithm agrees with the compacted graph").

use ultragraph::*;

/// Positions 0 to 4 with position 1 removed and the path 0 -> 2 -> 3 -> 4, frozen.
/// Three live nodes sit above the tombstone.
fn frozen_path_with_a_tombstone_at_one() -> UltraGraph<&'static str> {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D", "E"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();
    g.add_edge(3, 4, ()).unwrap();
    g.remove_node(1).unwrap();
    g.freeze();
    g
}

fn sorted_components(components: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    let mut sorted: Vec<Vec<usize>> = components
        .into_iter()
        .map(|mut component| {
            component.sort_unstable();
            component
        })
        .collect();
    sorted.sort();
    sorted
}

#[test]
fn test_algorithm_results_name_no_tombstone() {
    let g = frozen_path_with_a_tombstone_at_one();

    // A path has exactly one topological order: 0, 2, 3, 4.
    let order = g.topological_sort().unwrap().unwrap();
    assert_eq!(order, vec![0, 2, 3, 4]);

    // No directed cycle: every live node is its own component.
    let sccs = sorted_components(g.strongly_connected_components().unwrap());
    assert_eq!(sccs, vec![vec![0], vec![2], vec![3], vec![4]]);

    // The interior nodes of a path are its articulation points.
    assert_eq!(g.articulation_points().unwrap(), vec![2, 3]);

    // Every edge of a path is a bridge.
    assert_eq!(g.bridges().unwrap(), vec![(0, 2), (2, 3), (3, 4)]);

    // Each bridge is a biconnected component of its own.
    let blocks = sorted_components(g.biconnected_components().unwrap());
    assert_eq!(blocks, vec![vec![0, 2], vec![2, 3], vec![3, 4]]);

    // Scores are listed for the four live positions only.
    let mut scored: Vec<usize> = g
        .betweenness_centrality(true, false)
        .unwrap()
        .into_iter()
        .map(|(node, _)| node)
        .collect();
    scored.sort_unstable();
    assert_eq!(scored, vec![0, 2, 3, 4]);

    // The path from 0 to 4 passes the live positions above the tombstone.
    assert_eq!(g.shortest_path(0, 4).unwrap(), Some(vec![0, 2, 3, 4]));
}

#[test]
fn test_normalised_betweenness_counts_live_nodes() {
    let g = frozen_path_with_a_tombstone_at_one();

    let scores = g.betweenness_centrality(true, true).unwrap();
    let score_of =
        |node: usize| -> Option<f64> { scores.iter().find(|(n, _)| *n == node).map(|(_, s)| *s) };

    // Directed shortest paths with an interior node, on the path 0 -> 2 -> 3 -> 4:
    //   through 2: 0 -> 3 and 0 -> 4            (2 paths)
    //   through 3: 0 -> 4 and 2 -> 4            (2 paths)
    // Four live nodes normalise by (4 - 1)(4 - 2) = 6, so 2 / 6 = 1/3 for each.
    // Normalising by the five slots would give (5 - 1)(5 - 2) = 12 and 1/6 instead.
    let third = 1.0 / 3.0;
    assert!((score_of(2).unwrap() - third).abs() < 1e-12);
    assert!((score_of(3).unwrap() - third).abs() < 1e-12);
    assert!(score_of(0).unwrap().abs() < 1e-12);
    assert!(score_of(4).unwrap().abs() < 1e-12);
    assert_eq!(score_of(1), None);
    assert_eq!(scores.len(), 4);
}

#[test]
fn test_betweenness_with_fewer_than_three_live_nodes_in_five_slots() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D", "E"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 3, ()).unwrap();
    // Live: 0 and 3. Tombstones at 1, 2 and 4; a live node sits above two of them.
    g.remove_node(1).unwrap();
    g.remove_node(2).unwrap();
    g.remove_node(4).unwrap();
    g.freeze();
    assert_eq!(g.number_nodes(), 2);

    // With two live nodes no path has an interior node, so every score is 0, and the result
    // names the two live positions and nothing else.
    for (directed, normalized) in [(true, true), (true, false), (false, true), (false, false)] {
        let mut scores = g.betweenness_centrality(directed, normalized).unwrap();
        scores.sort_by_key(|(node, _)| *node);
        assert_eq!(
            scores.len(),
            2,
            "directed {directed}, normalized {normalized}"
        );
        assert_eq!(scores[0].0, 0);
        assert_eq!(scores[1].0, 3);
        assert_eq!(scores[0].1, 0.0);
        assert_eq!(scores[1].1, 0.0);
    }
}

/// Row E, the other side of the small-graph threshold: exactly three live nodes score, where two
/// (above) do not and four (`test_normalised_betweenness_counts_live_nodes`) do.
#[test]
fn test_betweenness_with_exactly_three_live_nodes_scores_the_middle() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 2, ()).unwrap();
    g.add_edge(2, 3, ()).unwrap();
    g.remove_node(1).unwrap();
    g.freeze();
    assert_eq!(g.number_nodes(), 3);

    let score_of = |scores: &[(usize, f64)], node: usize| -> f64 {
        scores
            .iter()
            .find(|(n, _)| *n == node)
            .map(|(_, s)| *s)
            .unwrap()
    };

    // The one directed shortest path with an interior node is 0 -> 2 -> 3, so node 2 scores 1
    // unnormalised; the threshold at three live nodes must let it through.
    let raw = g.betweenness_centrality(true, false).unwrap();
    assert_eq!(raw.len(), 3);
    assert!((score_of(&raw, 2) - 1.0).abs() < 1e-12);
    assert!(score_of(&raw, 0).abs() < 1e-12);
    assert!(score_of(&raw, 3).abs() < 1e-12);

    // Normalised by the three live nodes: 1 / ((3 - 1)(3 - 2)) = 1/2.
    let normalised = g.betweenness_centrality(true, true).unwrap();
    assert!((score_of(&normalised, 2) - 0.5).abs() < 1e-12);
}

/// Five live nodes pin the normalisation scale where four cannot: with integer division,
/// (4 - 1)(4 / 2) = (4 - 1)(4 - 2) = 6, but (5 - 1)(5 / 2) = 8 while (5 - 1)(5 - 2) = 12.
#[test]
fn test_normalised_betweenness_on_five_live_nodes_pins_the_scale() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C", "D", "E", "F"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    // The path 0 -> 2 -> 3 -> 4 -> 5 with a tombstone at 1.
    for (a, b) in [(0, 2), (2, 3), (3, 4), (4, 5)] {
        g.add_edge(a, b, ()).unwrap();
    }
    g.remove_node(1).unwrap();
    g.freeze();
    assert_eq!(g.number_nodes(), 5);

    let score_of = |scores: &[(usize, f64)], node: usize| -> f64 {
        scores
            .iter()
            .find(|(n, _)| *n == node)
            .map(|(_, s)| *s)
            .unwrap()
    };

    // On a path of five, the k-th interior node (k = 1, 2, 3 from the start) lies on
    // k * (4 - k) shortest paths with an interior: 3, 4, 3 for positions 2, 3, 4.
    // Directed scale (5 - 1)(5 - 2) = 12: 1/4, 1/3, 1/4.
    let directed = g.betweenness_centrality(true, true).unwrap();
    assert!((score_of(&directed, 2) - 0.25).abs() < 1e-12);
    assert!((score_of(&directed, 3) - 1.0 / 3.0).abs() < 1e-12);
    assert!((score_of(&directed, 4) - 0.25).abs() < 1e-12);

    // Undirected, each unordered pair once, scale (5 - 1)(5 - 2) / 2 = 6: 1/2, 2/3, 1/2.
    let undirected = g.betweenness_centrality(false, true).unwrap();
    assert!((score_of(&undirected, 2) - 0.5).abs() < 1e-12);
    assert!((score_of(&undirected, 3) - 2.0 / 3.0).abs() < 1e-12);
    assert!((score_of(&undirected, 4) - 0.5).abs() < 1e-12);
}

#[test]
fn test_algorithms_on_a_frozen_graph_with_no_live_node() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 1, ()).unwrap();
    g.add_edge(1, 2, ()).unwrap();
    for position in 0..3 {
        g.remove_node(position).unwrap();
    }
    g.freeze();

    assert_eq!(g.betweenness_centrality(true, true).unwrap(), Vec::new());
    assert_eq!(g.topological_sort().unwrap(), Some(Vec::new()));
    assert_eq!(
        g.strongly_connected_components().unwrap(),
        Vec::<Vec<usize>>::new()
    );
}
