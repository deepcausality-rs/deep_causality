/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Manifold::neighbors` API in
//! `src/types/manifold/api/neighbors.rs`. Verifies that the manifold delegates
//! correctly to user-chosen `Neighborhood` strategies on lattice and simplicial
//! complexes.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    FaceAdjacent, LatticeComplex, Manifold, Moore, Neighborhood, PointCloud, VonNeumann,
};

fn single_triangle_manifold() -> Manifold<deep_causality_topology::SimplicialComplex<f64>, f64> {
    // Single triangle = a valid simplicial manifold (passes orientation/link checks).
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();
    let complex = point_cloud.triangulate(1.2).unwrap();
    let n = complex.total_simplices();
    let data = CausalTensor::new(vec![0.0; n], vec![n]).unwrap();
    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_manifold_neighbors_face_adjacent_single_triangle() {
    // A single triangle has no top-cell neighbors -> empty result.
    let manifold = single_triangle_manifold();
    let n0: Vec<_> = manifold.neighbors(FaceAdjacent, 0).collect();
    // Delegation must match calling the strategy directly on the complex.
    let direct: Vec<_> = FaceAdjacent.neighbors(manifold.complex(), 0).collect();
    assert_eq!(n0, direct);
}

#[test]
fn test_manifold_neighbors_face_adjacent_out_of_range() {
    let manifold = single_triangle_manifold();
    let neighbors: Vec<_> = manifold.neighbors(FaceAdjacent, 9999).collect();
    assert!(neighbors.is_empty());
}

#[test]
fn test_manifold_neighbors_lattice_von_neumann() {
    // 3x3 lattice manifold; manifold over a LatticeComplex (no metric required).
    let complex: LatticeComplex<2, f64> = LatticeComplex::<2, f64>::square_torus(3);
    let n_top =
        <LatticeComplex<2, f64> as deep_causality_topology::ChainComplex>::num_cells(&complex, 2);
    let data = CausalTensor::new(vec![0.0f64; n_top], vec![n_top]).unwrap();

    let manifold: Manifold<LatticeComplex<2, f64>, f64> = Manifold::from_cubical(complex, data, 0);

    // Von Neumann neighborhood on a torus must have neighbors for cell 0.
    let neighbors: Vec<_> = manifold.neighbors(VonNeumann, 0).collect();
    assert!(
        !neighbors.is_empty(),
        "Von Neumann neighborhood should not be empty on a 3x3 torus"
    );
}

#[test]
fn test_manifold_neighbors_lattice_moore_matches_direct() {
    // Confirms the manifold's neighbors() method is a pure delegation.
    let complex: LatticeComplex<2, f64> = LatticeComplex::<2, f64>::square_torus(3);
    let n_top =
        <LatticeComplex<2, f64> as deep_causality_topology::ChainComplex>::num_cells(&complex, 2);
    let data = CausalTensor::new(vec![0.0f64; n_top], vec![n_top]).unwrap();

    let manifold: Manifold<LatticeComplex<2, f64>, f64> = Manifold::from_cubical(complex, data, 0);

    let via_manifold: Vec<_> = manifold.neighbors(Moore, 0).collect();
    let via_strategy: Vec<_> = Moore.neighbors(manifold.complex(), 0).collect();
    assert_eq!(via_manifold, via_strategy);
}
