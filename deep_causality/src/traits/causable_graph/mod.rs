/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use ultragraph::UltraGraphWeighted;

pub mod graph;
pub mod graph_reasoning;

// Type alias is shared between trait and implementation
pub(crate) type CausalGraph<T> = UltraGraphWeighted<T, u64>;

/// Which way a reachability traversal follows edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Reach {
    /// Follow outbound edges: the descendants of the start.
    Descendants,
    /// Follow inbound edges: the ancestors of the start.
    Ancestors,
}

/// The set of nodes reachable from `start`, inclusive of `start`, as a `slot_bound`-length mask.
/// `slot_bound` is `index_bound()`, one past the highest live index, so every position an edge
/// iterator yields fits; sizing by `number_nodes()` would not, once a lower causaloid was removed.
///
/// One body for the three pre-passes that used to open-code this traversal: the ancestor cone in
/// [`graph::CausableGraphReasoning::check_single_writer`] and the descendant pre-passes in both
/// reasoning engines. The two descendant copies were byte-identical; the ancestor one differed only
/// in the edge accessor.
///
/// # Two behaviours preserved deliberately
///
/// **A failed edge lookup stops the traversal rather than propagating.** The underlying graph
/// returns an error for a dynamic (not-frozen) graph, and every copy this replaces wrapped the
/// lookup in `if let Ok(..)`, so such a graph yields no edges and the walk simply ends. That is the
/// established behaviour of all three call sites and is reproduced here on purpose, not inherited by
/// accident.
///
/// **A `start` at or past `slot_bound` panics.** `seen[start]` indexes directly, as all three copies
/// did. Returning an empty mask instead would be a new behaviour at a site whose callers have
/// always validated the index before calling — `check_single_writer` rejects writers that hold no
/// causaloid, and both engines check `contains_causaloid` on the start and on every relay target.
///
/// # Direction, not a closure
///
/// `inbound_edges` and `outbound_edges` return `impl Iterator`, so their opaque types differ and a
/// closure seam would have to allocate a `Vec` per node to unify them. Matching on [`Reach`] inside
/// the loop keeps one body and allocates nothing beyond the mask and the stack.
pub(crate) fn reachable_mask<T: Clone>(
    graph: &CausalGraph<T>,
    start: usize,
    slot_bound: usize,
    reach: Reach,
) -> Vec<bool> {
    use ultragraph::GraphTraversal;

    let mut seen = vec![false; slot_bound];
    seen[start] = true;
    let mut stack = vec![start];
    while let Some(node) = stack.pop() {
        match reach {
            Reach::Descendants => {
                if let Ok(next) = graph.outbound_edges(node) {
                    for n in next {
                        if !seen[n] {
                            seen[n] = true;
                            stack.push(n);
                        }
                    }
                }
            }
            Reach::Ancestors => {
                if let Ok(next) = graph.inbound_edges(node) {
                    for n in next {
                        if !seen[n] {
                            seen[n] = true;
                            stack.push(n);
                        }
                    }
                }
            }
        }
    }
    seen
}

#[cfg(test)]
mod reachable_mask_tests {
    use super::{CausalGraph, Reach, reachable_mask};
    use ultragraph::{GraphMut, GraphView};

    /// `0 → 1 → 2`, plus an isolated `3`. Frozen, since the traversal reads edges.
    fn chain() -> CausalGraph<usize> {
        let mut g: CausalGraph<usize> = CausalGraph::with_capacity(4, None);
        for v in 0..4 {
            g.add_node(v).unwrap();
        }
        g.add_edge(0, 1, 1u64).unwrap();
        g.add_edge(1, 2, 1u64).unwrap();
        g.freeze();
        g
    }

    #[test]
    fn test_descendants_reach_transitively_not_just_the_start() {
        // The whole point of the pre-pass: `2` is reachable from `0` only through `1`. A traversal
        // that failed to expand would mark the start alone and still satisfy every call site's
        // existing test, which is what a surviving mutant showed.
        let g = chain();
        assert_eq!(
            reachable_mask(&g, 0, 4, Reach::Descendants),
            vec![true, true, true, false]
        );
    }

    #[test]
    fn test_ancestors_reach_transitively() {
        let g = chain();
        assert_eq!(
            reachable_mask(&g, 2, 4, Reach::Ancestors),
            vec![true, true, true, false]
        );
    }

    #[test]
    fn test_the_two_directions_are_not_the_same_walk() {
        // Descendants of `1` are {1,2}; its ancestors are {0,1}. A helper that ignored the
        // direction would return the same mask for both.
        let g = chain();
        assert_eq!(
            reachable_mask(&g, 1, 4, Reach::Descendants),
            vec![false, true, true, false]
        );
        assert_eq!(
            reachable_mask(&g, 1, 4, Reach::Ancestors),
            vec![true, true, false, false]
        );
    }

    #[test]
    fn test_an_isolated_node_reaches_only_itself() {
        let g = chain();
        assert_eq!(
            reachable_mask(&g, 3, 4, Reach::Descendants),
            vec![false, false, false, true]
        );
    }

    #[test]
    fn test_a_dynamic_graph_yields_no_edges_and_stops_at_the_start() {
        // Not frozen: the edge lookup fails and the traversal ends, marking only the start. This
        // is the behaviour all three replaced copies had through `if let Ok(..)`, preserved here
        // deliberately rather than inherited.
        let mut g: CausalGraph<usize> = CausalGraph::with_capacity(4, None);
        for v in 0..4 {
            g.add_node(v).unwrap();
        }
        g.add_edge(0, 1, 1u64).unwrap();
        assert_eq!(
            reachable_mask(&g, 0, 4, Reach::Descendants),
            vec![true, false, false, false]
        );
    }

    /// `0 → 1 → 2 → 3` with `0 → 2` beside it, node 1 removed before the freeze. Positions are
    /// permanent, so the live nodes sit at 0, 2 and 3: `number_nodes` is 3 and `index_bound` is 4.
    fn chain_with_removed_middle() -> CausalGraph<usize> {
        let mut g: CausalGraph<usize> = CausalGraph::with_capacity(4, None);
        for v in 0..4 {
            g.add_node(v).unwrap();
        }
        g.add_edge(0, 1, 1u64).unwrap();
        g.add_edge(1, 2, 1u64).unwrap();
        g.add_edge(2, 3, 1u64).unwrap();
        g.add_edge(0, 2, 1u64).unwrap();
        g.remove_node(1).unwrap();
        g.freeze();
        g
    }

    #[test]
    fn test_descendants_skip_a_removed_position_and_reach_past_number_nodes() {
        // Mask sized by `index_bound` = 4. From 0: 2 through `0 → 2`, then 3 through `2 → 3`;
        // position 1 holds no node and `0 → 1` is gone with it.
        let g = chain_with_removed_middle();
        assert_eq!(g.number_nodes(), 3);
        assert_eq!(
            reachable_mask(&g, 0, 4, Reach::Descendants),
            vec![true, false, true, true]
        );
    }

    #[test]
    fn test_a_start_at_a_live_index_at_or_above_number_nodes_walks_its_ancestors() {
        // 3 >= number_nodes() = 3. Ancestors of 3: 2 through `2 → 3`, then 0 through `0 → 2`;
        // the removed 1 is neither marked nor walked through.
        let g = chain_with_removed_middle();
        assert!(3 >= g.number_nodes());
        assert_eq!(
            reachable_mask(&g, 3, 4, Reach::Ancestors),
            vec![true, false, true, true]
        );
    }
}
