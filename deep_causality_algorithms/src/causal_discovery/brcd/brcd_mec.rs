/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Markov-equivalence-class (MEC) sizing and uniform DAG sampling.
//!
//! Each CPDAG represents a class of DAGs that all entail the same conditional
//! independencies (and, for BRCD, the same likelihood). BRCD weights a root
//! configuration by the **size** of its class, `p(G | R) = mec_size / Σ`, and
//! samples one representative DAG to score (the efficiency trick: one DAG stands
//! in for the whole class).
//!
//! ## What this computes
//!
//! A CPDAG decomposes into its **compelled arcs** (fixed directed edges) plus a
//! set of **chain components** — the connected components of the undirected
//! subgraph, each a chordal undirected graph (UCCG). The members of the
//! equivalence class are exactly: the compelled arcs, together with one *acyclic
//! moral orientation* (AMO) chosen independently from each chain component
//! (Andersson–Madigan–Perlman; Wienöbst et al. 2021). Therefore
//!
//! * `mec_size = ∏ |AMO(component)|`, and
//! * a uniform member is drawn by picking a uniform AMO per component
//!   independently.
//!
//! The per-component AMOs are enumerated by the MCS-based recursion ported from
//! the authoritative `ctx/next/brcd/BRCD/mcs_num.py` (`enumerate_amos`): a
//! maximum-cardinality search that branches over the maximum-label vertices and
//! orients each newly fixed edge toward the just-visited vertex, rejecting any
//! orientation that would create an unshielded collider. This is the in-tree
//! alternative to the external `cliquepicking` package used in `brcd.py`; it
//! gives the exact size and a uniform sample without an external dependency.
//!
//! ## Bound
//!
//! Enumeration is exponential in the worst case (a large clique has factorially
//! many AMOs). Both the per-component AMO count and the product across
//! components are capped at [`MEC_ENUM_BOUND`]; exceeding it returns
//! [`BrcdErrorEnum::ClassTooLarge`] rather than silently scoring a truncated class. A
//! future change can swap in a true clique-picking sampler behind this same API
//! if scale demands it.
//!
//! ## Precondition
//!
//! The input is expected to be a valid CPDAG: every edge is either a directed
//! arc or an undirected edge (no bidirected/circle endpoints), the arc
//! projection is acyclic, and each chain component is chordal. Bidirected or
//! partially-oriented edges yield [`BrcdErrorEnum::NotACpdag`]; a cyclic arc
//! projection yields [`BrcdErrorEnum::NotAcyclic`].

use crate::causal_discovery::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use deep_causality_rand::Rng;
use deep_causality_topology::{EdgeKind, MixedGraph};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Upper bound on the per-component AMO count and on the equivalence-class size
/// (the product across components). Beyond this the engine returns
/// [`BrcdErrorEnum::ClassTooLarge`] instead of enumerating an intractable class.
pub const MEC_ENUM_BOUND: usize = 100_000;

/// Returns the size of the Markov equivalence class of `graph`.
///
/// For a valid CPDAG this is the product of each chain component's acyclic-moral-
/// orientation count; an arcs-only (fully directed) acyclic graph has size `1`.
///
/// # Errors
/// * [`BrcdErrorEnum::NotACpdag`] if any edge is bidirected or partially oriented.
/// * [`BrcdErrorEnum::NotAcyclic`] if the arc projection contains a cycle.
/// * [`BrcdErrorEnum::ClassTooLarge`] if the class exceeds [`MEC_ENUM_BOUND`].
pub fn mec_size<N>(graph: &MixedGraph<N>) -> Result<usize, BrcdError> {
    validate_cpdag(graph)?;
    let mut size: usize = 1;
    for component in chain_components(graph) {
        let amos = enumerate_amos(&component)?;
        size = checked_class_product(size, amos.len())?;
    }
    Ok(size)
}

/// Returns a representative DAG from the Markov equivalence class of `graph`:
/// the compelled arcs plus the **first** enumerated AMO of each chain component.
/// Deterministic.
///
/// # Errors
/// As [`mec_size`].
pub fn representative_dag<N: Clone>(graph: &MixedGraph<N>) -> Result<MixedGraph<N>, BrcdError> {
    build_member(graph, |amo_count| {
        debug_assert!(amo_count > 0);
        0
    })
}

/// Draws a DAG uniformly at random from the Markov equivalence class of `graph`,
/// using `rng` for the per-component AMO choice. For a valid CPDAG the chain
/// components orient independently and every combination is a class member, so
/// an independent uniform AMO per component yields a uniform class member.
///
/// # Errors
/// As [`mec_size`].
pub fn mec_sample_dag<N: Clone, R: Rng>(
    graph: &MixedGraph<N>,
    rng: &mut R,
) -> Result<MixedGraph<N>, BrcdError> {
    build_member(graph, |amo_count| rng.random_range(0..amo_count))
}

// --- internals --------------------------------------------------------------

/// Validates that every edge is a directed arc or an undirected edge and that
/// the arc projection is acyclic.
fn validate_cpdag<N>(graph: &MixedGraph<N>) -> Result<(), BrcdError> {
    for edge in graph.edges().values() {
        match edge.kind() {
            EdgeKind::Directed | EdgeKind::Undirected => {}
            _ => return Err(BrcdError(BrcdErrorEnum::NotACpdag)),
        }
    }
    if graph.has_cycle() {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }
    Ok(())
}

/// Multiplies the running class size by a component's AMO count, capping at
/// [`MEC_ENUM_BOUND`].
fn checked_class_product(acc: usize, factor: usize) -> Result<usize, BrcdError> {
    match acc.checked_mul(factor) {
        Some(p) if p <= MEC_ENUM_BOUND => Ok(p),
        _ => Err(BrcdError(BrcdErrorEnum::ClassTooLarge {
            bound: MEC_ENUM_BOUND,
        })),
    }
}

/// A chain component: its vertices and the undirected adjacency restricted to
/// them, as a sorted-key/sorted-value map for deterministic enumeration.
struct Component {
    /// Undirected adjacency within the component (every key is a component vertex).
    adj: BTreeMap<usize, Vec<usize>>,
}

/// Returns the chain components of `graph` — the connected components of the
/// undirected subgraph. Vertices touched by no undirected edge form no
/// component (they are fixed by the compelled arcs alone).
fn chain_components<N>(graph: &MixedGraph<N>) -> Vec<Component> {
    // Vertices incident to at least one undirected edge.
    let mut undirected_vertices: BTreeSet<usize> = BTreeSet::new();
    for &(a, b) in graph.undirected_edges().iter() {
        undirected_vertices.insert(a);
        undirected_vertices.insert(b);
    }

    let mut seen: BTreeSet<usize> = BTreeSet::new();
    let mut components = Vec::new();

    for &start in undirected_vertices.iter() {
        if seen.contains(&start) {
            continue;
        }
        // BFS over undirected edges to collect this component's vertices.
        let mut members: BTreeSet<usize> = BTreeSet::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        seen.insert(start);
        members.insert(start);
        while let Some(v) = queue.pop_front() {
            for nb in graph.undirected_neighbors(v) {
                if seen.insert(nb) {
                    members.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        // Build the restricted adjacency (undirected_neighbors are all in-component).
        let mut adj: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &v in members.iter() {
            adj.insert(v, graph.undirected_neighbors(v));
        }
        components.push(Component { adj });
    }
    components
}

/// Enumerates the acyclic moral orientations of one chain component, each as a
/// list of directed edges `(parent, child)` covering every undirected edge of
/// the component exactly once. Ported from `mcs_num.py::enumerate_amos`.
///
/// # Errors
/// [`BrcdErrorEnum::ClassTooLarge`] if the AMO count exceeds [`MEC_ENUM_BOUND`].
fn enumerate_amos(component: &Component) -> Result<Vec<Vec<(usize, usize)>>, BrcdError> {
    let adj = &component.adj;
    let total_edges: usize = adj.values().map(Vec::len).sum::<usize>() / 2;

    let mut labels: BTreeMap<usize, i64> = adj.keys().map(|&v| (v, 0i64)).collect();
    let mut visited: BTreeSet<usize> = BTreeSet::new();
    let mut unique: BTreeSet<Vec<(usize, usize)>> = BTreeSet::new();
    let mut out: Vec<Vec<(usize, usize)>> = Vec::new();

    mcs_enum(
        adj,
        total_edges,
        &mut visited,
        &mut labels,
        &[],
        &mut unique,
        &mut out,
    )?;
    Ok(out)
}

/// Recursive MCS enumeration: branch over the maximum-label unvisited vertices,
/// orient each fixed edge toward the just-visited vertex unless it would create
/// an unshielded collider, and record every fully-oriented acyclic moral
/// orientation once.
#[allow(clippy::too_many_arguments)]
fn mcs_enum(
    adj: &BTreeMap<usize, Vec<usize>>,
    total_edges: usize,
    visited: &mut BTreeSet<usize>,
    labels: &mut BTreeMap<usize, i64>,
    oriented: &[(usize, usize)],
    unique: &mut BTreeSet<Vec<(usize, usize)>>,
    out: &mut Vec<Vec<(usize, usize)>>,
) -> Result<(), BrcdError> {
    // Base case: every vertex visited.
    if visited.len() == adj.len() {
        if oriented.len() == total_edges {
            let mut key = oriented.to_vec();
            key.sort_unstable();
            if unique.insert(key.clone()) {
                if out.len() >= MEC_ENUM_BOUND {
                    return Err(BrcdError(BrcdErrorEnum::ClassTooLarge {
                        bound: MEC_ENUM_BOUND,
                    }));
                }
                out.push(key);
            }
        }
        return Ok(());
    }

    // `compute_reachable` in the reference returns all unvisited vertices, so the
    // candidate set is the maximum-label vertices among the unvisited.
    let max_label = adj
        .keys()
        .filter(|v| !visited.contains(v))
        .map(|v| labels[v])
        .max()
        .expect("non-empty unvisited set");
    let candidates: Vec<usize> = adj
        .keys()
        .copied()
        .filter(|v| !visited.contains(v) && labels[v] == max_label)
        .collect();

    for v in candidates {
        visited.insert(v);
        let saved_labels = labels.clone();
        let mut next_oriented = oriented.to_vec();

        for &neighbor in &adj[&v] {
            if !visited.contains(&neighbor) {
                *labels.get_mut(&neighbor).expect("labelled vertex") += 1;
            } else if !next_oriented.contains(&(neighbor, v))
                && !next_oriented.contains(&(v, neighbor))
                && !creates_invalid_collider(v, neighbor, &next_oriented, adj)
            {
                next_oriented.push((neighbor, v));
            }
        }

        mcs_enum(
            adj,
            total_edges,
            visited,
            labels,
            &next_oriented,
            unique,
            out,
        )?;

        visited.remove(&v);
        *labels = saved_labels;
    }
    Ok(())
}

/// Returns `true` if orienting `neighbor → v` would create an unshielded
/// collider `other → v ← neighbor` with `other` non-adjacent to `neighbor`.
/// Ported from `mcs_num.py::creates_invalid_collider`.
fn creates_invalid_collider(
    v: usize,
    neighbor: usize,
    oriented: &[(usize, usize)],
    adj: &BTreeMap<usize, Vec<usize>>,
) -> bool {
    for &other in &adj[&v] {
        if other != neighbor
            && !adj[&neighbor].contains(&other)
            && oriented.contains(&(other, v))
            && !oriented.contains(&(v, other))
        {
            return true;
        }
    }
    false
}

/// Builds one class member: clones `graph` (keeping its compelled arcs, node
/// data, and cursor) and orients each chain component's undirected edges
/// according to the AMO selected by `choose` (given the component's AMO count).
fn build_member<N: Clone>(
    graph: &MixedGraph<N>,
    mut choose: impl FnMut(usize) -> usize,
) -> Result<MixedGraph<N>, BrcdError> {
    validate_cpdag(graph)?;
    let mut dag = graph.clone();
    let mut running: usize = 1;

    for component in chain_components(graph) {
        let amos = enumerate_amos(&component)?;
        if amos.is_empty() {
            // A chordal chain component always has at least one AMO; an empty
            // result means the component is not chordal, so the input is not a
            // valid CPDAG.
            return Err(BrcdError(BrcdErrorEnum::NotACpdag));
        }
        running = checked_class_product(running, amos.len())?;
        let pick = choose(amos.len());
        for &(parent, child) in &amos[pick] {
            // The cloned edge is still undirected; orient it parent → child.
            dag.orient(parent, child)
                .expect("AMO edge corresponds to an undirected edge of the clone");
        }
    }

    // A valid CPDAG always yields an acyclic member; guard against a malformed
    // input that slipped past validation rather than returning a cyclic "DAG".
    if dag.has_cycle() {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }
    Ok(dag)
}
