/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{CsmGraph, EdgeKey, GraphError, PathfindingGraphAlgorithms};
use alloc::collections::{BinaryHeap, VecDeque};
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::Reverse;

impl<N, W> PathfindingGraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Checks if a path exists from a start to a stop index.
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        is_reachable_impl(self, start_index, stop_index)
    }

    /// Returns the length of the shortest path (in number of nodes) from a start to a stop index.
    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        shortest_path_len_impl(self, start_index, stop_index)
    }

    /// Finds the complete shortest path from a start to a stop index.
    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        shortest_path_impl(self, start_index, stop_index)
    }

    fn shortest_weighted_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<(Vec<usize>, W)>, GraphError>
    where
        W: Copy + Ord + Default + core::ops::Add<Output = W>,
    {
        shortest_weighted_path_impl(self, start_index, stop_index)
    }
}

pub(crate) fn is_reachable_impl<W, A: Adjacency<W>>(
    a: &A,
    start_index: usize,
    stop_index: usize,
) -> Result<bool, GraphError> {
    shortest_path_len_impl(a, start_index, stop_index).map(|len| len.is_some())
}

/// Breadth-first search over the passing edges. A parallel edge reaches a node already visited
/// and changes nothing, so the result does not depend on the mode.
pub(crate) fn shortest_path_len_impl<W, A: Adjacency<W>>(
    a: &A,
    start_index: usize,
    stop_index: usize,
) -> Result<Option<usize>, GraphError> {
    if !a.passes(start_index) || !a.passes(stop_index) {
        return Ok(None);
    }
    if start_index == stop_index {
        return Ok(Some(1));
    }

    let mut queue = VecDeque::new();
    let mut visited = vec![false; a.slots()];

    queue.push_back((start_index, 1)); // (node, path_length)
    visited[start_index] = true;

    while let Some((current_node, current_len)) = queue.pop_front() {
        for (neighbor, _) in a.out_edges(current_node) {
            if neighbor == stop_index {
                return Ok(Some(current_len + 1));
            }
            if !visited[neighbor] {
                visited[neighbor] = true;
                queue.push_back((neighbor, current_len + 1));
            }
        }
    }
    Ok(None)
}

pub(crate) fn shortest_path_impl<W, A: Adjacency<W>>(
    a: &A,
    start_index: usize,
    stop_index: usize,
) -> Result<Option<Vec<usize>>, GraphError> {
    if !a.passes(start_index) || !a.passes(stop_index) {
        return Ok(None);
    }
    if start_index == stop_index {
        return Ok(Some(vec![start_index]));
    }

    let mut queue = VecDeque::new();
    let mut predecessors = vec![None; a.slots()];
    let mut visited = vec![false; a.slots()];

    queue.push_back(start_index);
    visited[start_index] = true;

    let mut found = false;
    'bfs_loop: while let Some(current_node) = queue.pop_front() {
        for (neighbor, _) in a.out_edges(current_node) {
            if !visited[neighbor] {
                visited[neighbor] = true;
                predecessors[neighbor] = Some(current_node);
                queue.push_back(neighbor);

                if neighbor == stop_index {
                    found = true;
                    break 'bfs_loop;
                }
            }
        }
    }

    if !found {
        return Ok(None);
    }

    // Reconstruct path by walking backwards from the stop index.
    let mut path = Vec::new();
    let mut current = Some(stop_index);
    while let Some(curr_index) = current {
        path.push(curr_index);
        current = predecessors[curr_index];
    }
    path.reverse();
    Ok(Some(path))
}

/// Dijkstra over the passing edges. Every parallel edge is relaxed, so the lightest one wins
/// under either mode.
pub(crate) fn shortest_weighted_path_impl<W, A: Adjacency<W>>(
    a: &A,
    start_index: usize,
    stop_index: usize,
) -> Result<Option<(Vec<usize>, W)>, GraphError>
where
    W: Copy + Ord + Default + core::ops::Add<Output = W>,
{
    if !a.passes(start_index) || !a.passes(stop_index) {
        return Ok(None);
    }

    if start_index == stop_index {
        return Ok(Some((vec![start_index], W::default())));
    }

    let num_nodes = a.slots();
    let mut distances: Vec<Option<W>> = vec![None; num_nodes];
    let mut predecessors: Vec<Option<usize>> = vec![None; num_nodes];
    let mut pq: BinaryHeap<(Reverse<W>, usize)> = BinaryHeap::new();

    distances[start_index] = Some(W::default());
    pq.push((Reverse(W::default()), start_index));

    while let Some((Reverse(dist), u)) = pq.pop() {
        if u == stop_index {
            let mut path = Vec::new();
            let mut current = Some(stop_index);
            while let Some(node) = current {
                path.push(node);
                if node == start_index {
                    break;
                }
                current = predecessors[node];
            }
            path.reverse();

            return Ok(Some((path, dist)));
        }

        if let Some(known_dist) = distances[u]
            && dist > known_dist
        {
            continue;
        }

        for (v, weight) in a.out_edges(u) {
            let new_dist = dist + *weight;

            if distances[v].is_none_or(|known| new_dist < known) {
                distances[v] = Some(new_dist);
                predecessors[v] = Some(u);
                pq.push((Reverse(new_dist), v));
            }
        }
    }

    Ok(None)
}
