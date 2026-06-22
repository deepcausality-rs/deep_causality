/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

// A single-point cloud reaches `find_duplicate_points` with `num_points == 1`,
// exercising the `num_points < 2 => None` early return. The duplicate check
// must pass (no pair exists), and triangulate succeeds with a lone vertex.
#[test]
fn test_triangulate_single_point_skips_duplicate_check() {
    let points = CausalTensor::new(vec![0.5, 0.5], vec![1, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let sc = pc.triangulate(1.0).expect("single point must triangulate");
    assert_eq!(sc.skeletons()[0].simplices().len(), 1); // one vertex
    assert_eq!(sc.skeletons()[1].simplices().len(), 0); // no edges
}

// A two-point cloud at non-trivial radius yields one edge, building a boundary
// operator with face lookups that hit the triplet-push path.
#[test]
fn test_triangulate_two_points_builds_edge_boundary() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let sc = pc.triangulate(2.0).expect("two points must triangulate");
    assert_eq!(sc.skeletons()[0].simplices().len(), 2);
    assert_eq!(sc.skeletons()[1].simplices().len(), 1); // one edge
    // Boundary operator d1 (edges -> vertices) must carry the two endpoints.
    assert_eq!(sc.boundary_operators()[0].values().len(), 2);
}
