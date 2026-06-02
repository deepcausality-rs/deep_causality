/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Markov-equivalence-class (MEC) sizing — trivial arcs-only case.
//!
//! Each I-CPDAG represents a class of DAGs that all entail the same likelihood.
//! BRCD weights a configuration by the **size** of its class, `p(G | R) =
//! mec_size / Σ`, and samples one representative DAG to score (the efficiency
//! trick: one DAG stands in for the whole class).
//!
//! The general count + uniform sample is the Wienöbst clique-picking algorithm
//! (`cliquepicking` in the reference). It is **deliberately deferred to the BRCD
//! estimator change.** This module ships only the **trivial case**: when the
//! graph is already fully directed (every edge a directed arc, no undirected
//! edges), the class has exactly one member, so the size is `1` and the
//! representative DAG is the input itself. Any input carrying an undirected edge
//! returns [`MecError::RequiresUniformSampler`] — it never silently guesses.
//!
//! **Scope (placeholder).** The trivial case is sufficient only for arcs-only
//! inputs: an already-directed CPDAG, or the service-map proxy used by the
//! `BRCD-C` variant. It is **not** sufficient for the general OB / Sock Shop
//! path: per `openspec/notes/rca/BRCD.md` §16.2, plain BRCD on OB / Sock Shop
//! consumes BOSS-learned CPDAGs (which carry undirected edges) and bootstrap
//! CPDAGs, so the full uniform clique-picking sampler is on that path too — not
//! only on Petshop. A faithful port must therefore port the Wienöbst sampler
//! (`mec_size` + uniform `sample_dag`) in the estimator change; this module is a
//! placeholder for it. The API ([`mec_size`], [`representative_dag`]) is shaped
//! so that sampler satisfies it without a signature change.

use deep_causality_topology::MixedGraph;

/// Reasons MEC sizing cannot return the trivial answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MecError {
    /// The graph still has undirected (or other non-directed) edges, so its
    /// equivalence class has more than one member. Counting/sampling it
    /// requires the uniform (clique-picking) sampler, which is not implemented
    /// in this change.
    RequiresUniformSampler,
    /// The directed-arc projection contains a cycle, so the input is not a DAG.
    NotAcyclic,
}

/// Returns `true` if every edge of `graph` is a directed arc (no undirected,
/// bidirected, or circle edges).
fn is_fully_directed<N>(graph: &MixedGraph<N>) -> bool {
    graph.num_edges() == graph.arcs().len()
}

/// Returns the size of the Markov equivalence class of `graph`.
///
/// Trivial case only: for a fully directed acyclic graph the size is `1`.
///
/// # Errors
/// * `RequiresUniformSampler` if the graph has any non-directed edge (the
///   general count needs the deferred uniform sampler).
/// * `NotAcyclic` if the arc projection contains a cycle.
pub fn mec_size<N>(graph: &MixedGraph<N>) -> Result<usize, MecError> {
    if !is_fully_directed(graph) {
        return Err(MecError::RequiresUniformSampler);
    }
    if graph.has_cycle() {
        return Err(MecError::NotAcyclic);
    }
    Ok(1)
}

/// Returns a representative DAG from the Markov equivalence class of `graph`.
///
/// Trivial case only: for a fully directed acyclic graph the representative is
/// the graph itself (returned as a clone). Errors mirror [`mec_size`].
pub fn representative_dag<N: Clone>(graph: &MixedGraph<N>) -> Result<MixedGraph<N>, MecError> {
    // Validates fully-directed + acyclic; the size is necessarily 1 here.
    let _ = mec_size(graph)?;
    Ok(graph.clone())
}
