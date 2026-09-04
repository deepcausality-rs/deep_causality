/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Chordality of the undirected projection of a `MixedGraph`.
//!
//! A graph is chordal when it has no induced cycle of length four or more — equivalently, when
//! every cycle of length four or more has a chord.
//!
//! Only `Undirected` edges are read. Directed, bidirected and partially directed edges take no
//! part, so what is tested is the projection whose connected components are a CPDAG's chain
//! components.
//!
//! Chordality is the precondition of clique-picking, which counts acyclic moral orientations and is
//! defined only on a chordal graph.

use crate::{EdgeKind, MixedGraph, TopologyError, TopologyErrorEnum};
use std::collections::VecDeque;

impl<T> MixedGraph<T> {
    /// Returns `Ok(())` when every connected component of the undirected projection is chordal.
    ///
    /// On failure the error names the offending chordless cycle.
    pub fn undirected_projection_is_chordal(&self) -> Result<(), TopologyError> {
        match self.find_chordless_cycle() {
            None => Ok(()),
            Some(cycle) => Err(TopologyError(TopologyErrorEnum::GraphError(format!(
                "undirected projection is not chordal: chordless cycle {cycle:?}"
            )))),
        }
    }

    /// The vertices of a chordless cycle of length four or more, in cycle order, or `None` when
    /// the undirected projection is chordal.
    pub fn find_chordless_cycle(&self) -> Option<Vec<usize>> {
        // A shortest cycle through `v` whose two neighbours on it are non-adjacent has no chord: a
        // chord would cut it into a shorter such cycle. So it is enough to take, over every vertex
        // and every non-adjacent pair of its neighbours, the shortest connecting path that avoids
        // the rest of the neighbourhood.
        let mut best: Option<Vec<usize>> = None;
        for v in 0..self.num_vertices() {
            let neighbors = self.undirected_neighbors(v);
            for i in 0..neighbors.len() {
                for j in (i + 1)..neighbors.len() {
                    let (u, w) = (neighbors[i], neighbors[j]);
                    // Only an undirected edge is a chord of the projection: `is_adjacent` would
                    // also count directed, bidirected and partially directed edges, which the
                    // projection does not contain.
                    if self.edge_kind(u, w) == Some(EdgeKind::Undirected) {
                        continue;
                    }
                    // Excluding v and its other neighbours keeps the path from taking a shortcut
                    // through the neighbourhood, which is what would give the cycle a chord.
                    let blocked: Vec<usize> = neighbors
                        .iter()
                        .copied()
                        .filter(|&x| x != u && x != w)
                        .chain(std::iter::once(v))
                        .collect();
                    let Some(path) = self.shortest_undirected_path(u, w, &blocked) else {
                        continue;
                    };
                    // `path` runs u..w, so the cycle is v + path and has length path.len() + 1. It
                    // is a genuine cycle of length four or more because u and w are non-adjacent,
                    // which forces at least one vertex between them.
                    if path.len() < 3 {
                        continue;
                    }
                    let mut cycle = Vec::with_capacity(path.len() + 1);
                    cycle.push(v);
                    cycle.extend(path);
                    if best.as_ref().is_none_or(|b| cycle.len() < b.len()) {
                        best = Some(cycle);
                    }
                }
            }
        }
        best
    }

    /// The shortest path from `from` to `to` over undirected edges, avoiding every vertex in
    /// `blocked`, as a vertex list including both endpoints. `None` when none exists.
    fn shortest_undirected_path(
        &self,
        from: usize,
        to: usize,
        blocked: &[usize],
    ) -> Option<Vec<usize>> {
        let n = self.num_vertices();
        let mut prev = vec![usize::MAX; n];
        let mut seen = vec![false; n];
        for &b in blocked {
            seen[b] = true;
        }
        if seen[from] || seen[to] {
            return None;
        }
        seen[from] = true;
        let mut queue = VecDeque::from([from]);
        while let Some(x) = queue.pop_front() {
            if x == to {
                let mut path = vec![to];
                let mut cur = to;
                while cur != from {
                    cur = prev[cur];
                    path.push(cur);
                }
                path.reverse();
                return Some(path);
            }
            for y in self.undirected_neighbors(x) {
                if !seen[y] {
                    seen[y] = true;
                    prev[y] = x;
                    queue.push_back(y);
                }
            }
        }
        None
    }
}
