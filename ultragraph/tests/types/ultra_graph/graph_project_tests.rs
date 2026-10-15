/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The projection view (spec: ultragraph-projection-view) and its two parallel-edge modes.
//!
//! Expected values come from hand counts written beside the literals, and from three oracles
//! independent of the graph: bridges by deleting each connection and testing connectivity,
//! betweenness by enumerating the shortest paths of graphs with at most eight nodes, and
//! projections by building the passing subgraph from a reference model with a separate filter.

use deep_causality_rand::{Rng, Xoshiro256};
use std::collections::{BTreeMap, VecDeque};
use std::ops::Add;
use ultragraph::*;

// --- Payloads ---

/// The two-variant kind that keys the `Weighted` payload.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Kind {
    #[default]
    A,
    B,
}

/// A payload keyed by its kind and ordered by its weight: the derived order compares the fields
/// in declaration order, so `weight` decides and `kind` breaks ties. Addition adds the weights.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Weighted {
    weight: u32,
    kind: Kind,
}

impl EdgeKey for Weighted {
    type Key = Kind;

    fn edge_key(&self) -> Self::Key {
        self.kind
    }
}

impl Add for Weighted {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            weight: self.weight + rhs.weight,
            kind: self.kind,
        }
    }
}

fn weighted(kind: Kind, weight: u32) -> Weighted {
    Weighted { weight, kind }
}

/// A node payload with two scopes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Zone {
    A,
    B,
}

/// An edge payload with no key whose validity the edge test reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Open {
    valid_until: Option<u64>,
}

impl EdgeKey for Open {
    type Key = ();

    fn edge_key(&self) -> Self::Key {}
}

/// A caller's own projection: one variant per scope, every test dispatched statically.
#[derive(Clone, Copy, Debug)]
enum Scope {
    ZoneA,
    ZoneB,
}

impl Projection<Zone, Weighted> for Scope {
    fn node(&self, node: &Zone) -> bool {
        match self {
            Scope::ZoneA => *node == Zone::A,
            Scope::ZoneB => *node == Zone::B,
        }
    }

    fn edge(&self, _edge: &Weighted) -> bool {
        true
    }
}

/// A node test and an edge test as plain function pointers.
type TestPair<N, W> = (fn(&N) -> bool, fn(&W) -> bool);

/// A projection that passes every node and every edge.
fn pass_all<N, W>() -> TestPair<N, W> {
    (|_| true, |_| true)
}

// --- Fixtures ---

/// A frozen graph with the node payload equal to its position and the given keyed edges.
fn frozen(n: usize, edges: &[(usize, usize, Kind, u32)]) -> UltraGraphWeighted<usize, Weighted> {
    let mut g = UltraGraphWeighted::new();
    for position in 0..n {
        assert_eq!(g.add_node(position).unwrap(), position);
    }
    for &(a, b, kind, weight) in edges {
        g.add_edge(a, b, weighted(kind, weight)).unwrap();
    }
    g.freeze();
    g
}

/// Nodes 0 and 1 in zone A, node 2 in zone B, with the given keyed edges, frozen.
fn frozen_zones(edges: &[(usize, usize, Kind)]) -> UltraGraphWeighted<Zone, Weighted> {
    let mut g = UltraGraphWeighted::new();
    g.add_node(Zone::A).unwrap(); // 0
    g.add_node(Zone::A).unwrap(); // 1
    g.add_node(Zone::B).unwrap(); // 2
    for &(a, b, kind) in edges {
        g.add_edge(a, b, weighted(kind, 1)).unwrap();
    }
    g.freeze();
    g
}

/// The score of `node` in a betweenness result.
fn score_of(scores: &[(usize, f64)], node: usize) -> f64 {
    scores
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("no score for node {node}"))
}

fn assert_scores(scores: &[(usize, f64)], expected: &[(usize, f64)], what: &str) {
    assert_eq!(
        scores.len(),
        expected.len(),
        "{what}: number of scored nodes"
    );
    for &(node, want) in expected {
        let got = score_of(scores, node);
        assert!(
            (got - want).abs() < 1e-12,
            "{what}: node {node} scores {got}, expected {want}"
        );
    }
}

fn sorted<T: Ord>(items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut items: Vec<T> = items.into_iter().collect();
    items.sort();
    items
}

// --- Projection is a node test and an edge test ---

#[test]
fn test_a_pair_of_closures_is_a_projection() {
    fn tests_of<P: Projection<u32, u64>>(p: &P) -> [bool; 4] {
        [p.node(&2), p.node(&1), p.edge(&9), p.edge(&10)]
    }

    let projection = (|n: &u32| *n > 1, |w: &u64| *w < 10);
    // The tests are the two closures: 2 > 1, not 1 > 1, 9 < 10, not 10 < 10.
    assert_eq!(tests_of(&projection), [true, false, true, false]);

    // And a frozen graph accepts the pair.
    let mut g: UltraGraphWeighted<u32, u64> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    g.add_node(2).unwrap();
    g.add_edge(0, 1, 5).unwrap();
    g.freeze();
    assert!(g.project(projection, ParallelEdges::Merge).is_ok());
}

#[test]
fn test_a_callers_own_type_is_a_projection() {
    let g = frozen_zones(&[(0, 1, Kind::A), (1, 2, Kind::A)]);

    let zone_a = g.project(Scope::ZoneA, ParallelEdges::Merge).unwrap();
    assert_eq!(zone_a.number_nodes(), 2);
    assert!(zone_a.contains_node(0));
    assert!(zone_a.contains_node(1));
    assert!(!zone_a.contains_node(2));

    let zone_b = g.project(Scope::ZoneB, ParallelEdges::Merge).unwrap();
    assert_eq!(zone_b.number_nodes(), 1);
    assert!(zone_b.contains_node(2));
    assert_eq!(zone_b.number_edges(), 0);
}

// --- ParallelEdges decides how parallel edges count ---

/// Row C: `test_results_that_do_not_depend_on_the_mode` is the case where the modes coincide.
#[test]
fn test_bridges_under_each_mode() {
    let g = frozen(
        3,
        &[(0, 1, Kind::A, 1), (0, 1, Kind::B, 1), (1, 2, Kind::A, 1)],
    );

    // Merge: {0,1} and {1,2} are one connection each; removing either disconnects the path.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    assert_eq!(merged.bridges().unwrap(), vec![(0, 1), (1, 2)]);

    // Distinct: {0,1} carries two edges, so only {1,2} is a bridge.
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    assert_eq!(distinct.bridges().unwrap(), vec![(1, 2)]);
}

#[test]
fn test_opposite_edges_count_once_in_both_modes() {
    let g = frozen(
        3,
        &[(0, 1, Kind::A, 1), (1, 0, Kind::A, 1), (1, 2, Kind::A, 1)],
    );

    // 0 -> 1 and 1 -> 0 are one undirected connection: max(1, 1) = 1 under Distinct, and one
    // per unordered pair under Merge. Both connections are bridges.
    for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
        let view = g.project(pass_all(), mode).unwrap();
        assert_eq!(view.bridges().unwrap(), vec![(0, 1), (1, 2)], "{mode:?}");
    }
}

#[test]
fn test_distinct_counts_the_larger_direction() {
    let g = frozen(
        3,
        &[
            (0, 1, Kind::A, 1),
            (0, 1, Kind::B, 1),
            (1, 0, Kind::A, 1),
            (1, 2, Kind::A, 1),
        ],
    );

    // {0,1} counts max(2, 1) = 2 undirected edges under Distinct, so it is not a bridge.
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    assert_eq!(distinct.bridges().unwrap(), vec![(1, 2)]);

    // Under Merge it is one connection, and a bridge.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    assert_eq!(merged.bridges().unwrap(), vec![(0, 1), (1, 2)]);
}

/// The graph of the two betweenness scenarios: 0 -> 1 twice (keys A and B), 0 -> 2, 1 -> 3,
/// 2 -> 3.
fn betweenness_graph() -> UltraGraphWeighted<usize, Weighted> {
    frozen(
        4,
        &[
            (0, 1, Kind::A, 1),
            (0, 1, Kind::B, 1),
            (0, 2, Kind::A, 1),
            (1, 3, Kind::A, 1),
            (2, 3, Kind::A, 1),
        ],
    )
}

#[test]
fn test_directed_betweenness_under_each_mode() {
    let g = betweenness_graph();

    // Directed, the only pair with an interior node is 0 -> 3 (every other reachable pair is
    // adjacent). Under Merge its shortest paths are 0 -> 1 -> 3 and 0 -> 2 -> 3: one of two
    // through 1, one of two through 2, so 1 and 2 score 1/2 each.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    let scores = merged.betweenness_centrality(true, false).unwrap();
    assert_scores(
        &scores,
        &[(0, 0.0), (1, 0.5), (2, 0.5), (3, 0.0)],
        "directed Merge",
    );

    // Under Distinct the doubled edge gives three shortest paths, 0 -> 1 -> 3 twice and
    // 0 -> 2 -> 3 once: 1 scores 2/3 and 2 scores 1/3.
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    let scores = distinct.betweenness_centrality(true, false).unwrap();
    assert_scores(
        &scores,
        &[(0, 0.0), (1, 2.0 / 3.0), (2, 1.0 / 3.0), (3, 0.0)],
        "directed Distinct",
    );
}

#[test]
fn test_undirected_betweenness_under_each_mode() {
    let g = betweenness_graph();

    // Undirected, the graph is the 4-cycle 0 - 1 - 3 - 2 - 0 with {0,1} doubled. The pairs with
    // an interior node are {0,3} (paths 0-1-3 and 0-2-3) and {1,2} (paths 1-0-2 and 1-3-2).
    // Under Merge each pair has two shortest paths, one through each middle node: every node
    // scores 1/2.
    let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
    let scores = merged.betweenness_centrality(false, false).unwrap();
    assert_scores(
        &scores,
        &[(0, 0.5), (1, 0.5), (2, 0.5), (3, 0.5)],
        "undirected Merge",
    );

    // Under Distinct the doubled edge gives each of those pairs three shortest paths, two of
    // which cross it: for {0,3}, 0-1-3 twice and 0-2-3 once, so 1 scores 2/3 and 2 scores 1/3;
    // for {1,2}, 1-0-2 twice and 1-3-2 once, so 0 scores 2/3 and 3 scores 1/3.
    let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
    let scores = distinct.betweenness_centrality(false, false).unwrap();
    assert_scores(
        &scores,
        &[
            (0, 2.0 / 3.0),
            (1, 2.0 / 3.0),
            (2, 1.0 / 3.0),
            (3, 1.0 / 3.0),
        ],
        "undirected Distinct",
    );
}

#[test]
fn test_the_weighted_path_takes_the_lightest_parallel_edge_in_both_modes() {
    let g = frozen(2, &[(0, 1, Kind::A, 5), (0, 1, Kind::B, 2)]);

    // The lighter of the two parallel edges weighs 2. The length is a sum starting at
    // `Weighted::default()`, whose kind is `A`, so only its weight is pinned.
    for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
        let view = g.project(pass_all(), mode).unwrap();
        let (path, length) = view.shortest_weighted_path(0, 1).unwrap().unwrap();
        assert_eq!(path, vec![0, 1], "{mode:?}");
        assert_eq!(length.weight, 2, "{mode:?}");
    }

    // The direct call answers the same.
    let (path, length) = g.shortest_weighted_path(0, 1).unwrap().unwrap();
    assert_eq!(path, vec![0, 1]);
    assert_eq!(length.weight, 2);
}

// --- project returns a view of the frozen graph ---

#[test]
fn test_projecting_a_dynamic_graph_is_refused() {
    let mut g: UltraGraphWeighted<usize, Weighted> = UltraGraphWeighted::new();
    g.add_node(0).unwrap();
    assert!(!g.is_frozen());

    assert!(matches!(
        g.project(pass_all(), ParallelEdges::Merge),
        Err(GraphError::GraphNotFrozen)
    ));
    assert!(matches!(
        g.project(pass_all(), ParallelEdges::Distinct),
        Err(GraphError::GraphNotFrozen)
    ));

    g.freeze();
    assert!(g.project(pass_all(), ParallelEdges::Merge).is_ok());
}

// --- A projection passes nodes and edges by their tests ---

#[test]
fn test_an_edge_between_scopes_fails_with_its_end() {
    let g = frozen_zones(&[(0, 1, Kind::A), (1, 2, Kind::A)]);
    let view = g.project(Scope::ZoneA, ParallelEdges::Merge).unwrap();

    // The edge 1 -> 2 fails with node 2.
    let out: Vec<usize> = view.outbound_edges(1).unwrap().collect();
    assert!(out.is_empty());
    assert!(!view.is_reachable(0, 2).unwrap());
    assert!(view.is_reachable(0, 1).unwrap());

    // On the graph itself the path exists.
    assert!(g.is_reachable(0, 2).unwrap());
}

#[test]
fn test_an_edge_test_filters_by_payload() {
    let mut g: UltraGraphWeighted<usize, Open> = UltraGraphWeighted::new();
    for position in 0..3 {
        g.add_node(position).unwrap();
    }
    g.add_edge(0, 1, Open { valid_until: None }).unwrap();
    g.add_edge(
        1,
        2,
        Open {
            valid_until: Some(5),
        },
    )
    .unwrap();
    g.freeze();

    let open = (|_: &usize| true, |edge: &Open| edge.valid_until.is_none());
    let view = g.project(open, ParallelEdges::Merge).unwrap();

    assert_eq!(view.shortest_path(0, 2).unwrap(), None);
    assert_eq!(view.shortest_path(0, 1).unwrap(), Some(vec![0, 1]));
    assert_eq!(view.number_edges(), 1);
    assert!(view.contains_edge(0, 1));
    assert!(!view.contains_edge(1, 2));

    // On the graph itself the closed edge is there.
    assert_eq!(g.shortest_path(0, 2).unwrap(), Some(vec![0, 1, 2]));
}

/// Row H: the node test flips on node 0 and keeps the last node.
#[test]
fn test_positions_are_not_renumbered() {
    let g = frozen(
        4,
        &[(0, 1, Kind::A, 1), (1, 2, Kind::A, 1), (2, 3, Kind::A, 1)],
    );
    let without_zero = (|n: &usize| *n != 0, |_: &Weighted| true);
    let view = g.project(without_zero, ParallelEdges::Merge).unwrap();

    // The chain 1 -> 2 -> 3 has one topological order, in the graph's positions.
    assert_eq!(view.topological_sort().unwrap(), Some(vec![1, 2, 3]));
    assert_eq!(view.shortest_path(1, 3).unwrap(), Some(vec![1, 2, 3]));
    assert_eq!(view.get_last_index(), Some(3));
    assert_eq!(view.get_node(3), Some(&3));
    assert_eq!(view.get_node(0), None);
}

// --- A view's reads and traversals list its passing elements ---

#[test]
fn test_reads_on_a_view_under_merge() {
    let g = frozen_zones(&[(0, 1, Kind::A), (0, 1, Kind::B), (1, 2, Kind::A)]);
    let view = g.project(Scope::ZoneA, ParallelEdges::Merge).unwrap();

    assert!(view.is_frozen());
    assert!(!view.is_empty());
    assert_eq!(view.number_nodes(), 2);
    assert!(view.contains_node(0));
    assert!(view.contains_node(1));
    assert!(!view.contains_node(2));
    assert_eq!(view.get_node(1), Some(&Zone::A));
    assert_eq!(view.get_node(2), None);
    assert_eq!(view.get_all_nodes(), vec![&Zone::A, &Zone::A]);
    assert_eq!(view.get_last_index(), Some(1));

    // Both edges to 1 are listed, whatever the mode.
    let edges = view.get_edges(0).unwrap();
    assert_eq!(
        sorted(
            edges
                .iter()
                .map(|(target, payload)| (*target, payload.kind))
        ),
        vec![(1, Kind::A), (1, Kind::B)]
    );
    let out: Vec<usize> = view.outbound_edges(0).unwrap().collect();
    assert_eq!(out, vec![1, 1]);
    let inbound: Vec<usize> = view.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![0, 0]);
    assert_eq!(view.number_edges(), 2);
    assert!(view.contains_edge(0, 1));
    assert!(view.contains_edge_keyed(0, 1, Kind::A));
    assert!(view.contains_edge_keyed(0, 1, Kind::B));

    // The edge 1 -> 2 fails with node 2.
    let out: Vec<usize> = view.outbound_edges(1).unwrap().collect();
    assert!(out.is_empty());
    assert_eq!(view.get_edges(1), Some(vec![]));
    assert!(!view.contains_edge(1, 2));
    assert!(!view.contains_edge_keyed(1, 2, Kind::A));

    // A failing node reads as absent.
    assert_eq!(view.get_edges(2), None);
    assert!(matches!(
        view.outbound_edges(2),
        Err(GraphError::NodeNotFound(2))
    ));
    assert!(matches!(
        view.inbound_edges(2),
        Err(GraphError::NodeNotFound(2))
    ));
}

#[test]
fn test_reads_on_a_view_under_distinct_list_the_same_edges() {
    let g = frozen_zones(&[(0, 1, Kind::A), (0, 1, Kind::B), (1, 2, Kind::A)]);
    let view = g.project(Scope::ZoneA, ParallelEdges::Distinct).unwrap();

    assert_eq!(view.number_nodes(), 2);
    assert_eq!(view.number_edges(), 2);
    assert_eq!(view.get_edges(0).unwrap().len(), 2);
    let out: Vec<usize> = view.outbound_edges(0).unwrap().collect();
    assert_eq!(out, vec![1, 1]);
    let inbound: Vec<usize> = view.inbound_edges(1).unwrap().collect();
    assert_eq!(inbound, vec![0, 0]);
}

#[test]
fn test_root_reads_return_the_root_only_when_it_passes() {
    let mut g: UltraGraphWeighted<Zone, Weighted> = UltraGraphWeighted::new();
    g.add_node(Zone::A).unwrap(); // 0
    g.add_root_node(Zone::B).unwrap(); // 1
    g.add_edge(1, 0, weighted(Kind::A, 1)).unwrap();
    g.freeze();

    let passes = g.project(Scope::ZoneB, ParallelEdges::Merge).unwrap();
    assert!(passes.contains_root_node());
    assert_eq!(passes.get_root_index(), Some(1));
    assert_eq!(passes.get_root_node(), Some(&Zone::B));

    let fails = g.project(Scope::ZoneA, ParallelEdges::Merge).unwrap();
    assert!(!fails.contains_root_node());
    assert_eq!(fails.get_root_index(), None);
    assert_eq!(fails.get_root_node(), None);
    assert_eq!(fails.number_nodes(), 1);
    assert_eq!(fails.number_edges(), 0);
}

/// Row A: a projection passing nothing.
#[test]
fn test_a_projection_passing_nothing() {
    let g = frozen(
        3,
        &[(0, 1, Kind::A, 1), (1, 2, Kind::A, 1), (2, 0, Kind::B, 1)],
    );
    let nothing = (|_: &usize| false, |_: &Weighted| true);

    for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
        let view = g.project(nothing, mode).unwrap();
        assert!(view.is_empty(), "{mode:?}");
        assert_eq!(view.number_nodes(), 0, "{mode:?}");
        assert_eq!(view.number_edges(), 0, "{mode:?}");
        assert!(view.get_all_nodes().is_empty(), "{mode:?}");
        assert_eq!(view.get_last_index(), None, "{mode:?}");
        assert!(!view.contains_node(0), "{mode:?}");
        assert!(!view.contains_edge(0, 1), "{mode:?}");
        assert_eq!(view.get_edges(0), None, "{mode:?}");
        assert!(matches!(
            view.outbound_edges(0),
            Err(GraphError::NodeNotFound(0))
        ));

        // Algorithms over no node.
        assert_eq!(view.topological_sort().unwrap(), Some(vec![]), "{mode:?}");
        assert!(!view.has_cycle().unwrap(), "{mode:?}");
        assert!(view.bridges().unwrap().is_empty(), "{mode:?}");
        assert!(view.articulation_points().unwrap().is_empty(), "{mode:?}");
        assert!(
            view.biconnected_components().unwrap().is_empty(),
            "{mode:?}"
        );
        assert!(
            view.strongly_connected_components().unwrap().is_empty(),
            "{mode:?}"
        );
        assert!(
            view.betweenness_centrality(true, false).unwrap().is_empty(),
            "{mode:?}"
        );
        assert!(!view.is_reachable(0, 1).unwrap(), "{mode:?}");
    }
}

/// Row B: a projection passing one node, which carries a self-loop.
#[test]
fn test_a_projection_passing_one_node() {
    let g = frozen(
        3,
        &[(1, 1, Kind::A, 1), (0, 1, Kind::A, 1), (1, 2, Kind::A, 1)],
    );
    let only_one = (|n: &usize| *n == 1, |_: &Weighted| true);

    for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
        let view = g.project(only_one, mode).unwrap();
        assert_eq!(view.number_nodes(), 1, "{mode:?}");
        assert_eq!(view.get_all_nodes(), vec![&1], "{mode:?}");
        assert_eq!(view.get_last_index(), Some(1), "{mode:?}");
        assert_eq!(view.number_edges(), 1, "{mode:?}"); // the self-loop alone
        assert!(view.contains_edge(1, 1), "{mode:?}");
        assert!(!view.contains_edge(0, 1), "{mode:?}");
        let out: Vec<usize> = view.outbound_edges(1).unwrap().collect();
        assert_eq!(out, vec![1], "{mode:?}");
        let inbound: Vec<usize> = view.inbound_edges(1).unwrap().collect();
        assert_eq!(inbound, vec![1], "{mode:?}");

        // A self-loop is a cycle; the single node is its own component with no bridge.
        assert!(view.has_cycle().unwrap(), "{mode:?}");
        assert_eq!(view.topological_sort().unwrap(), None, "{mode:?}");
        assert_eq!(
            view.strongly_connected_components().unwrap(),
            vec![vec![1]],
            "{mode:?}"
        );
        assert!(view.bridges().unwrap().is_empty(), "{mode:?}");
        assert_eq!(view.shortest_path(1, 1).unwrap(), Some(vec![1]), "{mode:?}");
    }
}

// --- The reference model and the independent oracles ---

/// The reference model: positions to payloads (`None` for a tombstone), edges as
/// `(source, target, key)` triples to payloads, and the root.
#[derive(Clone, Debug)]
struct Model {
    nodes: Vec<Option<u32>>,
    edges: BTreeMap<(usize, usize, Kind), Weighted>,
    root: Option<usize>,
}

impl Model {
    fn is_live(&self, position: usize) -> bool {
        self.nodes.get(position).is_some_and(Option::is_some)
    }

    fn live(&self) -> Vec<usize> {
        (0..self.nodes.len()).filter(|&p| self.is_live(p)).collect()
    }

    /// The subgraph a projection passes: the nodes whose payload passes `node_test` and the
    /// edges whose payload passes `edge_test` between two passing nodes. The filter walks the
    /// model's own tables and never reads the graph.
    fn filtered(
        &self,
        node_test: impl Fn(&u32) -> bool,
        edge_test: impl Fn(&Weighted) -> bool,
    ) -> Model {
        let nodes: Vec<Option<u32>> = self
            .nodes
            .iter()
            .map(|slot| slot.filter(|payload| node_test(payload)))
            .collect();
        let passes = |p: usize| nodes.get(p).is_some_and(Option::is_some);
        let edges = self
            .edges
            .iter()
            .filter(|&(&(a, b, _), payload)| passes(a) && passes(b) && edge_test(payload))
            .map(|(&triple, &payload)| (triple, payload))
            .collect();
        let root = self.root.filter(|&r| passes(r));
        Model { nodes, edges, root }
    }

    /// A frozen graph holding exactly the model: every position is issued in order, a tombstone
    /// is a node added and removed, and the root is added with `add_root_node`.
    fn build(&self) -> UltraGraphWeighted<u32, Weighted> {
        let mut g = UltraGraphWeighted::new();
        for (position, slot) in self.nodes.iter().enumerate() {
            let payload = slot.unwrap_or(u32::MAX);
            let issued = if self.root == Some(position) {
                g.add_root_node(payload).unwrap()
            } else {
                g.add_node(payload).unwrap()
            };
            assert_eq!(issued, position);
        }
        for (&(a, b, _), &payload) in &self.edges {
            g.add_edge(a, b, payload).unwrap();
        }
        for (position, slot) in self.nodes.iter().enumerate() {
            if slot.is_none() {
                g.remove_node(position).unwrap();
            }
        }
        g.freeze();
        g
    }

    /// Connections between ordered pairs of live nodes under `mode`, by the spec's rule: one
    /// per pair under Merge, one per edge under Distinct.
    fn directed_connections(&self, mode: ParallelEdges) -> BTreeMap<(usize, usize), usize> {
        let mut counts: BTreeMap<(usize, usize), usize> = BTreeMap::new();
        for &(a, b, _) in self.edges.keys() {
            let count = counts.entry((a, b)).or_insert(0);
            *count = match mode {
                ParallelEdges::Merge => 1,
                ParallelEdges::Distinct => *count + 1,
            };
        }
        counts
    }

    /// Connections between unordered pairs `{a, b}` with `a < b`, self-loops dropped, by the
    /// spec's rule: one per pair under Merge, `max(forward, reverse)` under Distinct.
    fn undirected_connections(&self, mode: ParallelEdges) -> BTreeMap<(usize, usize), usize> {
        let mut forward_reverse: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
        for &(a, b, _) in self.edges.keys() {
            if a == b {
                continue;
            }
            let entry = forward_reverse
                .entry((a.min(b), a.max(b)))
                .or_insert((0, 0));
            if a < b {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        forward_reverse
            .into_iter()
            .map(|(pair, (f, r))| {
                let count = match mode {
                    ParallelEdges::Merge => 1,
                    ParallelEdges::Distinct => f.max(r),
                };
                (pair, count)
            })
            .collect()
    }
}

/// Whether `from` reaches `to` over `connections` as undirected links, skipping the link at
/// `skip`.
fn connected_without(
    slots: usize,
    connections: &[(usize, usize)],
    skip: usize,
    from: usize,
    to: usize,
) -> bool {
    let mut seen = vec![false; slots];
    let mut queue = VecDeque::from([from]);
    seen[from] = true;
    while let Some(u) = queue.pop_front() {
        if u == to {
            return true;
        }
        for (i, &(a, b)) in connections.iter().enumerate() {
            if i == skip {
                continue;
            }
            let next = if a == u {
                b
            } else if b == u {
                a
            } else {
                continue;
            };
            if !seen[next] {
                seen[next] = true;
                queue.push_back(next);
            }
        }
    }
    false
}

/// Bridges by deletion: a connection is a bridge when its ends are disconnected without it.
/// Each parallel connection is its own entry, so a doubled pair is never a bridge.
fn bridges_oracle(m: &Model, mode: ParallelEdges) -> Vec<(usize, usize)> {
    let connections: Vec<(usize, usize)> = m
        .undirected_connections(mode)
        .into_iter()
        .flat_map(|(pair, count)| std::iter::repeat_n(pair, count))
        .collect();
    let slots = m.nodes.len();
    let mut bridges: Vec<(usize, usize)> = connections
        .iter()
        .enumerate()
        .filter(|&(i, &(a, b))| !connected_without(slots, &connections, i, a, b))
        .map(|(_, &pair)| pair)
        .collect();
    bridges.sort_unstable();
    bridges.dedup();
    bridges
}

/// Counts the shortest paths from the end of `path` to `t` that extend `path`, one per
/// connection, adding each to `total` and to `through[v]` for every interior node `v`.
fn enumerate_paths(
    adjacency: &BTreeMap<(usize, usize), usize>,
    dist: &[Option<usize>],
    path: &mut Vec<usize>,
    t: usize,
    total: &mut f64,
    through: &mut [f64],
) {
    let current = *path.last().unwrap();
    if current == t {
        *total += 1.0;
        for &v in &path[1..path.len() - 1] {
            through[v] += 1.0;
        }
        return;
    }
    let next_depth = dist[current].unwrap() + 1;
    for (&(a, b), &count) in adjacency.range((current, 0)..(current + 1, 0)) {
        debug_assert_eq!(a, current);
        if dist[b] != Some(next_depth) {
            continue;
        }
        for _ in 0..count {
            path.push(b);
            enumerate_paths(adjacency, dist, path, t, total, through);
            path.pop();
        }
    }
}

/// Unnormalised betweenness by enumerating every shortest path between every ordered pair of
/// live nodes over `adjacency`, a map from ordered pairs to their number of connections.
/// Undirected scores halve the sum over ordered pairs, as each unordered pair is seen twice.
fn betweenness_oracle(
    m: &Model,
    adjacency: &BTreeMap<(usize, usize), usize>,
    undirected: bool,
) -> Vec<(usize, f64)> {
    let slots = m.nodes.len();
    let live = m.live();
    let mut score = vec![0.0; slots];
    for &s in &live {
        // Breadth-first distances from s.
        let mut dist = vec![None; slots];
        dist[s] = Some(0);
        let mut queue = VecDeque::from([s]);
        while let Some(u) = queue.pop_front() {
            for (&(_, v), _) in adjacency.range((u, 0)..(u + 1, 0)) {
                if dist[v].is_none() {
                    dist[v] = Some(dist[u].unwrap() + 1);
                    queue.push_back(v);
                }
            }
        }
        for &t in &live {
            if t == s || dist[t].is_none() {
                continue;
            }
            let mut total = 0.0;
            let mut through = vec![0.0; slots];
            let mut path = vec![s];
            enumerate_paths(adjacency, &dist, &mut path, t, &mut total, &mut through);
            for v in 0..slots {
                score[v] += through[v] / total;
            }
        }
    }
    let factor = if undirected { 0.5 } else { 1.0 };
    live.into_iter().map(|v| (v, score[v] * factor)).collect()
}

/// The symmetric adjacency of the undirected connections: both `(a, b)` and `(b, a)` carry the
/// pair's count.
fn symmetric(connections: &BTreeMap<(usize, usize), usize>) -> BTreeMap<(usize, usize), usize> {
    connections
        .iter()
        .flat_map(|(&(a, b), &count)| [((a, b), count), ((b, a), count)])
        .collect()
}

/// A random model of two to eight nodes with parallel edges of both kinds, up to one tombstone
/// and sometimes a root, and the frozen graph built by the same operations.
fn random_model(rng: &mut Xoshiro256) -> (Model, UltraGraphWeighted<u32, Weighted>) {
    let n = rng.random_range(2..9usize);
    let root = if rng.random_range(0..3usize) == 0 {
        Some(rng.random_range(0..n))
    } else {
        None
    };
    let mut m = Model {
        nodes: Vec::new(),
        edges: BTreeMap::new(),
        root,
    };
    let mut g = UltraGraphWeighted::new();
    for position in 0..n {
        let payload = rng.random_range(0..10u32);
        m.nodes.push(Some(payload));
        let issued = if root == Some(position) {
            g.add_root_node(payload).unwrap()
        } else {
            g.add_node(payload).unwrap()
        };
        assert_eq!(issued, position);
    }
    let edge_count = rng.random_range(0..(3 * n + 1));
    for _ in 0..edge_count {
        let a = rng.random_range(0..n);
        let b = rng.random_range(0..n);
        let kind = if rng.random_range(0..2usize) == 0 {
            Kind::A
        } else {
            Kind::B
        };
        let payload = weighted(kind, rng.random_range(1..10u32));
        m.edges.insert((a, b, kind), payload);
        g.add_edge(a, b, payload).unwrap();
    }
    if rng.random_range(0..3usize) == 0 {
        let p = rng.random_range(0..n);
        m.nodes[p] = None;
        m.edges.retain(|&(a, b, _), _| a != p && b != p);
        if m.root == Some(p) {
            m.root = None;
        }
        g.remove_node(p).unwrap();
    }
    g.freeze();
    (m, g)
}

/// Every pair of live positions, for the pathway betweenness and the pairwise algorithms.
fn pairs(slots: usize) -> Vec<(usize, usize)> {
    (0..slots)
        .flat_map(|s| (0..slots).map(move |t| (s, t)))
        .collect()
}

fn assert_scores_close(got: &[(usize, f64)], want: &[(usize, f64)], what: &str) {
    assert_eq!(
        sorted(got.iter().map(|(n, _)| *n)),
        sorted(want.iter().map(|(n, _)| *n)),
        "{what}: scored nodes"
    );
    for &(node, score) in want {
        let actual = score_of(got, node);
        assert!(
            (actual - score).abs() < 1e-9,
            "{what}: node {node} scores {actual}, oracle {score}"
        );
    }
}

/// The model of `betweenness_graph` and of the bridges scenarios: the oracles must reproduce the
/// hand counts written beside those scenarios before they are trusted on generated graphs.
fn model_of(n: usize, edges: &[(usize, usize, Kind, u32)]) -> Model {
    Model {
        nodes: (0..n).map(|p| Some(p as u32)).collect(),
        edges: edges
            .iter()
            .map(|&(a, b, kind, weight)| ((a, b, kind), weighted(kind, weight)))
            .collect(),
        root: None,
    }
}

#[test]
fn test_the_oracles_reproduce_the_hand_counts() {
    // Bridges: the three scenarios of `ParallelEdges`.
    let doubled = model_of(
        3,
        &[(0, 1, Kind::A, 1), (0, 1, Kind::B, 1), (1, 2, Kind::A, 1)],
    );
    assert_eq!(
        bridges_oracle(&doubled, ParallelEdges::Merge),
        vec![(0, 1), (1, 2)]
    );
    assert_eq!(
        bridges_oracle(&doubled, ParallelEdges::Distinct),
        vec![(1, 2)]
    );

    let opposite = model_of(
        3,
        &[(0, 1, Kind::A, 1), (1, 0, Kind::A, 1), (1, 2, Kind::A, 1)],
    );
    assert_eq!(
        bridges_oracle(&opposite, ParallelEdges::Merge),
        vec![(0, 1), (1, 2)]
    );
    assert_eq!(
        bridges_oracle(&opposite, ParallelEdges::Distinct),
        vec![(0, 1), (1, 2)]
    );

    let larger = model_of(
        3,
        &[
            (0, 1, Kind::A, 1),
            (0, 1, Kind::B, 1),
            (1, 0, Kind::A, 1),
            (1, 2, Kind::A, 1),
        ],
    );
    assert_eq!(
        bridges_oracle(&larger, ParallelEdges::Distinct),
        vec![(1, 2)]
    );

    // Betweenness: the two scenarios on the doubled diamond.
    let diamond = model_of(
        4,
        &[
            (0, 1, Kind::A, 1),
            (0, 1, Kind::B, 1),
            (0, 2, Kind::A, 1),
            (1, 3, Kind::A, 1),
            (2, 3, Kind::A, 1),
        ],
    );
    let directed = |mode| betweenness_oracle(&diamond, &diamond.directed_connections(mode), false);
    assert_scores(
        &directed(ParallelEdges::Merge),
        &[(0, 0.0), (1, 0.5), (2, 0.5), (3, 0.0)],
        "oracle directed Merge",
    );
    assert_scores(
        &directed(ParallelEdges::Distinct),
        &[(0, 0.0), (1, 2.0 / 3.0), (2, 1.0 / 3.0), (3, 0.0)],
        "oracle directed Distinct",
    );
    let undirected = |mode| {
        betweenness_oracle(
            &diamond,
            &symmetric(&diamond.undirected_connections(mode)),
            true,
        )
    };
    assert_scores(
        &undirected(ParallelEdges::Merge),
        &[(0, 0.5), (1, 0.5), (2, 0.5), (3, 0.5)],
        "oracle undirected Merge",
    );
    assert_scores(
        &undirected(ParallelEdges::Distinct),
        &[
            (0, 2.0 / 3.0),
            (1, 2.0 / 3.0),
            (2, 1.0 / 3.0),
            (3, 1.0 / 3.0),
        ],
        "oracle undirected Distinct",
    );
}

#[test]
fn test_bridges_agree_with_the_deletion_oracle_under_each_mode() {
    let mut rng = Xoshiro256::from_seed(31);
    for case in 0..200 {
        let (m, g) = random_model(&mut rng);
        for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
            let view = g.project(pass_all(), mode).unwrap();
            assert_eq!(
                view.bridges().unwrap(),
                bridges_oracle(&m, mode),
                "case {case} {mode:?}: {m:?}"
            );
        }
        // The direct call runs on the simple projection, which is Merge.
        assert_eq!(
            g.bridges().unwrap(),
            bridges_oracle(&m, ParallelEdges::Merge),
            "case {case} direct: {m:?}"
        );
    }
}

#[test]
fn test_betweenness_agrees_with_the_path_enumeration_oracle_under_each_mode() {
    let mut rng = Xoshiro256::from_seed(32);
    for case in 0..200 {
        let (m, g) = random_model(&mut rng);
        for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
            let view = g.project(pass_all(), mode).unwrap();
            let directed = betweenness_oracle(&m, &m.directed_connections(mode), false);
            assert_scores_close(
                &view.betweenness_centrality(true, false).unwrap(),
                &directed,
                &format!("case {case} {mode:?} directed: {m:?}"),
            );
            let undirected =
                betweenness_oracle(&m, &symmetric(&m.undirected_connections(mode)), true);
            assert_scores_close(
                &view.betweenness_centrality(false, false).unwrap(),
                &undirected,
                &format!("case {case} {mode:?} undirected: {m:?}"),
            );
        }
        // The direct call runs on the simple projection, which is Merge.
        let directed = betweenness_oracle(&m, &m.directed_connections(ParallelEdges::Merge), false);
        assert_scores_close(
            &g.betweenness_centrality(true, false).unwrap(),
            &directed,
            &format!("case {case} direct directed: {m:?}"),
        );
        let undirected = betweenness_oracle(
            &m,
            &symmetric(&m.undirected_connections(ParallelEdges::Merge)),
            true,
        );
        assert_scores_close(
            &g.betweenness_centrality(false, false).unwrap(),
            &undirected,
            &format!("case {case} direct undirected: {m:?}"),
        );
    }
}

// --- Algorithms on the graph run on the simple projection ---

/// Every algorithm's result, so that two sources can be compared whole.
#[derive(Debug, PartialEq)]
struct AlgorithmResults {
    find_cycle: Option<Vec<usize>>,
    has_cycle: bool,
    topological_sort: Option<Vec<usize>>,
    is_reachable: Vec<bool>,
    shortest_path_len: Vec<Option<usize>>,
    shortest_path: Vec<Option<Vec<usize>>>,
    shortest_weighted_path: Vec<Option<(Vec<usize>, Weighted)>>,
    strongly_connected_components: Vec<Vec<usize>>,
    articulation_points: Vec<usize>,
    bridges: Vec<(usize, usize)>,
    biconnected_components: Vec<Vec<usize>>,
    betweenness: Vec<Vec<(usize, f64)>>,
    pathway_betweenness: Vec<Vec<(usize, f64)>>,
}

fn algorithm_results<G>(g: &G, slots: usize) -> AlgorithmResults
where
    G: GraphAlgorithms<u32, Weighted>,
{
    let pairs = pairs(slots);
    let flags = [(true, true), (true, false), (false, true), (false, false)];
    AlgorithmResults {
        find_cycle: g.find_cycle().unwrap(),
        has_cycle: g.has_cycle().unwrap(),
        topological_sort: g.topological_sort().unwrap(),
        is_reachable: pairs
            .iter()
            .map(|&(s, t)| g.is_reachable(s, t).unwrap())
            .collect(),
        shortest_path_len: pairs
            .iter()
            .map(|&(s, t)| g.shortest_path_len(s, t).unwrap())
            .collect(),
        shortest_path: pairs
            .iter()
            .map(|&(s, t)| g.shortest_path(s, t).unwrap())
            .collect(),
        shortest_weighted_path: pairs
            .iter()
            .map(|&(s, t)| g.shortest_weighted_path(s, t).unwrap())
            .collect(),
        strongly_connected_components: g.strongly_connected_components().unwrap(),
        articulation_points: g.articulation_points().unwrap(),
        bridges: g.bridges().unwrap(),
        biconnected_components: g.biconnected_components().unwrap(),
        betweenness: flags
            .iter()
            .map(|&(directed, normalized)| g.betweenness_centrality(directed, normalized).unwrap())
            .collect(),
        pathway_betweenness: flags
            .iter()
            .map(|&(directed, normalized)| {
                g.pathway_betweenness_centrality(&pairs, directed, normalized)
                    .unwrap()
            })
            .collect(),
    }
}

/// Every read and traversal for every position up to one past the table.
#[derive(Debug, PartialEq)]
struct ViewResults {
    is_frozen: bool,
    is_empty: bool,
    number_nodes: usize,
    number_edges: usize,
    get_all_nodes: Vec<u32>,
    get_last_index: Option<usize>,
    contains_root_node: bool,
    get_root_node: Option<u32>,
    get_root_index: Option<usize>,
    contains_node: Vec<bool>,
    get_node: Vec<Option<u32>>,
    contains_edge: Vec<bool>,
    contains_edge_keyed: Vec<bool>,
    get_edges: Vec<Option<Vec<(usize, Weighted)>>>,
    outbound: Vec<Result<Vec<usize>, GraphError>>,
    inbound: Vec<Result<Vec<usize>, GraphError>>,
}

fn view_results<G>(g: &G, slots: usize) -> ViewResults
where
    G: GraphTraversal<u32, Weighted>,
{
    let positions = 0..=slots;
    ViewResults {
        is_frozen: g.is_frozen(),
        is_empty: g.is_empty(),
        number_nodes: g.number_nodes(),
        number_edges: g.number_edges(),
        get_all_nodes: sorted(g.get_all_nodes().into_iter().copied()),
        get_last_index: g.get_last_index(),
        contains_root_node: g.contains_root_node(),
        get_root_node: g.get_root_node().copied(),
        get_root_index: g.get_root_index(),
        contains_node: positions.clone().map(|p| g.contains_node(p)).collect(),
        get_node: positions.clone().map(|p| g.get_node(p).copied()).collect(),
        contains_edge: pairs(slots + 1)
            .into_iter()
            .map(|(a, b)| g.contains_edge(a, b))
            .collect(),
        contains_edge_keyed: pairs(slots + 1)
            .into_iter()
            .flat_map(|(a, b)| {
                [
                    g.contains_edge_keyed(a, b, Kind::A),
                    g.contains_edge_keyed(a, b, Kind::B),
                ]
            })
            .collect(),
        get_edges: positions
            .clone()
            .map(|p| {
                g.get_edges(p)
                    .map(|edges| sorted(edges.into_iter().map(|(t, w)| (t, *w))))
            })
            .collect(),
        outbound: positions
            .clone()
            .map(|p| g.outbound_edges(p).map(sorted))
            .collect(),
        inbound: positions.map(|p| g.inbound_edges(p).map(sorted)).collect(),
    }
}

#[test]
fn test_direct_calls_equal_the_simple_projection() {
    let mut rng = Xoshiro256::from_seed(33);
    for case in 0..100 {
        let (m, g) = random_model(&mut rng);
        let slots = m.nodes.len();
        let simple = g.project(pass_all(), ParallelEdges::Merge).unwrap();

        assert_eq!(
            algorithm_results(&g, slots),
            algorithm_results(&simple, slots),
            "case {case} algorithms: {m:?}"
        );
        assert_eq!(
            view_results(&g, slots),
            view_results(&simple, slots),
            "case {case} reads and traversals: {m:?}"
        );
    }
}

// --- A projection's results equal those of the graph it selects ---

#[test]
fn test_agreement_over_generated_graphs_and_projections() {
    let mut rng = Xoshiro256::from_seed(34);
    for case in 0..100 {
        let (m, g) = random_model(&mut rng);
        let slots = m.nodes.len();

        // A node passes below a threshold on its payload; an edge passes below a threshold on
        // its weight, or by its kind. Thresholds at the ends give empty and full projections.
        let node_threshold = rng.random_range(0..11u32);
        let edge_threshold = rng.random_range(0..11u32);
        let by_kind = rng.random_range(0..3usize) == 0;
        let node_test = move |n: &u32| *n < node_threshold;
        let edge_test = move |w: &Weighted| {
            if by_kind {
                w.kind == Kind::A
            } else {
                w.weight < edge_threshold
            }
        };

        let selected = m.filtered(node_test, edge_test);
        let built = selected.build();

        for mode in [ParallelEdges::Merge, ParallelEdges::Distinct] {
            let view = g.project((node_test, edge_test), mode).unwrap();
            let expected = built.project(pass_all(), mode).unwrap();
            assert_eq!(
                algorithm_results(&view, slots),
                algorithm_results(&expected, slots),
                "case {case} {mode:?} algorithms: {m:?} selected {selected:?}"
            );
            // Reads and traversals do not depend on the mode: the built graph answers directly.
            assert_eq!(
                view_results(&view, slots),
                view_results(&built, slots),
                "case {case} {mode:?} reads and traversals: {m:?} selected {selected:?}"
            );
        }
    }
}

/// Row C: with parallel edges present, the results that count connections rather than edges
/// agree between the modes.
#[test]
fn test_results_that_do_not_depend_on_the_mode() {
    // Hand-built: a cycle with a doubled edge and a pendant path with a doubled edge.
    let hand_built = frozen(
        5,
        &[
            (0, 1, Kind::A, 1),
            (0, 1, Kind::B, 3),
            (1, 2, Kind::A, 1),
            (2, 0, Kind::A, 1),
            (2, 3, Kind::A, 1),
            (2, 3, Kind::B, 1),
            (3, 4, Kind::B, 2),
        ],
    );
    let (merged, distinct) = (
        hand_built
            .project(pass_all(), ParallelEdges::Merge)
            .unwrap(),
        hand_built
            .project(pass_all(), ParallelEdges::Distinct)
            .unwrap(),
    );
    // Hand-derived: the cycle 0 -> 1 -> 2 -> 0, articulation points 2 and 3, and the bridge
    // {3,4} alone, since {2,3} is doubled under Distinct and part of no cycle under Merge.
    assert!(merged.has_cycle().unwrap());
    assert!(distinct.has_cycle().unwrap());
    assert_eq!(merged.articulation_points().unwrap(), vec![2, 3]);
    assert_eq!(distinct.articulation_points().unwrap(), vec![2, 3]);
    assert_eq!(merged.bridges().unwrap(), vec![(2, 3), (3, 4)]);
    assert_eq!(distinct.bridges().unwrap(), vec![(3, 4)]);

    let mut rng = Xoshiro256::from_seed(35);
    for case in 0..100 {
        let (m, g) = random_model(&mut rng);
        let slots = m.nodes.len();
        let merged = g.project(pass_all(), ParallelEdges::Merge).unwrap();
        let distinct = g.project(pass_all(), ParallelEdges::Distinct).unwrap();
        let (a, b) = (
            algorithm_results(&merged, slots),
            algorithm_results(&distinct, slots),
        );

        assert_eq!(a.is_reachable, b.is_reachable, "case {case}: {m:?}");
        assert_eq!(a.find_cycle, b.find_cycle, "case {case}: {m:?}");
        assert_eq!(a.has_cycle, b.has_cycle, "case {case}: {m:?}");
        assert_eq!(a.topological_sort, b.topological_sort, "case {case}: {m:?}");
        assert_eq!(
            a.strongly_connected_components, b.strongly_connected_components,
            "case {case}: {m:?}"
        );
        assert_eq!(
            a.shortest_path_len, b.shortest_path_len,
            "case {case}: {m:?}"
        );
        assert_eq!(a.shortest_path, b.shortest_path, "case {case}: {m:?}");
        assert_eq!(
            a.articulation_points, b.articulation_points,
            "case {case}: {m:?}"
        );
        assert_eq!(
            a.biconnected_components, b.biconnected_components,
            "case {case}: {m:?}"
        );
        // The weighted path takes the lightest parallel edge under both modes.
        assert_eq!(
            a.shortest_weighted_path, b.shortest_weighted_path,
            "case {case}: {m:?}"
        );
    }
}
