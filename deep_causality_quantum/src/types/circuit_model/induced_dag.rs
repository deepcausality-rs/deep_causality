/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::collections::{BTreeSet, VecDeque};
use alloc::vec;
use alloc::vec::Vec;

/// The open DAG a compositional model induces (Lorenz & Tull, arXiv:2602.16612, Example 61):
/// one vertex per node of the model's grouping, one edge per wire leaving one node and entering
/// another. Derived, never stored beside the model.
///
/// The graph is a plain adjacency set with the reachability queries the abstraction layer needs:
/// parents and children for the structural precheck, blocked reachability for Definition 49's
/// `α(X)`, plain reachability for the parallelisable test of §7.2, and a cycle test for `build()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InducedDag {
    n: usize,
    edges: BTreeSet<(usize, usize)>,
}

impl InducedDag {
    /// An edgeless graph on `n` vertices.
    pub fn new(n: usize) -> Self {
        Self {
            n,
            edges: BTreeSet::new(),
        }
    }

    /// Adds the edge `a → b`. Out-of-range endpoints are ignored, so a caller building from a
    /// validated model never sees a panic here.
    pub fn add_edge(&mut self, a: usize, b: usize) -> &mut Self {
        if a < self.n && b < self.n {
            self.edges.insert((a, b));
        }
        self
    }

    /// The vertex count.
    pub fn num_vertices(&self) -> usize {
        self.n
    }

    /// The edges, ascending.
    pub fn edges(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.edges.iter().copied()
    }

    /// The edge count.
    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    /// Whether `a → b` is an edge.
    pub fn has_edge(&self, a: usize, b: usize) -> bool {
        self.edges.contains(&(a, b))
    }

    /// The parents of `v`, ascending.
    pub fn parents(&self, v: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(_, b)| *b == v)
            .map(|(a, _)| *a)
            .collect()
    }

    /// The children of `v`, ascending.
    pub fn children(&self, v: usize) -> Vec<usize> {
        self.edges
            .range((v, 0)..(v, self.n.max(1)))
            .map(|(_, b)| *b)
            .collect()
    }

    /// Whether a directed path of at least one edge runs from `a` to `b`.
    pub fn reaches(&self, a: usize, b: usize) -> bool {
        let mut seen = vec![false; self.n];
        let mut queue: VecDeque<usize> = self.children(a).into_iter().collect();
        while let Some(v) = queue.pop_front() {
            if v == b {
                return true;
            }
            if !seen[v] {
                seen[v] = true;
                queue.extend(self.children(v));
            }
        }
        false
    }

    /// Whether some vertex of `targets` is reached from `start` along a directed path none of
    /// whose vertices, `start` included, lies in `blocked`. A `start` in `targets` counts through
    /// the empty path, which is Definition 49's `π(X) ⊆ α(X)`.
    pub fn reaches_avoiding(
        &self,
        start: usize,
        targets: &BTreeSet<usize>,
        blocked: &BTreeSet<usize>,
    ) -> bool {
        if blocked.contains(&start) || start >= self.n {
            return false;
        }
        if targets.contains(&start) {
            return true;
        }
        let mut seen = vec![false; self.n];
        seen[start] = true;
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        while let Some(v) = queue.pop_front() {
            for c in self.children(v) {
                if blocked.contains(&c) || seen[c] {
                    continue;
                }
                if targets.contains(&c) {
                    return true;
                }
                seen[c] = true;
                queue.push_back(c);
            }
        }
        false
    }

    /// Whether the graph has a directed cycle.
    pub fn has_cycle(&self) -> bool {
        self.topological_order().is_none()
    }

    /// A topological order, or `None` if the graph has a cycle. Kahn's algorithm, ties broken by
    /// ascending vertex.
    pub fn topological_order(&self) -> Option<Vec<usize>> {
        let mut indegree = vec![0usize; self.n];
        for (_, b) in &self.edges {
            indegree[*b] += 1;
        }
        let mut ready: BTreeSet<usize> = (0..self.n).filter(|v| indegree[*v] == 0).collect();
        let mut order = Vec::with_capacity(self.n);
        while let Some(v) = ready.iter().next().copied() {
            ready.remove(&v);
            order.push(v);
            for c in self.children(v) {
                indegree[c] -= 1;
                if indegree[c] == 0 {
                    ready.insert(c);
                }
            }
        }
        (order.len() == self.n).then_some(order)
    }
}
