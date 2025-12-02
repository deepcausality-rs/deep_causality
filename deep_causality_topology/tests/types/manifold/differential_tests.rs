/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, SimplicialComplex, Skeleton, Simplex};

// Setup function to create a manifold from a point cloud
fn setup_triangle_manifold() -> Manifold<f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();

    Manifold::new(complex, data, 0).unwrap()
}

#[test]
fn test_exterior_derivative_d0() {
    let manifold = setup_triangle_manifold();
    // data on vertices: [10.0, 20.0, 30.0]
    let d0_form = manifold.exterior_derivative(0);
    assert_eq!(d0_form.shape(), &[3]); // 3 edges
                                       // d(f) on edge (v0,v1) is f(v1)-f(v0)
                                       // Edges are (0,1), (0,2), (1,2)
                                       // d(f)(e01) = f(v1) - f(v0) = 20-10=10
                                       // d(f)(e02) = f(v2) - f(v0) = 30-10=20
                                       // d(f)(e12) = f(v2) - f(v1) = 30-20=10
    let expected = vec![10.0, 10.0, 20.0]; // Order depends on complex construction
    let mut actual = d0_form.as_slice().to_vec();
    actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(actual, expected);
}

#[test]
fn test_exterior_derivative_d1() {
    let manifold = setup_triangle_manifold();
    let d1_form = manifold.exterior_derivative(1);
    assert_eq!(d1_form.shape(), &[1]); // 1 face
    assert_eq!(d1_form.len(), 1);
}

#[test]
fn test_exterior_derivative_nilpotency() {
    let manifold = setup_triangle_manifold();
    let d0_form = manifold.exterior_derivative(0);
    // Now, apply derivative again. We need to put d0_form back into a manifold
    let mut new_data = vec![0.0; 7];
    new_data[3..6].copy_from_slice(d0_form.as_slice());
    let complex = manifold.complex().clone();
    let manifold2 =
        Manifold::new(complex, CausalTensor::new(new_data, vec![7]).unwrap(), 0).unwrap();
    let d1_of_d0 = manifold2.exterior_derivative(1);
    // d(d(f)) should be zero
    assert_eq!(d1_of_d0.len(), 1);
    assert!((d1_of_d0.as_slice()[0]).abs() < 1e-9);
}

#[test]
fn test_exterior_derivative_out_of_bounds() {
    let manifold = setup_triangle_manifold();
    let d3_form = manifold.exterior_derivative(3);
    assert_eq!(d3_form.len(), 0);
}

#[test]
fn test_hodge_star_k0() {
    let manifold = setup_triangle_manifold();
    let star0 = manifold.hodge_star(0); // 0-form -> 2-form
    assert_eq!(star0.shape(), &[1]); // 1 face
    assert!(star0.as_slice()[0] != 0.0);
}

#[test]
fn test_hodge_star_k1() {
    let manifold = setup_triangle_manifold();
    let star1 = manifold.hodge_star(1); // 1-form -> 1-form
    assert_eq!(star1.shape(), &[3]); // 3 edges
}

#[test]
fn test_hodge_star_k2() {
    let manifold = setup_triangle_manifold();
    let star2 = manifold.hodge_star(2); // 2-form -> 0-form
    assert_eq!(star2.shape(), &[3]); // 3 vertices
}

#[test]
fn test_hodge_star_out_of_bounds() {
    let manifold = setup_triangle_manifold();
    let star3 = manifold.hodge_star(3);
    assert_eq!(star3.len(), 0);
}

#[test]
fn test_laplacian_scalar_field() {
    let manifold = setup_triangle_manifold();
    let laplacian = manifold.laplacian();
    assert_eq!(laplacian.shape(), &[3]); // on 3 vertices
    // The exact values depend on the orientation of the boundary operators,
    // which affects the sign. The magnitude should be correct.
    // L(f0) = (f1-f0) + (f2-f0) = 10 + 20 = 30
    // L(f1) = (f0-f1) + (f2-f1) = -10 + 10 = 0
    // L(f2) = (f0-f2) + (f1-f2) = -20 -10 = -30
    let mut result = laplacian.as_slice().to_vec();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Allow for small floating point errors
    assert!((result[0] - (-30.0)).abs() < 1e-9);
    assert!((result[1] - 0.0).abs() < 1e-9);
    assert!((result[2] - 30.0).abs() < 1e-9);
}

#[test]
fn test_laplacian_no_edges() {
    let skel0 = Skeleton::new(0, vec![Simplex::new(vec![0])]);
    let complex = SimplicialComplex::new(vec![skel0], vec![], vec![], vec![]);
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let manifold = Manifold::new(complex, data, 0).unwrap();
    let laplacian = manifold.laplacian();
    assert_eq!(laplacian.as_slice(), &[0.0]);
}

#[test]
fn test_laplacian_no_boundary_ops() {
    let skel0 = Skeleton::new(0, vec![Simplex::new(vec![0])]);
    let complex = SimplicialComplex::new(vec![skel0], vec![], vec![], vec![]);
    let data = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let manifold = Manifold::new(complex, data, 0).unwrap();
    let laplacian = manifold.laplacian();
    assert_eq!(laplacian.len(), 1);
    assert_eq!(laplacian.as_slice(), &[0.0]);
}
