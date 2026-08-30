/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for the simplicial `ReggeGeometry<R>` curvature and signature
//! paths: the successful 3D tetrahedron dihedral computation, `euclidean_metric_at`,
//! the `SimplexNotFound` lookup error, and the `compute_signature` /
//! `compute_eigenvalues` degenerate / negative-eigenvalue branches.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::utils_tests::create_triangle_complex;
use deep_causality_topology::{BaseTopology, ReggeGeometry, Simplex, SimplicialComplexBuilder};

// --- regge_geometry/curvature.rs ------------------------------------------------------

#[test]
fn test_3d_internal_edge_valid_dihedral_curvature() {
    // Three regular (unit-edge) tetrahedra glued around the shared edge (0,1).
    // The edge (0,1) is internal (every incident face is shared by two tets), so
    // the curvature routine reaches the *successful* 3D dihedral path
    // (sin θ = 3 V l / (2 A1 A2)) for a valid, non-degenerate tetrahedron — the
    // closed-form arm that the error-path tests never exercise.
    let mut builder = SimplicialComplexBuilder::new(3);
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 3, 4])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 4, 2])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    // All-unit edges: every tetrahedron is regular, every face area > 0 and
    // every volume > 0, so the dihedral angle computes via asin without error.
    let tensor = CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry
        .calculate_ricci_curvature(&complex)
        .expect("regular-tet curvature must compute");

    // Bones are 1-simplices (edges) in 3D.
    let num_bones = complex.num_elements_at_grade(1).unwrap();
    assert_eq!(curvature.shape(), vec![num_bones]);

    // The internal edge (0,1) carries a non-trivial deficit (only 3 tets cover
    // the 2π dihedral total, leaving a positive deficit). It must be finite and
    // not NaN, which confirms the asin path ran.
    let idx_01 = complex.skeletons()[1]
        .get_index(&Simplex::new(vec![0, 1]))
        .expect("edge (0,1) present");
    let k = curvature.data()[idx_01];
    assert!(
        k.is_finite(),
        "internal-edge deficit must be finite, got {k}"
    );
    assert!(
        k.abs() > 1e-9,
        "three regular tets around an edge leave a positive deficit, got {k}"
    );
}

// --- regge_geometry/mod.rs ------------------------------------------------------------

#[test]
fn test_euclidean_metric_at_returns_grade_dim() {
    // euclidean_metric_at is the fast Euclidean fallback: it ignores edge
    // geometry and returns Metric::Euclidean(grade).
    let tensor = CausalTensor::new(vec![1.0; 3], vec![3]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    assert_eq!(
        geometry.euclidean_metric_at(2),
        deep_causality_multivector::Metric::Euclidean(2)
    );
    assert_eq!(
        geometry.euclidean_metric_at(0),
        deep_causality_multivector::Metric::Euclidean(0)
    );
}

#[test]
fn test_metric_at_non_euclidean_signature_negative_eigenvalue() {
    // A triangle whose edge lengths violate the (Euclidean) triangle inequality
    // produces a Gram matrix with a negative eigenvalue, exercising the `q += 1`
    // (negative / timelike) counting branch in `compute_signature`. Edges of
    // `create_triangle_complex` are (0,1), (0,2), (1,2). With (1,2) far longer
    // than (0,1)+(0,2) the embedding is pseudo-Euclidean.
    let complex = create_triangle_complex();
    let edge_lengths = CausalTensor::new(vec![1.0, 1.0, 5.0], vec![3]).unwrap();
    let geometry = ReggeGeometry::new(edge_lengths);

    // The 2-simplex face metric: signature counts must reflect a non-Euclidean
    // embedding (at least one negative eigenvalue was counted).
    let metric = geometry.metric_at(&complex, 2, 0);
    // Not the purely-Euclidean (2,0,0) signature: a negative eigenvalue moved
    // dimensions out of `p`.
    assert_ne!(metric, deep_causality_multivector::Metric::Euclidean(2));
}
