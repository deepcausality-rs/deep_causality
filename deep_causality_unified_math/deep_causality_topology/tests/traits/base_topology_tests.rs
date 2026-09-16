/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `BaseTopology` Euler characteristic, across every structure that implements the trait.
//!
//! # What these pin
//!
//! `χ = Σₖ (−1)ᵏ Nₖ` is defined for any finite cell complex, so the trait supplies it from the two
//! queries every implementor already answers: the grades it carries, and how many cells sit at
//! each. The tests below check the six implementors report the value their own cell counts imply,
//! and that a complex failing the manifold conditions still reports one.
//!
//! # Where the expected values come from
//!
//! Hatcher, *Algebraic Topology*. A triangulated 2-sphere reads 2, a circle reads 0, a tree reads
//! 1, and `n` disjoint points read `n`.

use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, Graph, Hypergraph, Manifold, MixedGraph, PointCloud, Simplex, SimplicialComplex,
    Skeleton,
};

/// A rank-1 tensor of `n` zeros, the inert payload these structures carry.
fn zeros(n: usize) -> CausalTensor<f64> {
    CausalTensor::new(vec![0.0; n], vec![n]).expect("one entry per cell")
}

/// The boundary operator of a path or cycle on `edges`, each edge leaving `from` and entering `to`.
fn boundary(vertices: usize, edges: &[(usize, usize)]) -> CsrMatrix<i8> {
    let mut triplets = Vec::with_capacity(2 * edges.len());
    for (column, &(from, to)) in edges.iter().enumerate() {
        triplets.push((from, column, -1i8));
        triplets.push((to, column, 1i8));
    }
    CsrMatrix::from_triplets(vertices, edges.len(), &triplets).expect("two entries per edge")
}

// ============================================================================
// PointCloud: a 0-complex, so χ is the point count
// ============================================================================

#[test]
fn test_point_cloud_euler_characteristic_is_its_point_count() {
    let coordinates = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2])
        .expect("three points in the plane");
    let cloud = PointCloud::new(coordinates, zeros(3), 0).expect("a point cloud");

    // Three points with nothing joining them: three components, so χ = 3.
    assert_eq!(cloud.dimension(), 0);
    assert_eq!(cloud.euler_characteristic(), 3);
}

// ============================================================================
// Graph: a 1-complex, so χ = V − E
// ============================================================================

#[test]
fn test_graph_euler_characteristic_is_vertices_minus_edges() {
    // A path on four vertices is a tree, and every tree reads 1.
    let mut tree = Graph::new(4, zeros(4), 0).expect("four vertices");
    tree.add_edge(0, 1).expect("an edge in range");
    tree.add_edge(1, 2).expect("an edge in range");
    tree.add_edge(2, 3).expect("an edge in range");

    assert_eq!(tree.euler_characteristic(), 4 - 3);
}

#[test]
fn test_graph_euler_characteristic_reaches_zero_on_a_cycle() {
    // Closing the path into a cycle adds one edge and drops χ to 0, which is the circle's value.
    let mut cycle = Graph::new(4, zeros(4), 0).expect("four vertices");
    for (u, v) in [(0, 1), (1, 2), (2, 3), (3, 0)] {
        cycle.add_edge(u, v).expect("an edge in range");
    }

    assert_eq!(cycle.euler_characteristic(), 0);
}

#[test]
fn test_graph_euler_characteristic_goes_negative_when_edges_dominate() {
    // Two independent cycles sharing an edge: four vertices, five edges.
    let mut wedge = Graph::new(4, zeros(4), 0).expect("four vertices");
    for (u, v) in [(0, 1), (1, 2), (0, 2), (0, 3), (1, 3)] {
        wedge.add_edge(u, v).expect("an edge in range");
    }

    assert_eq!(wedge.euler_characteristic(), -1);
}

// ============================================================================
// Hypergraph and MixedGraph: 1-complexes over their own notion of an edge
// ============================================================================

#[test]
fn test_hypergraph_euler_characteristic_counts_nodes_against_hyperedges() {
    // Three nodes and two hyperedges, each joining a pair.
    let incidence = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1i8), (1, 0, 1), (1, 1, 1), (2, 1, 1)])
        .expect("two entries per hyperedge");
    let hypergraph = Hypergraph::new(incidence, zeros(3), 0).expect("a hypergraph");

    assert_eq!(hypergraph.euler_characteristic(), 3 - 2);
}

#[test]
fn test_mixed_graph_euler_characteristic_counts_vertices_against_edges() {
    use deep_causality_topology::Mark;

    let mut graph = MixedGraph::new(3, zeros(3), 0).expect("three vertices");
    graph
        .add_edge(0, 1, Mark::Arrow, Mark::Tail)
        .expect("an edge in range");
    graph
        .add_edge(1, 2, Mark::Circle, Mark::Circle)
        .expect("an edge in range");

    // The edge marks record orientation and carry no weight in a cell count.
    assert_eq!(graph.euler_characteristic(), 3 - 2);
}

// ============================================================================
// SimplicialComplex: the full alternating sum
// ============================================================================

#[test]
fn test_simplicial_complex_euler_characteristic_of_a_triangulated_sphere_is_two() {
    // The boundary of a tetrahedron: four vertices, six edges, four triangles.
    let vertices = (0..4).map(|i| Simplex::new(vec![i])).collect();
    let edge_pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
    let edges = edge_pairs
        .iter()
        .map(|&(a, b)| Simplex::new(vec![a, b]))
        .collect();
    let faces = [[0, 1, 2], [0, 1, 3], [0, 2, 3], [1, 2, 3]]
        .iter()
        .map(|f| Simplex::new(f.to_vec()))
        .collect();

    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![
            Skeleton::new(0, vertices),
            Skeleton::new(1, edges),
            Skeleton::new(2, faces),
        ],
        vec![boundary(4, &edge_pairs)],
        vec![],
        vec![],
    );

    assert_eq!(complex.dimension(), 2);
    assert_eq!(complex.euler_characteristic(), 4 - 6 + 4);
}

#[test]
fn test_simplicial_complex_euler_characteristic_survives_a_non_manifold_complex() {
    // Two triangles meeting at the single vertex 0. The link of that vertex is two disjoint arcs,
    // so the complex is no manifold, and it still has an Euler characteristic: five vertices, six
    // edges, two triangles.
    let vertices = (0..5).map(|i| Simplex::new(vec![i])).collect();
    let edge_pairs = [(0, 1), (0, 2), (1, 2), (0, 3), (0, 4), (3, 4)];
    let edges = edge_pairs
        .iter()
        .map(|&(a, b)| Simplex::new(vec![a, b]))
        .collect();
    let faces = [[0, 1, 2], [0, 3, 4]]
        .iter()
        .map(|f| Simplex::new(f.to_vec()))
        .collect();

    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![
            Skeleton::new(0, vertices),
            Skeleton::new(1, edges),
            Skeleton::new(2, faces),
        ],
        vec![boundary(5, &edge_pairs)],
        vec![],
        vec![],
    );

    // Two discs joined at a point is contractible to a wedge of two discs, so χ = 1.
    assert_eq!(complex.euler_characteristic(), 5 - 6 + 2);

    // The manifold constructor rejects the same complex, which is what this test is here to record:
    // the certificate and the invariant are separate questions.
    let cells = 5 + 6 + 2;
    assert!(Manifold::new(complex, zeros(cells), 0).is_err());
}

// ============================================================================
// Manifold: unchanged by the move, and now inherited rather than overridden
// ============================================================================

#[test]
fn test_manifold_euler_characteristic_reads_through_base_topology() {
    // A single edge with its two endpoints: an interval, which reads 1.
    let vertices = vec![Simplex::new(vec![0]), Simplex::new(vec![1])];
    let edges = vec![Simplex::new(vec![0, 1])];
    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![Skeleton::new(0, vertices), Skeleton::new(1, edges)],
        vec![boundary(2, &[(0, 1)])],
        vec![],
        vec![],
    );

    let manifold = Manifold::new(complex, zeros(3), 0).expect("an interval is a manifold");

    assert_eq!(manifold.euler_characteristic(), 2 - 1);
}

#[test]
fn test_an_empty_grade_contributes_nothing() {
    // A structure reporting `None` at a grade contributes zero there. A point cloud carries
    // nothing above grade 0, so asking past its dimension changes no sum.
    let coordinates =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).expect("two points in the plane");
    let cloud = PointCloud::new(coordinates, zeros(2), 0).expect("a point cloud");

    assert_eq!(cloud.num_elements_at_grade(1), None);
    assert_eq!(cloud.euler_characteristic(), 2);
}
