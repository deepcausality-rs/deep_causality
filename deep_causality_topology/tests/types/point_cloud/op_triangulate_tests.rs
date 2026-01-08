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
