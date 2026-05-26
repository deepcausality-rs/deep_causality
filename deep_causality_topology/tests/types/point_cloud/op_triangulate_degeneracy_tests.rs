/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Regression tests for the degeneracy-detection narrowing of
//! `PointCloud::triangulate`. Each test exercises one classified degeneracy
//! category from `harden-simplicial-hodge-degeneracy-detection` Decision 1.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::PointCloud;

// -----------------------------------------------------------------------------
// Category (a): duplicate-input-point rejection
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_rejects_duplicate_input_points() {
    // Four points in 2D where indices 1 and 2 are identical.
    let points = CausalTensor::new(
        vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        vec![4, 2],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate(1.5).unwrap_err();
    let msg = format!("{}", err);

    assert!(
        msg.contains("duplicate point"),
        "error must name the duplicate-point category; got: {}",
        msg
    );
    assert!(
        msg.contains("index 1") && msg.contains("index 2"),
        "error must reference both offending indices 1 and 2; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_rejects_two_identical_points() {
    // Two identical points at index 0 and 1.
    let points = CausalTensor::new(vec![0.5, 0.5, 0.5, 0.5], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate(1.0).unwrap_err();
    let msg = format!("{}", err);

    assert!(msg.contains("duplicate point"));
    assert!(msg.contains("index 0") && msg.contains("index 1"));
}

// -----------------------------------------------------------------------------
// Category (b): zero-volume top-dimensional simplex rejection
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_rejects_three_collinear_points_in_2d() {
    // Three points on the x-axis. With radius 3.0 every pair connects, so
    // clique expansion builds the 2-simplex {0,1,2}. The triangle has zero
    // area; the top-mass branch must surface this as an error rather than
    // silently substituting T::zero().
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 2.0, 0.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate(3.0).unwrap_err();
    let msg = format!("{}", err);

    assert!(
        msg.contains("top-dimensional simplex"),
        "error must name the top-dimensional-simplex category; got: {}",
        msg
    );
    assert!(
        msg.contains("below tolerance"),
        "error must name the below-tolerance condition; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_rejects_four_coplanar_points_in_3d() {
    // Four points all at z = 0 in 3D ambient. Connectivity radius covers all
    // pairs, so clique expansion (capped at ambient_dim = 3) builds the
    // 3-simplex {0,1,2,3}, whose Cayley-Menger determinant is zero. The
    // top-mass branch catches it via the unified volume-below-tolerance path.
    let points = CausalTensor::new(
        vec![
            0.0, 0.0, 0.0, // p0
            1.0, 0.0, 0.0, // p1
            0.0, 1.0, 0.0, // p2
            1.0, 1.0, 0.0, // p3
        ],
        vec![4, 3],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate(2.0).unwrap_err();
    let msg = format!("{}", err);

    assert!(
        msg.contains("top-dimensional simplex"),
        "error must name the top-dimensional-simplex category; got: {}",
        msg
    );
}

// -----------------------------------------------------------------------------
// Regression-prevention: non-degenerate input still succeeds
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_unit_triangle_still_succeeds() {
    // Generic-position three-point triangle. The new rejection logic must not
    // over-reject canonical non-degenerate input.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();

    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    assert_eq!(complex.skeletons()[1].simplices().len(), 3);
    assert_eq!(complex.skeletons()[2].simplices().len(), 1);
    assert_eq!(complex.max_simplex_dimension(), 2);
}

// -----------------------------------------------------------------------------
// Threshold boundary behaviour
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_accepts_volume_above_threshold() {
    // Triangle (0,0), (1,0), (0,1e-6). Area = 5e-7, well above f64::EPSILON *
    // 100 = 2.22e-14. Gram pivots are 1.0 and 1e-12, both above the same
    // pivot threshold. The complex must be accepted.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1e-6], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate(1.5).unwrap();

    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    assert_eq!(complex.skeletons()[1].simplices().len(), 3);
    assert_eq!(complex.skeletons()[2].simplices().len(), 1);
}

#[test]
fn test_triangulate_rejects_volume_below_threshold() {
    // Triangle (0,0), (1,0), (0, 4e-15). Area = 2e-15, below f64::EPSILON * 100
    // = 2.22e-14. The duplicate-point check passes (distances are all near
    // unity), so the rejection lands at the top-mass branch.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 4e-15], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate(1.5).unwrap_err();
    let msg = format!("{}", err);

    assert!(msg.contains("top-dimensional simplex"));
    assert!(msg.contains("below tolerance"));
}
