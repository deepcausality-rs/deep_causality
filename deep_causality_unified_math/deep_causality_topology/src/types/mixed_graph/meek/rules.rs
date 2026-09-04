/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The four rule predicates.
//!
//! Each answers one question: does this rule compel the undirected edge `a — b` to `a → b`? None of
//! them mutates the graph; [`super`] applies the orientation.

use crate::MixedGraph;
use std::collections::BTreeSet;

impl<T> MixedGraph<T> {
    /// R1: some `c` has `c → a` and `c` not adjacent to `b`.
    pub(super) fn meek_r1(&self, a: usize, b: usize) -> bool {
        self.parents(a).into_iter().any(|c| !self.is_adjacent(c, b))
    }

    /// R2: some `c` has `a → c → b`.
    pub(super) fn meek_r2(&self, a: usize, b: usize) -> bool {
        let parents_b: BTreeSet<usize> = self.parents(b).into_iter().collect();
        self.children(a).into_iter().any(|c| parents_b.contains(&c))
    }

    /// R3: some `c, d` have `d — a — c` undirected, `d → b`, `c → b`, and `c` not adjacent to `d`.
    pub(super) fn meek_r3(&self, a: usize, b: usize) -> bool {
        let parents_b = self.parents(b);
        let undirected_a: BTreeSet<usize> = self.undirected_neighbors(a).into_iter().collect();
        for i in 0..parents_b.len() {
            for j in (i + 1)..parents_b.len() {
                let (c, d) = (parents_b[i], parents_b[j]);
                if undirected_a.contains(&c) && undirected_a.contains(&d) && !self.is_adjacent(c, d)
                {
                    return true;
                }
            }
        }
        false
    }

    /// R4: some `c, d` have `d — a — c` undirected, `d → c → b`, and `b` not adjacent to `d`.
    ///
    /// `c` and `d` are distinct by construction — `d → c` cannot hold for `c == d` — so the loops
    /// need no separate guard.
    pub(super) fn meek_r4(&self, a: usize, b: usize) -> bool {
        let undirected_a = self.undirected_neighbors(a);
        let parents_b: BTreeSet<usize> = self.parents(b).into_iter().collect();
        for &d in &undirected_a {
            if self.is_adjacent(b, d) {
                continue;
            }
            for &c in &undirected_a {
                if parents_b.contains(&c) && self.has_arc(d, c) {
                    return true;
                }
            }
        }
        false
    }

    /// True when `a → b` is present.
    pub(super) fn has_arc(&self, a: usize, b: usize) -> bool {
        self.children(a).contains(&b)
    }
}
