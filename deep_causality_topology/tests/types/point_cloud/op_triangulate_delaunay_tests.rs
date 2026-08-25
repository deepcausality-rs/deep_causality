/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `PointCloud::triangulate_delaunay`.
//!
//! Coverage:
//! - Degenerate-input rejection (3 classes: `D != 2`, `n < 3`, collinear)
//! - Three-point triangle (Delaunay-trivial baseline)
//! - Unit-square cocircular case (4 vertices, 5 edges, 2 triangles)
//! - Random non-degenerate fuzz: empty-circumcircle invariant
//! - Manifold compatibility: `Manifold::with_metric` accepts every output

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, TopologyErrorEnum};

// -----------------------------------------------------------------------------
// Degenerate-input rejection
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_delaunay_rejects_non_2d_ambient() {
    let points = CausalTensor::new(
        vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        vec![4, 3],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate_delaunay().unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("2D ambient") || msg.contains("D == 2"),
        "must name the 2D-only constraint; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_delaunay_rejects_fewer_than_three_points() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate_delaunay().unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("at least 3 points"),
        "must name the minimum-vertex requirement; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_delaunay_rejects_collinear_input() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 2.0, 0.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate_delaunay().unwrap_err();
    let msg = format!("{}", err);
    assert!(
        msg.contains("non-collinear"),
        "must name the non-collinearity requirement; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_delaunay_rejects_duplicate_input_points() {
    let points =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let err = pc.triangulate_delaunay().unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("duplicate point"));
}

// -----------------------------------------------------------------------------
// Non-degenerate inputs
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_delaunay_three_point_triangle() {
    // Canonical right triangle.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();
    assert_eq!(complex.skeletons()[0].simplices().len(), 3);
    assert_eq!(complex.skeletons()[1].simplices().len(), 3);
    assert_eq!(complex.skeletons()[2].simplices().len(), 1);
}

#[test]
fn test_triangulate_delaunay_unit_square_cocircular() {
    // Four corners of the unit square — the canonical cocircular case.
    // Per `spec.md`: 4 vertices, 5 edges (4 sides + 1 diagonal), 2 triangles.
    let points =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();
    assert_eq!(complex.skeletons()[0].simplices().len(), 4, "vertices");
    assert_eq!(complex.skeletons()[1].simplices().len(), 5, "edges");
    assert_eq!(complex.skeletons()[2].simplices().len(), 2, "triangles");
}

#[test]
fn test_triangulate_delaunay_unit_square_is_deterministic() {
    // Same input → same output across repeated calls.
    let make_pc = || {
        let points =
            CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], vec![4, 2]).unwrap();
        let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
        PointCloud::new(points, metadata, 0).unwrap()
    };

    let complex_a = make_pc().triangulate_delaunay().unwrap();
    let complex_b = make_pc().triangulate_delaunay().unwrap();

    let edges_a: Vec<Vec<usize>> = complex_a.skeletons()[1]
        .simplices()
        .iter()
        .map(|s| s.vertices().clone())
        .collect();
    let edges_b: Vec<Vec<usize>> = complex_b.skeletons()[1]
        .simplices()
        .iter()
        .map(|s| s.vertices().clone())
        .collect();
    assert_eq!(edges_a, edges_b, "edge set deterministic across calls");

    let tris_a: Vec<Vec<usize>> = complex_a.skeletons()[2]
        .simplices()
        .iter()
        .map(|s| s.vertices().clone())
        .collect();
    let tris_b: Vec<Vec<usize>> = complex_b.skeletons()[2]
        .simplices()
        .iter()
        .map(|s| s.vertices().clone())
        .collect();
    assert_eq!(tris_a, tris_b, "triangle set deterministic across calls");
}

#[test]
fn test_triangulate_delaunay_no_super_vertex_artefacts() {
    // After super-vertex removal, every output triangle's vertices reference
    // real input points (indices < n).
    let points = CausalTensor::new(
        vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.5, 0.5],
        vec![5, 2],
    )
    .unwrap();
    let metadata = CausalTensor::new(vec![1.0; 5], vec![5]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();
    let n = 5;
    for tri in complex.skeletons()[2].simplices() {
        for &v in tri.vertices() {
            assert!(
                v < n,
                "triangle vertex {} >= n={} (super-triangle leakage)",
                v,
                n
            );
        }
    }
}

#[test]
fn test_triangulate_delaunay_random_input_empty_circumcircle() {
    // Pseudo-random non-cocircular points. The Delaunay property: no input
    // vertex lies strictly inside the circumcircle of any output triangle.
    let raw: Vec<f64> = vec![
        0.13, 0.27, 0.71, 0.42, 0.93, 0.81, 0.18, 0.66, 0.55, 0.36, 0.31, 0.92, 0.84, 0.07,
    ];
    let n = raw.len() / 2;
    let points = CausalTensor::new(raw.clone(), vec![n, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0; n], vec![n]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();

    let triangles: Vec<Vec<usize>> = complex.skeletons()[2]
        .simplices()
        .iter()
        .map(|s| s.vertices().clone())
        .collect();

    let tol = f64::EPSILON * 100.0;
    for tri in &triangles {
        let (i, j, k) = (tri[0], tri[1], tri[2]);
        let a = (raw[i * 2], raw[i * 2 + 1]);
        let b = (raw[j * 2], raw[j * 2 + 1]);
        let c = (raw[k * 2], raw[k * 2 + 1]);
        for p in 0..n {
            if p == i || p == j || p == k {
                continue;
            }
            let d = (raw[p * 2], raw[p * 2 + 1]);
            // Compute signed in-circumcircle determinant; account for triangle orientation.
            let ax = a.0 - d.0;
            let ay = a.1 - d.1;
            let bx = b.0 - d.0;
            let by = b.1 - d.1;
            let cx = c.0 - d.0;
            let cy = c.1 - d.1;
            let a_sq = ax * ax + ay * ay;
            let b_sq = bx * bx + by * by;
            let c_sq = cx * cx + cy * cy;
            let det = ax * (by * c_sq - cy * b_sq) - ay * (bx * c_sq - cx * b_sq)
                + a_sq * (bx * cy - cx * by);
            // Triangle orientation: signed area of (a, b, c).
            let signed_area_2 = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0);
            // For CCW triangles (signed_area_2 > 0): det > 0 means p is inside the circle.
            // For CW triangles (signed_area_2 < 0): det < 0 means p is inside.
            // We normalize by multiplying det by sign of signed_area_2.
            let inside_signed = if signed_area_2 > 0.0 { det } else { -det };
            assert!(
                inside_signed <= tol,
                "Delaunay property violated: point {} is strictly inside circumcircle of triangle {:?} (signed det = {})",
                p,
                tri,
                inside_signed
            );
        }
    }
}

// -----------------------------------------------------------------------------
// Manifold compatibility
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_delaunay_unit_square_accepted_by_manifold_with_metric() {
    // The whole point of `triangulate_delaunay`: the output passes
    // `Manifold::with_metric`'s manifold-property check, which the
    // Vietoris-Rips path fails on this exact input.
    let points =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0], vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();
    let n_edges = complex.skeletons()[1].simplices().len();
    let unit_square: [(f64, f64); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    let edge_lengths: Vec<f64> = complex.skeletons()[1]
        .simplices()
        .iter()
        .map(|s| {
            let v = s.vertices();
            let a = unit_square[v[0]];
            let b = unit_square[v[1]];
            ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
        })
        .collect();
    let regge = ReggeGeometry::new(CausalTensor::new(edge_lengths, vec![n_edges]).unwrap());

    let total = complex.total_simplices();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::with_metric(complex, data, Some(regge), 0);
    assert!(
        manifold.is_ok(),
        "Manifold::with_metric must accept the Delaunay unit square: {:?}",
        manifold.err()
    );
}

#[test]
fn test_triangulate_delaunay_three_point_accepted_by_manifold_with_metric() {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0], vec![3, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().unwrap();
    let n_edges = complex.skeletons()[1].simplices().len();
    // Edges of the (0,0)-(1,0)-(0,1) triangle: lengths 1, 1, sqrt(2).
    let coords: [(f64, f64); 3] = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)];
    let edge_lengths: Vec<f64> = complex.skeletons()[1]
        .simplices()
        .iter()
        .map(|s| {
            let v = s.vertices();
            let a = coords[v[0]];
            let b = coords[v[1]];
            ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
        })
        .collect();
    let regge = ReggeGeometry::new(CausalTensor::new(edge_lengths, vec![n_edges]).unwrap());

    let total = complex.total_simplices();
    let data = CausalTensor::new(vec![0.0; total], vec![total]).unwrap();
    let manifold = Manifold::with_metric(complex, data, Some(regge), 0);
    assert!(
        manifold.is_ok(),
        "Manifold::with_metric must accept three-point Delaunay output: {:?}",
        manifold.err()
    );
}

/// The super-triangle's bounding box is taken over every input point, not over
/// the first one. A cloud whose first point is interior to the others still
/// triangulates to the full Delaunay complex of the four points: one interior
/// vertex inside a triangular hull gives 3 triangles and 6 edges (V − E + F = 1
/// for a disk).
#[test]
fn test_triangulate_delaunay_when_first_point_is_not_the_extreme() {
    use deep_causality_topology::{BaseTopology, ChainComplex};

    // Point 0 is strictly inside the hull of points 1..3, so min/max in both
    // axes are set by later points.
    let points =
        CausalTensor::new(vec![1.0, 0.5, 0.0, 0.0, 2.0, 0.0, 1.0, 2.0], vec![4, 2]).unwrap();
    let metadata = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).unwrap();

    let complex = pc.triangulate_delaunay().expect("non-degenerate input");

    assert_eq!(complex.num_elements_at_grade(0).unwrap(), 4);
    assert_eq!(
        complex.num_cells(2),
        3,
        "one interior point in a triangular hull fans into three triangles"
    );
    assert_eq!(
        complex.num_cells(1),
        6,
        "Euler: 4 - E + 3 = 1 for a triangulated disk"
    );
}

// -----------------------------------------------------------------------------
// Shape-contract rejection
//
// Precondition 1 covers both halves of the shape: the `points` tensor is rank 2
// and its ambient dimension is 2. A rank-1 tensor has no axis 1, so the check
// runs before any indexing.
// -----------------------------------------------------------------------------

#[test]
fn test_triangulate_delaunay_rejects_rank_one_points_tensor() {
    use deep_causality_topology::TopologyError;

    let points = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap();
    let metadata = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).expect("rank-1 cloud constructs");

    let err = pc.triangulate_delaunay().unwrap_err();
    assert!(
        matches!(err, TopologyError(TopologyErrorEnum::PointCloudError(_))),
        "rank-1 coordinates are a documented precondition violation; got: {:?}",
        err
    );
    let msg = format!("{}", err);
    assert!(
        msg.contains("rank-2") && msg.contains("rank 1"),
        "message must name the required and the supplied rank; got: {}",
        msg
    );
}

#[test]
fn test_triangulate_delaunay_rejects_rank_three_points_tensor() {
    use deep_causality_topology::TopologyError;

    let points =
        CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![2, 2, 2]).unwrap();
    let metadata = CausalTensor::new(vec![0.0; 2], vec![2]).unwrap();
    let pc = PointCloud::new(points, metadata, 0).expect("rank-3 cloud constructs");

    let err = pc.triangulate_delaunay().unwrap_err();
    assert!(
        matches!(err, TopologyError(TopologyErrorEnum::PointCloudError(_))),
        "rank-3 coordinates are a precondition violation; got: {:?}",
        err
    );
    assert!(
        format!("{}", err).contains("rank 3"),
        "message must name the supplied rank; got: {}",
        err
    );
}
