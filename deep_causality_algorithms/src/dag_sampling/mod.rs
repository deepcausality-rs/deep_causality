/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Clique-Picking counting of acyclic moral orientations (AMOs).
//!
//! A polynomial-time, dependency-free port of the Wienöbst–Bannach–Liśkiewicz
//! "Clique-Picking" counter (AAAI 2021; the algorithm behind the `cliquepicking`
//! package) for the number of acyclic moral orientations of a chordal undirected
//! graph — equivalently, the size of the Markov equivalence class a CPDAG's
//! undirected (chain-component) part represents.
//!
//! ## What this computes
//!
//! For a chordal undirected graph, every acyclic moral orientation corresponds to
//! one member of the represented Markov equivalence class. A CPDAG decomposes into
//! compelled arcs plus chain components (the connected components of its
//! undirected subgraph, each chordal); the class size is the product of the AMO
//! counts of those components. This module counts that quantity.
//!
//! ## Relationship to `brcd::brcd_mec`
//!
//! [`brcd::brcd_mec::mec_size`](crate::brcd::brcd_mec::mec_size) computes the same
//! number by **exact enumeration**, capped at `MEC_ENUM_BOUND = 100_000`. This
//! module is a **polynomial-time** alternative that never enumerates, so it scales
//! to classes far beyond that bound; it is validated to match the enumerator
//! exactly on every class small enough for the enumerator to handle.
//!
//! ## Counting only (for now)
//!
//! This is the **counting** path of Clique-Picking. The module is structured so a
//! uniform AMO sampler (the other half of the reference) can be added later
//! against the same internal clique-tree machinery without reshaping this code.
//!
//! ## Generic count type
//!
//! The reference counts in `num_bigint::BigUint`. To stay dependency-free, the
//! count type here is a generic `T: RealField + FromPrimitive` (from
//! `deep_causality_num`), matching the BRCD numeric bound. It instantiates at
//! `f64` and at the higher-precision `deep_causality_num::Float106`.
//!
//! ## Precondition
//!
//! The undirected part of the input is **assumed** to be a valid CPDAG's chain
//! structure: each connected component chordal. This is not checked (mirroring
//! `brcd_mec`'s documented precondition); an invalid input may give a wrong count.

mod chordal;
mod clique_tree;
mod combinatorics;
mod count;
mod graph;
mod index_set;
mod lazy_tokens;
mod memoization;
mod utils;

pub use count::{count_amos, count_chordal};
pub use graph::Graph;

use crate::dag_sampling::graph::Graph as InternalGraph;
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_topology::MixedGraph;

/// Returns the size of the Markov equivalence class of `graph`'s undirected
/// (chain-component) structure: the number of acyclic moral orientations of its
/// undirected subgraph, as a value of `T`.
///
/// This is the polynomial-time Clique-Picking drop-in alternative to
/// [`brcd::brcd_mec::mec_size`](crate::brcd::brcd_mec::mec_size) for the
/// undirected part. It counts AMOs without enumerating them, so unlike the
/// enumerator it has no class-size bound.
///
/// The undirected edges are read via the same accessors the BRCD code uses
/// (`undirected_edges`, `undirected_neighbors`, `num_vertices`); directed arcs are
/// ignored (they are compelled and orient deterministically, contributing a
/// factor of one). Vertices incident to no undirected edge contribute a factor of
/// one as well.
///
/// # Precondition
///
/// `graph`'s undirected subgraph is assumed to be a valid CPDAG chain structure —
/// every connected component chordal. This is **not** checked; an input that
/// violates the assumption may yield a wrong count (the same contract as
/// `brcd_mec`).
pub fn mec_size<T: RealField + FromPrimitive, N>(graph: &MixedGraph<N>) -> T {
    let n = graph.num_vertices();
    // Build the internal undirected graph over all vertices. Isolated vertices
    // become singleton components (AMO count 1), so they are neutral in the
    // product — matching brcd_mec, which simply skips them.
    let edges: Vec<(usize, usize)> = graph.undirected_edges();
    let g = InternalGraph::from_edge_list(edges, n);
    count_chordal::<T>(&g)
}
