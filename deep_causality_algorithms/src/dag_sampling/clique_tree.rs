/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The clique tree of a chordal graph and its derived structures.
//!
//! Ported (counting-only API) from the authoritative
//! `cliquepicking_rs::clique_tree`. From an MCS perfect-elimination ordering this
//! builds the maximal cliques and the junction (clique) tree, then exposes the
//! quantities the counting recursion needs:
//!
//! * [`separators`](CliqueTree::separators) — one separator (clique∩clique) per
//!   directed tree edge, indexed by [`get_edge_id`](CliqueTree::get_edge_id),
//! * [`flowers`](CliqueTree::flowers) — the "flower" of each directed edge: the
//!   sub-tree reachable through that separator,
//! * [`forbidden_sets`](CliqueTree::forbidden_sets) — per-clique forbidden
//!   separator sizes used by `rho`.
//!
//! All indices are `usize`; this module carries no count type. Chordality of the
//! input is assumed, not checked.

use crate::dag_sampling::chordal;
use crate::dag_sampling::graph::Graph;
use crate::dag_sampling::index_set::IndexSet;
use crate::dag_sampling::utils;
use std::collections::VecDeque;

/// The clique (junction) tree of a chordal graph.
#[derive(Clone, Debug)]
pub(crate) struct CliqueTree {
    /// The maximal cliques, each as a sorted index set over the original graph.
    pub(crate) cliques: Vec<IndexSet>,
    /// The tree over clique nodes (`tree.n == cliques.len()`).
    pub(crate) tree: Graph,
    /// BFS rank of each clique node; used to assign stable directed edge ids.
    rank: Vec<usize>,
    /// BFS distance from the root clique; used to orient forbidden-set scans.
    dists: Vec<usize>,
}

impl CliqueTree {
    /// Builds the clique tree of `g` from its MCS perfect-elimination ordering.
    /// Assumes `g` is connected and chordal.
    pub(crate) fn from(g: &Graph) -> CliqueTree {
        let mcs_ordering = chordal::mcs(g);
        let inv_mcs_ordering = utils::inverse_permutation(&mcs_ordering);

        let mut cliques = Vec::new();
        let mut current_clique = Vec::new();
        let mut tree_edges = Vec::new();

        let mut num_cliques = 0;
        let mut visited = vec![false; g.n];
        let mut clique_id = vec![0; g.n];

        for &u in &mcs_ordering {
            let visited_neighbors =
                IndexSet::from(g.neighbors(u).cloned().filter(|&v| visited[v]).collect());
            if !visited_neighbors.equal_to_vec(&current_clique) {
                num_cliques += 1;
                let k = visited_neighbors
                    .iter()
                    .copied()
                    .max_by_key(|&x| inv_mcs_ordering[x])
                    .unwrap(); // safe if g is connected
                let p = clique_id[k];
                tree_edges.push((p, num_cliques));
                cliques.push(IndexSet::from(current_clique));
                current_clique = visited_neighbors.to_vec();
            }
            current_clique.push(u);
            clique_id[u] = num_cliques;
            visited[u] = true;
        }

        // Assumes a non-empty graph.
        cliques.push(IndexSet::from(current_clique));
        let tree = Graph::from_edge_list(tree_edges, cliques.len());

        // Compute clique ranks (inverse of BFS order).
        let bfs_ordering = tree.bfs_ordering();
        let rank = utils::inverse_permutation(&bfs_ordering);

        // Compute BFS distances from the root clique.
        let mut tree_visited = vec![false; tree.n];
        let mut queue = VecDeque::new();
        queue.push_back(0);
        tree_visited[0] = true;

        let mut dists = vec![0; tree.n];
        while let Some(u) = queue.pop_front() {
            for &v in tree.neighbors(u) {
                if !tree_visited[v] {
                    tree_visited[v] = true;
                    dists[v] = dists[u] + 1;
                    queue.push_back(v);
                }
            }
        }

        CliqueTree {
            cliques,
            tree,
            rank,
            dists,
        }
    }

    /// Maps an undirected tree edge `{u, v}` to one of its two directed edge ids.
    /// The id is stable per direction and indexes the separator/flower tables.
    pub(crate) fn get_edge_id(&self, u: usize, v: usize) -> usize {
        if self.rank[u] < self.rank[v] {
            2 * (self.rank[v] - 1) + 1
        } else {
            2 * (self.rank[u] - 1)
        }
    }

    /// Returns the separator (intersection of the two incident cliques) for every
    /// directed tree edge, indexed by [`get_edge_id`](CliqueTree::get_edge_id).
    pub(crate) fn separators(&self) -> Vec<IndexSet> {
        let mut separators = vec![IndexSet::new(); 2 * (self.tree.n - 1)];
        for u in 0..self.tree.n {
            for &v in self.tree.neighbors(u) {
                separators[self.get_edge_id(u, v)] = self.cliques[u].intersection(&self.cliques[v]);
            }
        }
        separators
    }

    /// Returns the "flower" of every directed tree edge: starting from the head
    /// clique, the connected set of cliques reachable while still containing the
    /// edge's separator and crossing only edges with a *different* separator.
    pub(crate) fn flowers(&self, separators: &[IndexSet]) -> Vec<IndexSet> {
        let mut flowers = vec![IndexSet::new(); 2 * (self.tree.n - 1)];
        let mut visited = vec![false; self.tree.n];

        for s in 0..self.tree.n {
            for &t in self.tree.neighbors(s) {
                let edge_id = self.get_edge_id(s, t);
                let st_sep = &separators[edge_id];
                let mut flower = Vec::new();
                flower.push(t);
                visited[s] = true;
                visited[t] = true;
                let mut q = VecDeque::new();
                q.push_back(t);
                while let Some(u) = q.pop_front() {
                    for &v in self.tree.neighbors(u) {
                        if !visited[v]
                            && st_sep.is_subset(&self.cliques[v])
                            && separators[self.get_edge_id(u, v)] != *st_sep
                        {
                            flower.push(v);
                            visited[v] = true;
                            q.push_back(v);
                        }
                    }
                }
                visited[s] = false;
                for &f in &flower {
                    visited[f] = false;
                }
                flowers[edge_id] = IndexSet::from(flower);
            }
        }
        flowers
    }

    /// Returns, per clique, the list of `(u, v, separator_size)` triples for the
    /// directed tree edges whose flower contains that clique (scanned in the
    /// root-ward direction), sorted by descending separator size.
    pub(crate) fn forbidden_sets(
        &self,
        separators: &[IndexSet],
        flowers: &[IndexSet],
    ) -> Vec<Vec<(usize, usize, usize)>> {
        let mut forbidden_sets: Vec<Vec<(usize, usize, usize)>> = vec![Vec::new(); self.tree.n];

        for u in 0..self.tree.n {
            for &v in self.tree.neighbors(u) {
                if self.dists[u] > self.dists[v] {
                    continue;
                }
                let edge_id = self.get_edge_id(u, v);
                for &clique_id in &flowers[edge_id] {
                    forbidden_sets[clique_id].push((u, v, separators[edge_id].len()));
                }
            }
        }

        for forbidden_set in forbidden_sets.iter_mut() {
            forbidden_set.sort_by_key(|x| x.2);
            forbidden_set.reverse();
        }

        forbidden_sets
    }
}
