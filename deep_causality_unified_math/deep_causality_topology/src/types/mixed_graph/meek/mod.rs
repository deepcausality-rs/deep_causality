/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Meek orientation rules (Meek 1995) over the mixed projection of a `MixedGraph`.
//!
//! A rule orients an undirected edge that is *compelled* — one whose direction is the same in every
//! consistent DAG extension. Applied to a fixpoint, they yield a maximally oriented PDAG.
//!
//! With `a — b` the edge under consideration, `∼` adjacency and `≁` non-adjacency, each rule
//! orients `a → b` when:
//!
//! - **R1** — some `c` has `c → a` and `c ≁ b`. Orienting `b → a` would make `c → a ← b` a new
//!   unshielded collider.
//! - **R2** — some `c` has `a → c → b`. Orienting `b → a` would close the cycle `a → c → b → a`.
//! - **R3** — some `c, d` have `d — a — c` undirected, `d → b`, `c → b`, and `c ≁ d`.
//! - **R4** — some `c, d` have `d — a — c` undirected, `d → c → b`, and `b ≁ d`.
//!
//! # Which closure to call
//!
//! The two differ in the hypothesis under which they are complete.
//!
//! [`MixedGraph::meek_complete`] applies all four. Sound and complete for any set of arcs admitting
//! a consistent DAG extension, which includes arcs that did not arise from v-structures —
//! background knowledge, a fixed edge orientation, an added indicator node.
//!
//! [`MixedGraph::meek_complete_r1_r3`] applies the first three. Complete only for the pattern of a
//! DAG, and it matches the Python reference `graphical_models.PDAG.to_complete_pdag` (uhlerlab,
//! MIT). Use it when reference parity is the requirement; otherwise the input's provenance has to
//! be known to be a pattern for its result to mean anything.
//!
//! # What is not checked
//!
//! Neither closure validates its input. A PDAG admitting no consistent DAG extension still gets an
//! orientation, and that orientation is meaningless.
//!
//! The sweep tries `a → b` before `b → a`, so an edge both directions compel is oriented the way
//! the sweep reached it — a consequence of iteration order, not a decision, and the closure does not
//! report the conflict. Such an edge is a symptom of non-extendability rather than a
//! characterisation of it: a PDAG can admit no extension with no single edge compelled both ways.

mod rules;

use crate::{EdgeKind, MixedGraph};

impl<T> MixedGraph<T> {
    /// Closes the graph under R1–R4 in place, reaching the maximally oriented PDAG.
    ///
    /// Edges no rule compels stay undirected. Terminates because a pass that changes anything
    /// orients at least one edge, and the undirected set only shrinks.
    pub fn meek_complete(&mut self) {
        self.close(true)
    }

    /// Closes the graph under R1–R3 only.
    ///
    /// Complete for the pattern of a DAG, not for one carrying background knowledge.
    pub fn meek_complete_r1_r3(&mut self) {
        self.close(false)
    }

    /// Sweeps the undirected edges, orienting those a rule compels, until a sweep changes nothing.
    fn close(&mut self, with_r4: bool) {
        loop {
            let mut changed = false;
            for (u, v) in self.undirected_edges() {
                // An earlier orientation in this same sweep may already have taken this edge.
                if self.edge_kind(u, v) != Some(EdgeKind::Undirected) {
                    continue;
                }
                let (u, v) = if self.compels(u, v, with_r4) {
                    (u, v)
                } else if self.compels(v, u, with_r4) {
                    (v, u)
                } else {
                    continue;
                };
                self.orient(u, v)
                    .expect("an undirected edge can always be oriented");
                changed = true;
            }
            if !changed {
                return;
            }
        }
    }

    /// True when any enabled rule compels `a — b` to `a → b`.
    fn compels(&self, a: usize, b: usize, with_r4: bool) -> bool {
        self.meek_r1(a, b)
            || self.meek_r2(a, b)
            || self.meek_r3(a, b)
            || (with_r4 && self.meek_r4(a, b))
    }
}
