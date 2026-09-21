/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Setup function to create a manifold from a point cloud.
//
// Attaches a `ReggeGeometry` metric with all-unit edge lengths. Required after
// R4.5 widening: `hodge_star`, `codifferential`, and `laplacian` now route
// through the `HasHodgeStar<R>` trait and panic if `self.metric.is_none()`.
// The simplicial impl of `HasHodgeStar<R>` ignores the metric instance and
// reads from `SimplicialComplex::hodge_star_operators`, so any
// correctly-sized `ReggeGeometry` is valid — we use unit lengths.
fn setup_triangle_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap();
    let point_cloud = PointCloud::new(points, metadata, 0).unwrap();

    let complex = point_cloud.triangulate(1.2).unwrap();
    // Complex has 3 vertices, 3 edges, 1 face. Total 7 simplices.
    let data = CausalTensor::new(vec![10.0, 20.0, 30.0, 1.0, 2.0, 3.0, 100.0], vec![7]).unwrap();
    let metric = ReggeGeometry::new(CausalTensor::new(vec![1.0, 1.0, 1.0], vec![3]).unwrap());

    Manifold::with_metric(complex, data, Some(metric), 0).unwrap()
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
    actual.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
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
    assert!((d1_of_d0.as_slice()[0] as f64).abs() < 1e-9);
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
    assert_eq!(star0.shape(), &[3]); // 1 face
    assert!(star0.as_slice()[0] != 0.0);
}

#[test]
fn test_hodge_star_k1() {
    let manifold = setup_triangle_manifold();
    let star1 = manifold.hodge_star(1); // 1-form -> 1-form
    assert_eq!(star1.shape(), &[3]); // 1-form
}

#[test]
fn test_hodge_star_k2() {
    let manifold = setup_triangle_manifold();
    let star2 = manifold.hodge_star(2); // 2-form -> 0-form
    assert_eq!(star2.shape(), &[1]);
}

#[test]
fn test_hodge_star_out_of_bounds() {
    let manifold = setup_triangle_manifold();
    let star3 = manifold.hodge_star(3);
    assert_eq!(star3.len(), 0);
}

#[test]
fn test_laplacian_scalar_field_geometric() {
    let manifold = setup_triangle_manifold();
    let laplacian = manifold.laplacian(0);

    assert_eq!(laplacian.shape(), &[3]);

    // `M_1` is the lumped Whitney edge mass, the same quantity the endpoint grades carry. It is
    // not the edge length: the mass scales as `h^(n-2k)`, so in two dimensions it is
    // scale-invariant, which a length is not.
    //
    // 1.  Geometry: v0 = (0,0), v1 = (1,0), v2 = (0.5,1). Area = 0.5.
    //     Barycentric gradients: grad(l0) = (-1,-0.5), grad(l1) = (1,-0.5), grad(l2) = (0,1).
    // 2.  Masses:
    //     M_0 = |T| / 3 = 1/6                      (lumped vertex mass, unchanged)
    //     M_1[e_ij] = (2|T| / ((n+1)(n+2)))
    //                 * (|grad l_i|^2 + |grad l_j|^2 - grad l_i . grad l_j),   n = 2
    //       e01 = 13/48 = 0.2708333...
    //       e02 = 11/48 = 0.2291666...
    //       e12 = 11/48 = 0.2291666...
    // 3.  Flux at v0 = M_1[e01](10-20) + M_1[e02](10-30) = -2.708333 - 4.583333 = -7.291666...
    // 4.  Laplacian at v0 = flux / M_0 = -7.291666... * 6 = -43.75.
    //
    // The three values are exact rationals: -175/4, 5/2, 165/4. Cross-checked against an
    // independent numpy evaluation of the same Whitney formula.
    let mut result = laplacian.as_slice().to_vec();
    result.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());

    let expected = [-43.75, 2.5, 41.25];

    for (a, b) in result.iter().zip(expected.iter()) {
        assert!((a - b).abs() < 1e-9, "Mismatch: Got {}, Expected {}", a, b);
    }
}
