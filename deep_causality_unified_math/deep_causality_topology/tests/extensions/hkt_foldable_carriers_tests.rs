/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Foldable` for the five carriers that had `Functor` and `CoMonad` but no reduction.
//!
//! `GraphWitness`, `MixedGraphWitness`, `HypergraphWitness`, `PointCloudWitness<C>` and
//! `TopologyWitness<R>` each hold their elements in a `CausalTensor`, exactly as
//! `CellComplexWitness`, `ChainWitness`, `CochainWitness` and `LatticeComplexWitness` — which do
//! implement `Foldable` — already do. `fold` delegates to `CausalTensorWitness::fold` the same way
//! `fmap` delegates to `CausalTensorWitness::fmap`.
//!
//! Every case uses distinct values and a base-10 accumulator, so a reversal shows as `4321`
//! against `1234` rather than hiding behind a commutative sum. Each carrier also has something
//! beside the elements — a vertex count, an incidence matrix, a point tensor, a complex — and a
//! fold that read the wrong field would still return a plausible number, so the values are chosen
//! to make the right answer unique.

use deep_causality_haft::{Foldable, Functor};
use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{
    Graph, GraphWitness, Hypergraph, HypergraphWitness, MixedGraph, MixedGraphWitness, PointCloud,
    PointCloudWitness, Topology, TopologyWitness,
};
use std::sync::Arc;

fn t(xs: &[i64]) -> CausalTensor<i64> {
    CausalTensor::new(xs.to_vec(), vec![xs.len()]).unwrap()
}

/// Base-10 accumulation: order-sensitive, and every element must be visited exactly once.
fn digits<A: Into<i64>>(acc: i64, x: A) -> i64 {
    acc * 10 + x.into()
}

// ---------------------------------------------------------------------------
// Graph
// ---------------------------------------------------------------------------

fn graph(xs: &[i64]) -> Graph<i64> {
    Graph::new(xs.len(), t(xs), 0).unwrap()
}

#[test]
fn test_graph_fold_visits_every_node_in_order() {
    assert_eq!(GraphWitness::fold(graph(&[1, 2, 3, 4]), 0, digits), 1234);
}

#[test]
fn test_graph_fold_on_a_single_node() {
    assert_eq!(GraphWitness::fold(graph(&[7]), 0, digits), 7);
}

#[test]
fn test_graph_fold_seed_is_returned_untouched_when_it_dominates() {
    // A seed of 0 hides an implementation that discards it. 5 does not: 5*10+1 = 51.
    assert_eq!(GraphWitness::fold(graph(&[1]), 5, digits), 51);
}

#[test]
fn test_graph_fold_composes_after_fmap() {
    let doubled = GraphWitness::fmap(graph(&[1, 2, 3]), |x| x * 2);
    assert_eq!(GraphWitness::fold(doubled, 0, digits), 246);
}

// ---------------------------------------------------------------------------
// MixedGraph
// ---------------------------------------------------------------------------

#[test]
fn test_mixed_graph_fold_visits_every_node_in_order() {
    let g = MixedGraph::new(4, t(&[1, 2, 3, 4]), 0).unwrap();
    assert_eq!(MixedGraphWitness::fold(g, 0, digits), 1234);
}

// ---------------------------------------------------------------------------
// Hypergraph
// ---------------------------------------------------------------------------

#[test]
fn test_hypergraph_fold_visits_every_node_in_order() {
    // Three vertices, one hyperedge touching all of them.
    let incidence = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1i8), (1, 0, 1), (2, 0, 1)]).unwrap();
    let h = Hypergraph::new(incidence, t(&[1, 2, 3]), 0).unwrap();
    assert_eq!(HypergraphWitness::fold(h, 0, digits), 123);
}

// ---------------------------------------------------------------------------
// PointCloud — the element is the metadata, not the points
// ---------------------------------------------------------------------------

#[test]
fn test_point_cloud_fold_reads_the_metadata_not_the_points() {
    // The points and the metadata carry *different* values on purpose. A fold that reached for
    // `points` would return 987 and still look like a working fold.
    let points = CausalTensor::new(vec![9i64, 8, 7], vec![3]).unwrap();
    let metadata = t(&[1, 2, 3]);
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    assert_eq!(PointCloudWitness::<i64>::fold(pc, 0, digits), 123);
}

// ---------------------------------------------------------------------------
// Topology
// ---------------------------------------------------------------------------

#[test]
fn test_topology_fold_visits_every_coefficient_in_order() {
    // A triangle's 0-skeleton has three vertices, so grade 0 carries three coefficients.
    let top = Topology::new(Arc::new(create_triangle_complex()), 0, t(&[1, 2, 3]), 0).unwrap();
    assert_eq!(TopologyWitness::<f64>::fold(top, 0, digits), 123);
}

// ---------------------------------------------------------------------------
// Shared: the accumulator type is free
// ---------------------------------------------------------------------------

#[test]
fn test_fold_changes_the_accumulator_type() {
    let joined = GraphWitness::fold(graph(&[1, 2, 3]), String::new(), |mut acc, x| {
        acc.push_str(&x.to_string());
        acc
    });
    assert_eq!(joined, "123");
}
