/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

#[test]
fn test_point_cloud_triangulate_success() {
    // Isosceles Right Triangle points (0,0), (1,0), (0,1)
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Radius 1.1 covers edges of length 1.0.
    // Distance (1,0)-(0,1) is sqrt(2) approx 1.414.
    // So with 1.1, we get edges (0,1) and (0,2), but not (1,2).
    let complex = pc.triangulate(1.1);
    assert!(complex.is_ok());
    let sc = complex.unwrap();

    // Expect 3 vertices (0-simplices)
    assert_eq!(sc.skeletons()[0].simplices().len(), 3);
    // Expect 2 edges (1-simplices): (0,1) and (0,2)
    assert_eq!(sc.skeletons()[1].simplices().len(), 2);
    // No 2-simplices (face) expected because not all edges form a clique
    assert_eq!(sc.skeletons().len(), 2);
}

#[test]
fn test_point_cloud_triangulate_complete_triangle() {
    // Equilateral triangle side 1.0
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Radius 1.1 should connect everything
    let complex = pc.triangulate(1.1).unwrap();

    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    assert_eq!(complex.skeletons()[1].simplices().len(), 3); // 3 edges
    assert_eq!(complex.skeletons()[2].simplices().len(), 1); // 1 face
    assert_eq!(complex.max_simplex_dimension(), 2);
}

#[test]
fn test_point_cloud_triangulate_tetrahedron() {
    // Regular tetrahedron-ish (0,0,0), (1,0,0), (0,1,0), (0,0,1)
    // Edges are length 1 or sqrt(2). sqrt(2) ~ 1.414.
    // If we pick radius 1.5, all points should connect.
    let points = CausalTensor::new(
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        vec![4, 3],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();

    // 0-simplices: 4 vertices
    assert_eq!(complex.skeletons()[0].simplices().len(), 4);

    // 1-simplices: 4 choose 2 = 6 edges
    assert_eq!(complex.skeletons()[1].simplices().len(), 6);

    // 2-simplices: 4 choose 3 = 4 faces
    assert_eq!(complex.skeletons()[2].simplices().len(), 4);

    // 3-simplices: 4 choose 4 = 1 tetrahedron
    assert_eq!(complex.skeletons()[3].simplices().len(), 1);

    assert_eq!(complex.max_simplex_dimension(), 3);
}

#[test]
fn test_point_cloud_triangulate_empty_error() {
    // Can't create empty PointCloud via new(), but if we could or if logic changes...
    // Actually PointCloud::new checks for empty points.
    // But let's verify if we bypass or simulate it?
    // The closest we can get is a PointCloud that is valid but maybe triangulation fails?
    // No, empty check is inside triangulate, but construction prevents empty.
    // However, if we construct one manually (unsafe?) or if requirements relax...
    // For now, let's trust the PointCloud::new validation covers the empty case.
    // But we CAN verify behavior with radius 0.0 -> no edges.

    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 1.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    // Radius 0.0 -> no edges
    let complex = pc.triangulate(0.0).unwrap();
    assert_eq!(complex.skeletons()[0].simplices().len(), 2); // 2 vertices
    assert_eq!(complex.skeletons()[1].simplices().len(), 0); // 0 edges

    // Implementation detail: triangulate always initializes 0 and 1 skeletons.
    // So max_simplex_dimension reports 1 (the capacity/structure), even if 1-skeleton is empty.
    assert_eq!(complex.max_simplex_dimension(), 1);
}

#[test]
fn test_point_cloud_triangulate_caps_top_grade_at_ambient_dim_for_coplanar_2d_corners() {
    // Four coplanar corners of a unit square in 2D ambient. All six pairwise
    // distances are ≤ √2, so with radius 1.5 every edge clique is included.
    // Before the ambient-dimension cap, clique expansion would push the
    // top grade to 3 (a flat "tetrahedron" spanning all 4 vertices), whose
    // signed volume is zero and which collapses lumped-mass M_0 to zero —
    // making `Manifold::codifferential` return all zeros on this complex.
    //
    // With the cap in place, `triangulate` stops at grade 2 (matching the
    // ambient dimension), producing the geometrically meaningful
    // {4 vertices, 6 edges, 4 triangles} decomposition.
    let points =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();

    assert_eq!(complex.max_simplex_dimension(), 2);
    assert_eq!(complex.skeletons().len(), 3);
    assert_eq!(complex.skeletons()[0].simplices().len(), 4);
    // Six edges: four sides + two diagonals (both diagonals fit under radius 1.5).
    assert_eq!(complex.skeletons()[1].simplices().len(), 6);
    // Four triangles: the six edges form a complete graph K4, every 3-clique
    // is a 2-simplex.
    assert_eq!(complex.skeletons()[2].simplices().len(), 4);

    // Hodge ⋆ at grade 0 must now have non-zero diagonal entries (lumped mass
    // = sum of incident 2-simplex areas / 3). Each vertex touches at least one
    // triangle, so the M_0 values vector must be non-empty and at least one
    // value must be strictly positive.
    let star_0 = &complex.hodge_star_operators()[0];
    assert!(
        !star_0.values().is_empty(),
        "M_0 must have entries after the ambient-dim cap"
    );
    let max_diag = star_0.values().iter().copied().fold(0.0_f64, f64::max);
    assert!(
        max_diag > 0.0,
        "M_0 must have a positive diagonal after the ambient-dim cap; got max ⋆_0 = {}",
        max_diag
    );
}

#[test]
fn test_point_cloud_triangulate_caps_top_grade_at_ambient_dim_for_5_coplanar_2d_points() {
    // Five coplanar points in 2D ambient with sufficient radius to form
    // many higher-clique cliques. Without the cap, clique expansion would
    // push max_dim to 4 (a flat 4-simplex). With the cap, max_dim is 2.
    let points = CausalTensor::new(
        vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.5, 0.5],
        vec![5, 2],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0; 5], vec![5]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();

    assert_eq!(complex.max_simplex_dimension(), 2);
    assert_eq!(complex.skeletons().len(), 3);
}

#[test]
fn test_point_cloud_triangulate_cap_does_not_fire_for_matching_dim_tetrahedron() {
    // 4 points in 3D ambient — the legitimate tetrahedron case. The cap
    // (ambient_dim = 3) does not fire and the 3-simplex is built as before.
    // This is the regression guard for the existing tetrahedron test path.
    let points = CausalTensor::new(
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        vec![4, 3],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0; 4], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();
    assert_eq!(complex.max_simplex_dimension(), 3);
}
