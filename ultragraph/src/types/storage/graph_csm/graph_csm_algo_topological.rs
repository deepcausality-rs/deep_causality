/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{CsmGraph, EdgeKey, GraphError, TopologicalGraphAlgorithms};
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

/// Private enum representing the state of a node during a DFS traversal.
/// It is not part of the public API.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NodeState {
    /// The node has not yet been visited.
    Unvisited,
    /// The node is currently in the recursion stack (being visited).
    VisitingInProgress,
    /// The node and all its descendants have been fully visited and are known to be cycle-free.
    Visited,
}

impl<N, W> TopologicalGraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Finds a cycle in the graph using an iterative Depth-First Search (DFS).
    ///
    /// This method traverses the graph to detect a back edge, which indicates a cycle.
    /// The iterative approach using an explicit stack makes it robust against stack
    /// overflows, even for very deep or large graphs. The search is comprehensive and
    /// will find a cycle in any of the graph's disconnected components if one exists.
    ///
    /// # Returns
    ///
    /// - `Some(Vec<usize>)`: If a cycle is found, this returns a vector of node indices
    ///   that form the cycle. The path explicitly starts and ends with the same node
    ///   to represent the closed loop (e.g., `[1, 2, 0, 1]`). For a self-loop on
    ///   node `n`, it returns `vec![n, n]`.
    /// - `None`: If the graph is a Directed Acyclic Graph (DAG) and contains no cycles.
    ///
    /// # Complexity
    ///
    /// - **Time Complexity:** O(V + E), where V is the number of nodes and E is the
    ///   number of edges, as each node and edge is visited exactly once.
    /// - **Space Complexity:** O(V) for storing node states, predecessors, and the DFS stack.
    ///
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        find_cycle_impl(self)
    }

    /// Checks if the graph contains any directed cycles.
    ///
    /// This implementation leverages the iterative `topological_sort` method,
    /// making it a highly robust way to detect cycles in graphs of any size
    /// without risking a stack overflow.
    fn has_cycle(&self) -> Result<bool, GraphError> {
        has_cycle_impl(self)
    }

    /// Computes a topological sort of the graph using the iterative Kahn's algorithm.
    ///
    /// This method is robust against stack overflows, making it suitable for graphs
    /// of any size. It works by repeatedly finding nodes with no incoming edges.
    ///
    /// # Returns
    /// - `Some(Vec<usize>)` if the graph is a Directed Acyclic Graph (DAG). The
    ///   vector contains the nodes in a valid topological order.
    /// - `None` if the graph contains a cycle, as a topological sort is not possible.
    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        topological_sort_impl(self)
    }
}

/// Depth-first search over the passing edges. A parallel edge reaches a node in the same state
/// as the first one did, so the result does not depend on the mode.
pub(crate) fn find_cycle_impl<W, A: Adjacency<W>>(a: &A) -> Result<Option<Vec<usize>>, GraphError> {
    let num_nodes = a.slots();
    if num_nodes == 0 {
        return Ok(None);
    }

    let mut states = vec![NodeState::Unvisited; num_nodes];
    let mut predecessors = vec![None; num_nodes];

    for i in 0..num_nodes {
        if a.passes(i) && states[i] == NodeState::Unvisited {
            let mut stack: Vec<(usize, _)> = Vec::new();

            states[i] = NodeState::VisitingInProgress;
            stack.push((i, a.out_edges(i)));

            while let Some((u_ref, neighbors)) = stack.last_mut() {
                let u = *u_ref;

                if let Some((v, _)) = neighbors.next() {
                    if states[v] == NodeState::VisitingInProgress {
                        // --- Cycle Found: Reconstruct the Path ---
                        let mut path = vec![u];
                        let mut current = u;

                        // A self-loop closes at `u` itself; any other cycle is walked back through
                        // the predecessors until `v`.
                        while current != v {
                            let Some(predecessor) = predecessors[current] else {
                                break;
                            };
                            path.push(predecessor);
                            current = predecessor;
                        }
                        path.reverse();
                        path.push(v); // Make the cycle explicit: [v, ..., u, v]
                        return Ok(Some(path));
                    }

                    if states[v] == NodeState::Unvisited {
                        predecessors[v] = Some(u);
                        states[v] = NodeState::VisitingInProgress;
                        stack.push((v, a.out_edges(v)));
                    }
                } else {
                    // If the neighbor iterator is exhausted, we are done with this node.
                    states[u] = NodeState::Visited;
                    stack.pop();
                }
            }
        }
    }

    Ok(None) // No cycles found after checking all nodes.
}

pub(crate) fn has_cycle_impl<W, A: Adjacency<W>>(a: &A) -> Result<bool, GraphError> {
    topological_sort_impl(a).map(|order| order.is_none())
}

/// Kahn's algorithm over the passing edges. Every edge counts once in the in-degree it raises
/// and once in the decrement that lowers it, so the result does not depend on the mode.
pub(crate) fn topological_sort_impl<W, A: Adjacency<W>>(
    a: &A,
) -> Result<Option<Vec<usize>>, GraphError> {
    // Arrays are sized by position; the passing count decides whether every node was sorted.
    let live = a.passing_count();
    let num_nodes = a.slots();
    if live == 0 {
        // An empty graph is trivially sorted.
        return Ok(Some(Vec::new()));
    }

    // 1. Compute in-degrees for all nodes.
    let mut in_degrees = vec![0; num_nodes];
    for i in 0..num_nodes {
        for (neighbor, _) in a.out_edges(i) {
            in_degrees[neighbor] += 1;
        }
    }

    // 2. Initialize a queue with all nodes that have an in-degree of 0.
    let mut queue: VecDeque<usize> = (0..num_nodes)
        .filter(|&i| a.passes(i) && in_degrees[i] == 0)
        .collect();

    // 3. Process the queue.
    let mut sorted_list = Vec::with_capacity(live);
    while let Some(u) = queue.pop_front() {
        sorted_list.push(u);

        // For each neighbor of the dequeued node, decrement its in-degree.
        for (v, _) in a.out_edges(u) {
            in_degrees[v] -= 1;
            if in_degrees[v] == 0 {
                queue.push_back(v);
            }
        }
    }

    // 4. Validate the result.
    if sorted_list.len() == live {
        Ok(Some(sorted_list))
    } else {
        Ok(None) // Cycle detected
    }
}
