/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Meek orientation rules (Meek 1995) for completing a PDAG to its CPDAG.
//!
//! Repeatedly orients undirected edges that are *compelled* — forced by the
//! two structural constraints "no new unshielded collider" and "no directed
//! cycle" — until a fixpoint is reached. The result is the unique maximally
//! oriented graph (CPDAG) for the input pattern.
//!
//! Rules R1–R3 are implemented; R4 is **deliberately omitted**, for parity with
//! the reference. R4 is only required to complete a graph that already carries
//! background-knowledge orientations (orientations not arising from
//! v-structures). The Python BRCD reference calls
//! `graphical_models.PDAG.to_complete_pdag` (uhlerlab, MIT —
//! <https://github.com/uhlerlab/graphical_models>), whose completion applies
//! Meek R1–R3 only; this port is therefore at parity with the exact code the
//! reference runs. For a pattern of a DAG, R1–R3 are provably complete
//! (Meek 1995, Theorem 3). Should a future change need to maximally orient a
//! graph carrying background knowledge, R4 would be added then — but that is a
//! deliberate divergence from the reference, not a defect of this port.
//!
//! Each undirected edge `a — b` below is oriented into `a → b` when a rule fires.

use deep_causality_topology::{EdgeKind, MixedGraph};
use std::collections::BTreeSet;

/// Completes a PDAG to its CPDAG in place by closing under Meek rules R1–R3.
///
/// The graph's directed arcs and undirected edges are read through `MixedGraph`'s
/// projection accessors; compelled undirected edges are oriented via `orient`.
/// Undirected edges that no rule forces remain undirected. Terminates because
/// every pass that changes anything orients at least one undirected edge, and
/// the undirected set only shrinks.
///
/// Completion assumes the input is an extendable PDAG. For a contradictory
/// (non-extendable) input where a rule compels *both* directions of an edge,
/// completion picks one and proceeds rather than signalling — consistency is
/// deferred to the caller's validity pass
/// ([`crate::brcd::validity::is_valid_configuration`]), which rejects the result
/// if completion produced a cycle or a new unshielded collider.
pub fn meek_complete<N>(graph: &mut MixedGraph<N>) {
    loop {
        let undirected = graph.undirected_edges();
        let mut changed = false;
        for (u, v) in undirected {
            // The edge may already have been oriented earlier in this pass.
            if graph.edge_kind(u, v) != Some(EdgeKind::Undirected) {
                continue;
            }
            if rule_forces(graph, u, v) {
                graph
                    .orient(u, v)
                    .expect("an undirected edge can always be oriented");
                changed = true;
            } else if rule_forces(graph, v, u) {
                graph
                    .orient(v, u)
                    .expect("an undirected edge can always be oriented");
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
}

/// True if any of Meek R1–R3 compels the undirected edge `a — b` to `a → b`.
fn rule_forces<N>(g: &MixedGraph<N>, a: usize, b: usize) -> bool {
    meek_r1(g, a, b) || meek_r2(g, a, b) || meek_r3(g, a, b)
}

/// R1 (no new collider): there is `k → a` with `k` not adjacent to `b`.
///
/// Orienting `b → a` would make `k → a ← b` a new unshielded collider.
fn meek_r1<N>(g: &MixedGraph<N>, a: usize, b: usize) -> bool {
    g.parents(a).into_iter().any(|k| !g.is_adjacent(k, b))
}

/// R2 (no cycle): there is a directed 2-path `a → k → b`.
///
/// Orienting `b → a` would close the cycle `a → k → b → a`.
fn meek_r2<N>(g: &MixedGraph<N>, a: usize, b: usize) -> bool {
    let parents_b: BTreeSet<usize> = g.parents(b).into_iter().collect();
    g.children(a).into_iter().any(|k| parents_b.contains(&k))
}

/// R3 (no new collider, via two R2 steps): there are `k1 → b`, `k2 → b` with
/// `a — k1`, `a — k2` undirected and `k1`, `k2` non-adjacent.
fn meek_r3<N>(g: &MixedGraph<N>, a: usize, b: usize) -> bool {
    let parents_b = g.parents(b);
    let undirected_a: BTreeSet<usize> = g.undirected_neighbors(a).into_iter().collect();
    for i in 0..parents_b.len() {
        for j in (i + 1)..parents_b.len() {
            let (k1, k2) = (parents_b[i], parents_b[j]);
            if undirected_a.contains(&k1) && undirected_a.contains(&k2) && !g.is_adjacent(k1, k2) {
                return true;
            }
        }
    }
    false
}
