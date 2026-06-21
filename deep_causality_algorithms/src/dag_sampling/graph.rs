/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A minimal undirected graph: the input type of the clique-picking counter.
//!
//! Ported (counting-only API) from the authoritative `cliquepicking_rs::graph`.
//! This is a `usize`-indexed undirected graph that the counter operates on,
//! intentionally decoupled from [`deep_causality_topology::MixedGraph`] so the
//! ported algorithm stays a faithful, self-contained copy of the reference. The
//! public [`mec_size`](crate::dag_sampling::mec_size) entry point translates a
//! `MixedGraph` into this type; [`count_amos`](crate::dag_sampling::count_amos)
//! and [`count_chordal`](crate::dag_sampling::count_chordal) take it directly, so
//! it is re-exported as the public input type.
//!
//! Each undirected edge is expected to be given exactly once in the edge list.

use crate::dag_sampling::index_set::IndexSet;
use std::collections::VecDeque;

/// An undirected graph stored as per-vertex sorted neighbor sets — the input to
/// the Clique-Picking counter.
#[derive(Clone, Debug)]
pub struct Graph {
    /// Number of vertices.
    pub(crate) n: usize,
    /// Number of (undirected) edges.
    pub(crate) m: usize,
    /// Adjacency: `neighbors[u]` is the sorted set of vertices adjacent to `u`.
    neighbors: Vec<IndexSet>,
}

impl Graph {
    /// Builds a graph from an adjacency list (each inner vector lists a vertex's
    /// neighbors). The edge count is derived as half the total degree.
    pub fn from_adjacency_list(adjacency_list: Vec<Vec<usize>>) -> Graph {
        let n = adjacency_list.len();
        let m = adjacency_list.iter().map(Vec::len).sum::<usize>() / 2;
        let neighbors = adjacency_list.into_iter().map(IndexSet::from).collect();
        Graph { n, m, neighbors }
    }

    /// Builds a graph on `n` vertices from an undirected edge list. Each edge is
    /// assumed to appear once; it is inserted in both directions.
    pub fn from_edge_list(edge_list: Vec<(usize, usize)>, n: usize) -> Graph {
        let mut adjacency_list = vec![Vec::new(); n];
        for &(u, v) in &edge_list {
            adjacency_list[u].push(v);
            adjacency_list[v].push(u);
        }
        Graph::from_adjacency_list(adjacency_list)
    }

    /// Iterates over the (sorted) neighbors of vertex `u`.
    pub(crate) fn neighbors(&self, u: usize) -> std::slice::Iter<'_, usize> {
        self.neighbors[u].iter()
    }

    /// Returns a breadth-first vertex ordering starting from vertex `0`.
    /// Assumes the graph is connected (used on clique trees, which are trees).
    pub(crate) fn bfs_ordering(&self) -> Vec<usize> {
        let mut queue = VecDeque::new();
        let mut visited = vec![false; self.n];
        let mut visit_ordering = Vec::new();

        queue.push_back(0);
        visited[0] = true;

        while let Some(u) = queue.pop_front() {
            visit_ordering.push(u);
            for &v in self.neighbors(u) {
                if !visited[v] {
                    queue.push_back(v);
                    visited[v] = true;
                }
            }
        }
        visit_ordering
    }

    /// Splits the graph into its connected components, each renumbered to a fresh
    /// `0..component_size` vertex range. Returns one `Graph` per component.
    pub(crate) fn connected_components(&self) -> Vec<Graph> {
        let mut queue = VecDeque::new();
        let mut component_of = vec![usize::MAX; self.n];
        let mut new_id = vec![usize::MAX; self.n];
        let mut cnt = 0;

        let mut component_vertices: Vec<Vec<usize>> = Vec::new();
        for i in 0..self.n {
            if component_of[i] == usize::MAX {
                let mut component = Vec::new();
                queue.push_back(i);
                component_of[i] = cnt;
                new_id[i] = component.len();
                component.push(i);
                while let Some(u) = queue.pop_front() {
                    for &v in self.neighbors(u) {
                        if component_of[v] == usize::MAX {
                            queue.push_back(v);
                            component_of[v] = cnt;
                            new_id[v] = component.len();
                            component.push(v);
                        }
                    }
                }
                component_vertices.push(component);
                cnt += 1;
            }
        }

        let mut adjacency_lists: Vec<Vec<Vec<usize>>> = component_vertices
            .iter()
            .map(|component| vec![Vec::new(); component.len()])
            .collect();
        for i in 0..self.n {
            for &j in self.neighbors(i) {
                adjacency_lists[component_of[i]][new_id[i]].push(new_id[j]);
            }
        }

        adjacency_lists
            .into_iter()
            .map(Graph::from_adjacency_list)
            .collect()
    }
}
