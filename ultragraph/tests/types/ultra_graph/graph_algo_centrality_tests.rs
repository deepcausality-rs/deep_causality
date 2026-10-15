/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

#[test]
fn test_betweenness_centrality_undirected_unnormalized() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_node(4).unwrap();

    // Path graph: 0-1-2-3-4
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 0, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(2, 1, 1).unwrap();
    g.add_edge(2, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.add_edge(3, 4, 1).unwrap();
    g.add_edge(4, 3, 1).unwrap();
    g.freeze();

    let result = g.betweenness_centrality(false, false).unwrap();
    // The raw scores are 6.0 and 8.0, but because the graph is undirected,
    // the final centrality score is halved.
    let expected = [(0, 0.0), (1, 3.0), (2, 4.0), (3, 3.0), (4, 0.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_betweenness_centrality_directed_unnormalized() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();

    // 0 -> 1 -> 2
    // 0 -> 3 -> 2
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(0, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.freeze();

    let result = g.betweenness_centrality(true, false).unwrap();
    let expected = [(0, 0.0), (1, 0.5), (2, 0.0), (3, 0.5)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_betweenness_centrality_normalized() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();

    // Path graph: 0-1-2-3
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 0, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(2, 1, 1).unwrap();
    g.add_edge(2, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.freeze();

    let result = g.betweenness_centrality(false, true).unwrap();

    // Unnormalized scores are 4.0 for nodes 1 and 2.
    // Normalization factor for this convention is (N-1)(N-2) = 6.
    // Expected value is 4.0 / 6.0 = 2.0 / 3.0
    let expected = [(0, 0.0), (1, 2.0 / 3.0), (2, 2.0 / 3.0), (3, 0.0)];

    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_betweenness_centrality_directed_normalized() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();

    // 0 -> 1 -> 2
    // 0 -> 3 -> 2
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(0, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.freeze();

    // N = 4
    // Directed normalization factor: (N-1)(N-2) = (3)(2) = 6
    // Unnormalized BC(1) = 0.5, BC(3) = 0.5
    // Normalized BC(1) = 0.5 / 6.0 = 0.0833...
    // Normalized BC(3) = 0.5 / 6.0 = 0.0833...
    let result = g.betweenness_centrality(true, true).unwrap();
    let expected = [(0, 0.0), (1, 0.5 / 6.0), (2, 0.0), (3, 0.5 / 6.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_pathway_betweenness_centrality_undirected() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();
    g.add_node(4).unwrap();

    // Graph: 0-1-2-3-4
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 0, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(2, 1, 1).unwrap();
    g.add_edge(2, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.add_edge(3, 4, 1).unwrap();
    g.add_edge(4, 3, 1).unwrap();
    g.freeze();

    let pathways = vec![(0, 4)]; // Pathway 0 -> 4
    let result = g
        .pathway_betweenness_centrality(&pathways, false, false)
        .unwrap();
    let expected = [(0, 0.0), (1, 1.0), (2, 1.0), (3, 1.0), (4, 0.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });

    let pathways = vec![(0, 2), (1, 3)]; // Pathways 0->2 and 1->3
    let result = g
        .pathway_betweenness_centrality(&pathways, false, false)
        .unwrap();
    let expected = [(0, 0.0), (1, 1.0), (2, 1.0), (3, 0.0), (4, 0.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_pathway_betweenness_centrality_directed() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();

    // 0 -> 1 -> 2
    // 0 -> 3 -> 2
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(0, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.freeze();

    let pathways = vec![(0, 2)]; // Pathway 0 -> 2
    let result = g
        .pathway_betweenness_centrality(&pathways, true, false)
        .unwrap();
    let expected = [(0, 0.0), (1, 0.5), (2, 0.0), (3, 0.5)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_pathway_betweenness_centrality_normalized() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.add_node(2).unwrap();
    g.add_node(3).unwrap();

    // Graph: 0-1-2-3
    g.add_edge(0, 1, 1).unwrap();
    g.add_edge(1, 0, 1).unwrap();
    g.add_edge(1, 2, 1).unwrap();
    g.add_edge(2, 1, 1).unwrap();
    g.add_edge(2, 3, 1).unwrap();
    g.add_edge(3, 2, 1).unwrap();
    g.freeze();

    let pathways = vec![(0, 3)]; // Pathway 0 -> 3
    let result = g
        .pathway_betweenness_centrality(&pathways, false, true)
        .unwrap();
    // Unnormalized BC for node 1 and 2 is 1.0 for pathway 0->3
    // Normalized by number of pathways (1)
    let expected = [(0, 0.0), (1, 1.0), (2, 1.0), (3, 0.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });

    let pathways = vec![(0, 2), (1, 3)]; // Pathways 0->2 and 1->3
    let result = g
        .pathway_betweenness_centrality(&pathways, false, true)
        .unwrap();
    // Unnormalized BC for node 1 and 2 is 1.0 for pathway 0->2 and 1.0 for pathway 1->3
    // Total unnormalized BC for node 1 and 2 is 2.0
    // Normalized by number of pathways (2)
    // Expected BC(1) = 2.0 / 2.0 = 1.0
    // Expected BC(2) = 2.0 / 2.0 = 1.0
    let expected = [(0, 0.0), (1, 0.5), (2, 0.5), (3, 0.0)];
    result.iter().for_each(|(node, score)| {
        let expected_score = expected.iter().find(|(n, _)| n == node).unwrap().1;
        assert!(
            (score - expected_score).abs() < 1e-9,
            "Node {node}: Expected {expected_score}, Got {score}"
        );
    });
}

#[test]
fn test_centrality_empty_graph() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.freeze();
    let bc = g.betweenness_centrality(true, true).unwrap();
    assert!(bc.is_empty());
    let pbc = g.pathway_betweenness_centrality(&[], true, true).unwrap();
    assert!(pbc.is_empty());
}

#[test]
fn test_centrality_single_node_graph() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.freeze();
    let bc = g.betweenness_centrality(true, true).unwrap();
    assert_eq!(bc.len(), 1);
    assert_eq!(bc[0], (0, 0.0));

    let pathways = vec![(0, 0)];
    let pbc = g
        .pathway_betweenness_centrality(&pathways, true, true)
        .unwrap();
    assert_eq!(pbc.len(), 1);
    assert_eq!(pbc[0], (0, 0.0));
}

#[test]
fn test_centrality_invalid_pathway_nodes() {
    let mut g = UltraGraphWeighted::<i32, i32>::new();
    g.add_node(0).unwrap();
    g.add_node(1).unwrap();
    g.freeze();

    let pathways = vec![(0, 99), (99, 1)]; // Invalid nodes
    let pbc = g
        .pathway_betweenness_centrality(&pathways, true, true)
        .unwrap();
    assert!(pbc.iter().all(|&(_, score)| score == 0.0)); // All scores should be 0.0 as no valid paths
}

/// A pathway through a diamond: the two shortest paths from 0 to 3 pass one through 1 and one
/// through 2, so each carries half the pathway's dependency.
#[test]
fn test_pathway_betweenness_splits_a_diamond_in_halves() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    for (a, b) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        g.add_edge(a, b, ()).unwrap();
    }
    g.freeze();

    let scores = g
        .pathway_betweenness_centrality(&[(0, 3)], true, false)
        .unwrap();
    let score_of = |node: usize| scores.iter().find(|(n, _)| *n == node).unwrap().1;

    // sigma(1) / sigma(3) * delta(3) = 1 / 2 * 1 = 1/2 for node 1, likewise for node 2; the
    // endpoints earn nothing.
    assert!((score_of(1) - 0.5).abs() < 1e-12);
    assert!((score_of(2) - 0.5).abs() < 1e-12);
    assert_eq!(score_of(0), 0.0);
    assert_eq!(score_of(3), 0.0);
}

/// A pathway from a node to itself is valid and counts toward the normalisation: with the
/// diamond pathway (0, 3) and the self-pathway (0, 0), the halves are divided by two pathways.
#[test]
fn test_a_self_pathway_counts_toward_the_normalisation() {
    let mut g = UltraGraph::new();
    for payload in 0..4 {
        g.add_node(payload).unwrap();
    }
    for (a, b) in [(0, 1), (0, 2), (1, 3), (2, 3)] {
        g.add_edge(a, b, ()).unwrap();
    }
    g.freeze();

    let scores = g
        .pathway_betweenness_centrality(&[(0, 3), (0, 0)], true, true)
        .unwrap();
    let score_of = |node: usize| scores.iter().find(|(n, _)| *n == node).unwrap().1;

    // 1/2 from the diamond pathway, over 2 valid pathways: 1/4 for nodes 1 and 2. Counting
    // the self-pathway as nothing would leave 1/2.
    assert!((score_of(1) - 0.25).abs() < 1e-12);
    assert!((score_of(2) - 0.25).abs() < 1e-12);
}
