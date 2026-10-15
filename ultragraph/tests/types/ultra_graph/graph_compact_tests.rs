/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `compact_ids` (spec: ultragraph-stable-node-ids, "compact_ids reclaims tombstones and returns
//! the mapping" and "Every algorithm agrees with the compacted graph").

use deep_causality_rand::{Rng, Xoshiro256};
use std::collections::BTreeSet;
use ultragraph::*;

#[test]
fn test_compacting_a_graph_with_one_tombstone() {
    let mut g = UltraGraph::new();
    assert_eq!(g.add_node("A").unwrap(), 0);
    assert_eq!(g.add_node("B").unwrap(), 1);
    assert_eq!(g.add_node("C").unwrap(), 2);
    g.add_edge(0, 2, ()).unwrap(); // A -> C
    g.add_edge(2, 0, ()).unwrap(); // C -> A
    g.remove_node(1).unwrap();

    let table = g.compact_ids().unwrap();

    // A keeps 0, B is gone, C moves from 2 to 1.
    assert_eq!(table, vec![Some(0), None, Some(1)]);
    assert_eq!(g.get_node(0), Some(&"A"));
    assert_eq!(g.get_node(1), Some(&"C"));
    assert!(!g.contains_node(2));
    assert_eq!(g.number_nodes(), 2);
    assert_eq!(g.get_last_index(), Some(1));
    assert!(g.contains_edge(0, 1));
    assert!(g.contains_edge(1, 0));
    assert!(!g.contains_edge(0, 2));
    assert!(!g.contains_edge(2, 0));
    assert_eq!(g.number_edges(), 2);

    // The reclaimed slot is reused: the next position is 2.
    assert_eq!(g.add_node("D").unwrap(), 2);
}

#[test]
fn test_compacting_a_graph_without_tombstones_changes_nothing() {
    let mut g = UltraGraphWeighted::new();
    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 1, 10u32).unwrap();
    g.add_edge(1, 3, 20).unwrap();
    g.add_edge(3, 0, 30).unwrap();

    let table = g.compact_ids().unwrap();

    assert_eq!(table, vec![Some(0), Some(1), Some(2), Some(3)]);
    for (position, payload) in ["A", "B", "C", "D"].into_iter().enumerate() {
        assert_eq!(g.get_node(position), Some(&payload));
    }
    assert_eq!(g.get_edges(0).unwrap(), vec![(1, &10)]);
    assert_eq!(g.get_edges(1).unwrap(), vec![(3, &20)]);
    assert_eq!(g.get_edges(2).unwrap(), Vec::new());
    assert_eq!(g.get_edges(3).unwrap(), vec![(0, &30)]);
    assert_eq!(g.number_edges(), 3);
    assert_eq!(g.add_node("E").unwrap(), 4);
}

#[test]
fn test_compacting_a_frozen_graph_is_refused() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    g.add_edge(0, 2, ()).unwrap();
    g.remove_node(1).unwrap();
    g.freeze();

    assert_eq!(g.compact_ids(), Err(GraphError::GraphIsFrozen));

    // Unchanged: still frozen, C still at 2, the tombstone still at 1.
    assert!(g.is_frozen());
    assert_eq!(g.get_node(0), Some(&"A"));
    assert!(!g.contains_node(1));
    assert_eq!(g.get_node(2), Some(&"C"));
    assert!(g.contains_edge(0, 2));
    assert_eq!(g.number_nodes(), 2);
}

/// Row A: `compact_ids` on a graph with no node.
#[test]
fn test_compact_ids_on_an_empty_graph() {
    let mut g: UltraGraph<&str> = UltraGraph::new();

    assert_eq!(g.compact_ids(), Ok(Vec::new()));

    assert!(g.is_empty());
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.get_last_index(), None);
    assert_eq!(g.add_node("A").unwrap(), 0);
}

#[test]
fn test_compact_ids_remaps_the_root() {
    let mut g = UltraGraph::new();
    for (position, payload) in ["A", "B", "C"].into_iter().enumerate() {
        assert_eq!(g.add_node(payload).unwrap(), position);
    }
    assert_eq!(g.add_root_node("ROOT").unwrap(), 3);
    g.add_edge(3, 2, ()).unwrap();
    g.remove_node(1).unwrap();

    let table = g.compact_ids().unwrap();

    // A 0 -> 0, B gone, C 2 -> 1, ROOT 3 -> 2.
    assert_eq!(table, vec![Some(0), None, Some(1), Some(2)]);
    assert_eq!(g.get_root_index(), Some(2));
    assert_eq!(g.get_root_node(), Some(&"ROOT"));
    assert!(g.contains_edge(2, 1));
    assert!(!g.contains_edge(3, 2));
}

// --- Agreement over generated graphs ---

fn map_positions(positions: &[usize], back: &[usize]) -> Vec<usize> {
    positions.iter().map(|&p| back[p]).collect()
}

fn mapped_sorted_components(components: &[Vec<usize>], back: &[usize]) -> Vec<Vec<usize>> {
    let mut mapped: Vec<Vec<usize>> = components
        .iter()
        .map(|component| {
            let mut mapped = map_positions(component, back);
            mapped.sort_unstable();
            mapped
        })
        .collect();
    mapped.sort();
    mapped
}

fn mapped_sorted_pairs(pairs: &[(usize, usize)], back: &[usize]) -> Vec<(usize, usize)> {
    let mut mapped: Vec<(usize, usize)> = pairs
        .iter()
        .map(|&(a, b)| {
            let (a, b) = (back[a], back[b]);
            if a < b { (a, b) } else { (b, a) }
        })
        .collect();
    mapped.sort_unstable();
    mapped
}

fn mapped_sorted_scores(scores: &[(usize, f64)], back: &[usize]) -> Vec<(usize, f64)> {
    let mut mapped: Vec<(usize, f64)> = scores
        .iter()
        .map(|&(node, score)| (back[node], score))
        .collect();
    mapped.sort_by_key(|(node, _)| *node);
    mapped
}

fn assert_scores_agree(with_tombstones: &[(usize, f64)], compacted: &[(usize, f64)], what: &str) {
    assert_eq!(
        with_tombstones.len(),
        compacted.len(),
        "{what}: entry count"
    );
    for (a, b) in with_tombstones.iter().zip(compacted) {
        assert_eq!(a.0, b.0, "{what}: node");
        assert!(
            (a.1 - b.1).abs() < 1e-9,
            "{what}: node {} scores {} with tombstones and {} compacted",
            a.0,
            a.1,
            b.1
        );
    }
}

/// Random graphs with random removals, frozen once as is and once after `compact_ids`; every
/// algorithm of `GraphAlgorithms` agrees after the compacted result is mapped back through the
/// table. The last position is never removed, so a live node sits above every tombstone.
#[test]
fn test_agreement_over_generated_graphs() {
    let mut rng = Xoshiro256::from_seed(0x5EED_0002);

    for round in 0..40 {
        let n: usize = rng.random_range(4..10usize);
        let mut g = UltraGraphWeighted::<u64, u64>::with_capacity(n, None);
        for p in 0..n {
            assert_eq!(g.add_node(p as u64).unwrap(), p);
        }

        // Distinct directed pairs; self-loops allowed.
        let edge_draws = rng.random_range(n..2 * n + 1);
        let mut pairs = BTreeSet::new();
        for _ in 0..edge_draws {
            let a = rng.random_range(0..n);
            let b = rng.random_range(0..n);
            if pairs.insert((a, b)) {
                let w = rng.random_range(1..10u64);
                g.add_edge(a, b, w).unwrap();
            }
        }

        // One to three removals below the last position.
        let removal_draws = rng.random_range(1..4usize);
        let mut removed = BTreeSet::new();
        for _ in 0..removal_draws {
            removed.insert(rng.random_range(0..n - 1));
        }
        for &p in &removed {
            g.remove_node(p).unwrap();
        }

        // The expected table and its inverse, from the removal set alone.
        let live: Vec<usize> = (0..n).filter(|p| !removed.contains(p)).collect();
        let mut expected_table = vec![None; n];
        for (new, &old) in live.iter().enumerate() {
            expected_table[old] = Some(new);
        }
        let back: &[usize] = &live; // back[new] = old
        let identity: Vec<usize> = (0..n).collect();

        let mut compacted = g.clone();
        g.freeze();

        for &t in &removed {
            assert!(
                !g.contains_node(t),
                "round {round}: position {t} is a tombstone"
            );
        }
        assert_eq!(g.number_nodes(), live.len(), "round {round}");

        let table = compacted.compact_ids().unwrap();
        assert_eq!(table, expected_table, "round {round}: table");
        compacted.freeze();
        assert_eq!(compacted.number_nodes(), live.len(), "round {round}");

        let what = format!("round {round}");

        // Topological.
        assert_eq!(
            g.find_cycle().unwrap(),
            compacted
                .find_cycle()
                .unwrap()
                .map(|cycle| map_positions(&cycle, back)),
            "{what}: find_cycle"
        );
        assert_eq!(
            g.has_cycle().unwrap(),
            compacted.has_cycle().unwrap(),
            "{what}: has_cycle"
        );
        assert_eq!(
            g.topological_sort().unwrap(),
            compacted
                .topological_sort()
                .unwrap()
                .map(|order| map_positions(&order, back)),
            "{what}: topological_sort"
        );

        // Structural.
        assert_eq!(
            mapped_sorted_components(&g.strongly_connected_components().unwrap(), &identity),
            mapped_sorted_components(&compacted.strongly_connected_components().unwrap(), back),
            "{what}: strongly_connected_components"
        );
        assert_eq!(
            g.articulation_points().unwrap(),
            {
                let mut mapped = map_positions(&compacted.articulation_points().unwrap(), back);
                mapped.sort_unstable();
                mapped
            },
            "{what}: articulation_points"
        );
        assert_eq!(
            mapped_sorted_pairs(&g.bridges().unwrap(), &identity),
            mapped_sorted_pairs(&compacted.bridges().unwrap(), back),
            "{what}: bridges"
        );
        assert_eq!(
            mapped_sorted_components(&g.biconnected_components().unwrap(), &identity),
            mapped_sorted_components(&compacted.biconnected_components().unwrap(), back),
            "{what}: biconnected_components"
        );

        // Pathfinding, on the same (start, stop) pairs mapped through the table.
        let mut pathways = Vec::new();
        let mut mapped_pathways = Vec::new();
        for _ in 0..4 {
            let s = live[rng.random_range(0..live.len())];
            let t = live[rng.random_range(0..live.len())];
            pathways.push((s, t));
            mapped_pathways.push((table[s].unwrap(), table[t].unwrap()));
        }
        for (&(s, t), &(cs, ct)) in pathways.iter().zip(&mapped_pathways) {
            assert_eq!(
                g.is_reachable(s, t).unwrap(),
                compacted.is_reachable(cs, ct).unwrap(),
                "{what}: is_reachable({s}, {t})"
            );
            assert_eq!(
                g.shortest_path_len(s, t).unwrap(),
                compacted.shortest_path_len(cs, ct).unwrap(),
                "{what}: shortest_path_len({s}, {t})"
            );
            assert_eq!(
                g.shortest_path(s, t).unwrap(),
                compacted
                    .shortest_path(cs, ct)
                    .unwrap()
                    .map(|path| map_positions(&path, back)),
                "{what}: shortest_path({s}, {t})"
            );
            assert_eq!(
                g.shortest_weighted_path(s, t).unwrap(),
                compacted
                    .shortest_weighted_path(cs, ct)
                    .unwrap()
                    .map(|(path, weight)| (map_positions(&path, back), weight)),
                "{what}: shortest_weighted_path({s}, {t})"
            );
        }

        // Centrality, every combination of direction and normalisation.
        for (directed, normalized) in [(true, true), (true, false), (false, true), (false, false)] {
            let label = format!("{what}: betweenness directed {directed} normalized {normalized}");
            assert_scores_agree(
                &mapped_sorted_scores(
                    &g.betweenness_centrality(directed, normalized).unwrap(),
                    &identity,
                ),
                &mapped_sorted_scores(
                    &compacted
                        .betweenness_centrality(directed, normalized)
                        .unwrap(),
                    back,
                ),
                &label,
            );
            let label =
                format!("{what}: pathway betweenness directed {directed} normalized {normalized}");
            assert_scores_agree(
                &mapped_sorted_scores(
                    &g.pathway_betweenness_centrality(&pathways, directed, normalized)
                        .unwrap(),
                    &identity,
                ),
                &mapped_sorted_scores(
                    &compacted
                        .pathway_betweenness_centrality(&mapped_pathways, directed, normalized)
                        .unwrap(),
                    back,
                ),
                &label,
            );
        }
    }
}

/// A root past the node table, as `from_parts` accepts, has no row in the mapping and is cleared
/// by `compact_ids` instead of being looked up.
#[test]
fn test_compact_ids_clears_a_root_past_the_table() {
    let mut g: DynamicGraph<&str, ()> =
        DynamicGraph::from_parts((vec![Some("A")], vec![vec![]], Some(5)));
    assert_eq!(g.root_index(), Some(5));
    assert!(!g.contains_root_node());

    let table = g.compact_ids();

    assert_eq!(table, vec![Some(0)]);
    assert_eq!(g.root_index(), None);
    assert_eq!(g.get_node(0), Some(&"A"));
}
