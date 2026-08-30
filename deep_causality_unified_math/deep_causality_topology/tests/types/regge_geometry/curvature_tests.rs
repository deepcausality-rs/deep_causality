/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::f64::consts::PI;

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, ReggeGeometry, Simplex, SimplicialComplexBuilder};

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_2d_flat_triangle_boundary() {
    let mut builder = SimplicialComplexBuilder::new(2);
    builder.add_simplex(Simplex::new(vec![0, 1, 2])).unwrap(); // Will implicitly add edges and vertices
    let complex = builder.build::<f64>().unwrap();

    // Metric: 3 edges.
    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();

    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry
        .calculate_ricci_curvature(&complex)
        .expect("Calculation failed");

    // 2D -> Curvature at vertices (0)
    let num_verts = complex.num_elements_at_grade(0).unwrap();
    assert_eq!(curvature.shape(), vec![num_verts]);

    // Boundary vertices have 0 curvature
    for &val in curvature.data() {
        assert_eq!(val, 0.0);
    }
}

#[test]
fn test_3d_flat_tetrahedron_boundary() {
    let mut builder = SimplicialComplexBuilder::new(3);
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();

    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry
        .calculate_ricci_curvature(&complex)
        .expect("Calculation failed");

    // 3D -> Curvature at edges (bones are n-2 = 1)
    let num_bones = complex.num_elements_at_grade(1).unwrap();
    assert_eq!(curvature.shape(), vec![num_bones]);

    for &val in curvature.data() {
        assert_eq!(val, 0.0);
    }
}

#[test]
fn test_2d_internal_flat_hexagon() {
    let mut builder = SimplicialComplexBuilder::new(2);
    // Center 0.
    // 6 Triangles around 0.
    let indices = [1, 2, 3, 4, 5, 6];
    for i in 0..6 {
        let v1 = indices[i];
        let v2 = indices[(i + 1) % 6];
        builder.add_simplex(Simplex::new(vec![0, v1, v2])).unwrap();
    }
    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry.calculate_ricci_curvature(&complex).unwrap();

    // Check center (0)
    let idx_0 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![0]))
        .unwrap();
    let k_0 = curvature.data()[idx_0];

    assert!(
        k_0.abs() < 1e-6,
        "Flat hexagon center should have 0 curvature. Got {}",
        k_0
    );
}

#[test]
fn test_2d_internal_positive_curvature_pentagon() {
    let mut builder = SimplicialComplexBuilder::new(2);
    let indices = [1, 2, 3, 4, 5];
    for i in 0..5 {
        let v1 = indices[i];
        let v2 = indices[(i + 1) % 5];
        builder.add_simplex(Simplex::new(vec![0, v1, v2])).unwrap();
    }
    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let lengths = vec![1.0; num_edges];
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let curvature = geometry.calculate_ricci_curvature(&complex).unwrap();

    let idx_0 = complex.skeletons()[0]
        .get_index(&Simplex::new(vec![0]))
        .unwrap();
    let k_0 = curvature.data()[idx_0];

    // 360 - 5*60 = 60 deg = PI/3
    let expected = PI / 3.0;
    assert!((k_0 - expected).abs() < 1e-6, "Expected PI/3, got {}", k_0);
}

#[test]
fn test_dimension_mismatch_error() {
    let mut builder = SimplicialComplexBuilder::new(1);
    builder.add_simplex(Simplex::new(vec![0, 1])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let tensor = CausalTensor::new(vec![1.0], vec![1]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    assert!(matches!(
        res.err().unwrap().0,
        deep_causality_topology::TopologyErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_2d_triangle_inequality_violation_errors() {
    // 2D pentagon around vertex 0 — set one rim edge to a huge length so the
    // triangle inequality fails in the 2D angle calculation (line 136 branch).
    let mut builder = SimplicialComplexBuilder::new(2);
    let indices = [1, 2, 3, 4, 5];
    for i in 0..5 {
        let v1 = indices[i];
        let v2 = indices[(i + 1) % 5];
        builder.add_simplex(Simplex::new(vec![0, v1, v2])).unwrap();
    }
    let complex = builder.build::<f64>().unwrap();
    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let mut lengths = vec![1.0; num_edges];

    // Find rim edge (1, 2) and inflate it so 1 + 1 < 100 (violates inequality on
    // triangle 0-1-2 with spokes 0-1=1, 0-2=1, rim 1-2=100).
    if let Some(idx) = complex.skeletons()[1].get_index(&Simplex::new(vec![1, 2])) {
        lengths[idx] = 100.0;
    }
    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let err = geometry.calculate_ricci_curvature(&complex).unwrap_err();
    assert!(matches!(
        err.0,
        deep_causality_topology::TopologyErrorEnum::ManifoldError(_)
    ));
}

#[test]
fn test_manifold_error_triangle_inequality() {
    // We need an INTERNAL bone for calculation to occur.
    // In 3D, create 3 tets around edge (0,1):
    // T1: 0,1,2,3
    // T2: 0,1,3,4
    // T3: 0,1,4,2
    // Edge (0,1) is shared by faces (0,1,2), (0,1,3), (0,1,4).
    // All these faces are shared by 2 tets, so they are internal.
    // Thus edge (0,1) is internal.

    let mut builder = SimplicialComplexBuilder::new(3);
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 3, 4])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 4, 2])).unwrap();
    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let mut lengths = vec![1.0; num_edges];

    // Set edge 0-2 to 10.0, while 0-1 and 1-2 are 1.0.
    // Triangle (0,1,2) will violate inequality Check.
    // We need to find index of 0-2.
    if let Some(idx) = complex.skeletons()[1].get_index(&Simplex::new(vec![0, 2])) {
        lengths[idx] = 10.0;
    } else {
        panic!("Edge 0-2 not found");
    }

    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err(), "Should return error for impossible triangle");

    let err = res.err().unwrap();
    if let deep_causality_topology::TopologyErrorEnum::ManifoldError(msg) = err.0 {
        assert!(msg.contains("Triangle inequality") || msg.contains("Degenerate"));
    } else {
        panic!("Expected ManifoldError, got {:?}", err);
    }
}

#[test]
fn test_curvature_missing_boundary_operators() {
    // Manually construct a complex without boundary operators
    // Vertices {0, 1, 2}
    // Face {0, 1, 2}
    let s0 = deep_causality_topology::Skeleton::new(
        0,
        vec![
            Simplex::new(vec![0]),
            Simplex::new(vec![1]),
            Simplex::new(vec![2]),
        ],
    );
    let s1 = deep_causality_topology::Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![1, 2]),
        ],
    );
    let s2 = deep_causality_topology::Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]);

    // Empty boundary ops
    let complex =
        deep_causality_topology::SimplicialComplex::new(vec![s0, s1, s2], vec![], vec![], vec![]);

    let num_edges = 3;
    let tensor = CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    if let deep_causality_topology::TopologyErrorEnum::InvalidInput(msg) = res.err().unwrap().0 {
        assert!(msg.contains("boundary operators"));
    } else {
        panic!("Expected InvalidInput for missing boundary operators");
    }
}

#[test]
fn test_3d_degenerate_tetrahedron_face() {
    // Degenerate face: 3 vertices collinear.
    // 0, 1, 2 collinear.
    // Tet T1: 0, 1, 2, 3.
    // Edges of face (0,1,2): (0,1)=1 (Bone), (1,2)=1, (0,2)=2.
    // Area(0,1,2) = 0 because 1+1=2 (Triangle inequality satisfied but area zero/degenerate).
    // Or actually 1+1=2 is degenerate triangle.

    // We need edge (0,1) to be internal to trigger calculation involving face (0,1,2).
    // So we need other tets around (0,1).
    // T2: 0,1,3,4.
    // T3: 0,1,4,2.

    let mut builder = SimplicialComplexBuilder::new(3);
    // Add all 3 tets
    builder.add_simplex(Simplex::new(vec![0, 1, 2, 3])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 3, 4])).unwrap();
    builder.add_simplex(Simplex::new(vec![0, 1, 4, 2])).unwrap();

    let complex = builder.build::<f64>().unwrap();

    let num_edges = complex.num_elements_at_grade(1).unwrap();
    let mut lengths = vec![1.0; num_edges];

    // Set (0,2) length to 2.0.
    // (0,1) is 1.0 default. (1,2) is 1.0 default.
    // So face (0,1,2) has sides 1, 1, 2 -> Area 0.
    let edge_02 = Simplex::new(vec![0, 2]);
    if let Some(idx) = complex.skeletons()[1].get_index(&edge_02) {
        lengths[idx] = 2.0;
    } else {
        // If (0,2) is not explicitly an edge?
        // Simplex(0,1,2,3) contains (0,2). It should be there.
        panic!("Edge (0,2) not found in complex");
    }

    let tensor = CausalTensor::new(lengths, vec![num_edges]).unwrap();
    let geometry = ReggeGeometry::new(tensor);

    let res = geometry.calculate_ricci_curvature(&complex);
    assert!(res.is_err());
    let err = res.err().unwrap();
    let err_msg = format!("{:?}", err);

    // Check for specific error
    if !err_msg.contains("Degenerate face")
        && !err_msg.contains("Triangle inequality")
        && !err_msg.contains("Tetrahedron inequality violated")
    {
        panic!("Test Failed. Unexpected error message: {}", err_msg);
    }
}

// ============================================================================
// Structural guards of the dihedral-angle computation.
//
// `calculate_ricci_curvature` reads the bone/face/top-cell incidence from the
// complex's stored boundary operators and the vertex lists from its skeletons.
// The tests below hand-build complexes whose incidence and vertex lists
// disagree, which is what each guard is there to catch.
// ============================================================================

use deep_causality_linear::CsrMatrix;
use deep_causality_topology::{SimplicialComplex, Skeleton};

fn verts(n: usize) -> Skeleton {
    Skeleton::new(0, (0..n).map(|i| Simplex::new(vec![i])).collect())
}

fn geometry(lengths: Vec<f64>) -> ReggeGeometry<f64> {
    let n = lengths.len();
    ReggeGeometry::new(CausalTensor::new(lengths, vec![n]).unwrap())
}

/// In 2D the bone is a vertex of the triangle whose angle is being measured, so
/// removing it must leave exactly two vertices. An incidence matrix that routes
/// a vertex to a triangle it is not part of leaves three, and the calculation
/// rejects the complex instead of indexing into the wrong pair.
#[test]
fn two_dimensional_bone_outside_its_triangle_is_rejected() {
    let sk1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    // Two top cells so the single edge reads as interior, neither containing vertex 0.
    let sk2 = Skeleton::new(
        2,
        vec![Simplex::new(vec![1, 2, 3]), Simplex::new(vec![1, 2, 3])],
    );
    let d1 = CsrMatrix::from_triplets(4, 1, &[(0, 0, -1i8), (1, 0, 1)]).unwrap();
    let d2 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1i8), (0, 1, -1)]).unwrap();

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![verts(4), sk1, sk2], vec![d1, d2], vec![], vec![]);

    let err = geometry(vec![1.0])
        .calculate_ricci_curvature(&complex)
        .unwrap_err();
    match err.0 {
        deep_causality_topology::TopologyErrorEnum::InvalidInput(msg) => {
            assert_eq!(msg, "Simplex topology error");
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

/// Every edge of a triangle whose angle is measured must be present in the
/// 1-skeleton, since its length is read from the metric by skeleton index. A
/// missing edge is reported rather than silently treated as zero-length.
#[test]
fn missing_edge_length_is_reported_as_simplex_not_found() {
    // The triangle [0,1,2] needs edges [0,1], [0,2] and [1,2]; only [0,1] exists.
    let sk1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let sk2 = Skeleton::new(
        2,
        vec![Simplex::new(vec![0, 1, 2]), Simplex::new(vec![0, 1, 2])],
    );
    let d1 = CsrMatrix::from_triplets(3, 1, &[(0, 0, -1i8), (1, 0, 1)]).unwrap();
    let d2 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1i8), (0, 1, -1)]).unwrap();

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![verts(3), sk1, sk2], vec![d1, d2], vec![], vec![]);

    let err = geometry(vec![1.0])
        .calculate_ricci_curvature(&complex)
        .unwrap_err();
    assert_eq!(
        err,
        deep_causality_topology::TopologyError::SimplexNotFound()
    );
}

/// In 3D the bone is an edge of the tetrahedron, so removing its two endpoints
/// must leave exactly the two opposite vertices. An incidence matrix that routes
/// an edge to a tetrahedron that does not contain it leaves four.
#[test]
fn three_dimensional_bone_outside_its_tetrahedron_is_rejected() {
    let sk1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let sk2 = Skeleton::new(2, vec![Simplex::new(vec![2, 3, 4])]);
    let sk3 = Skeleton::new(
        3,
        vec![
            Simplex::new(vec![2, 3, 4, 5]),
            Simplex::new(vec![2, 3, 4, 5]),
        ],
    );
    let d1 = CsrMatrix::from_triplets(6, 1, &[(0, 0, -1i8), (1, 0, 1)]).unwrap();
    let d2 = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1i8)]).unwrap();
    let d3 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1i8), (0, 1, -1)]).unwrap();

    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![verts(6), sk1, sk2, sk3],
        vec![d1, d2, d3],
        vec![],
        vec![],
    );

    let err = geometry(vec![1.0])
        .calculate_ricci_curvature(&complex)
        .unwrap_err();
    match err.0 {
        deep_causality_topology::TopologyErrorEnum::InvalidInput(msg) => {
            assert_eq!(msg, "Bone not in simplex");
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

/// The 3D dihedral angle divides by the areas of the two faces meeting at the
/// bone. Edge lengths that make one of those faces degenerate (1, 1, 2 — the
/// three points are collinear) give it zero area, and the calculation reports
/// the degeneracy rather than dividing by it.
///
/// The tetrahedron here is `[0,1,2,3]` with `|01| = |12| = |13| = 1`,
/// `|02| = |03| = 2` and `|23| = 0`. Both faces at the bone `[0,1]` are the
/// degenerate `(1, 1, 2)` triangle, and the Cayley-Menger matrix has two equal
/// rows, so the volume is exactly zero and the area guard is what fires.
#[test]
fn degenerate_face_at_the_bone_is_reported() {
    let edges = vec![
        Simplex::new(vec![0, 1]),
        Simplex::new(vec![0, 2]),
        Simplex::new(vec![0, 3]),
        Simplex::new(vec![1, 2]),
        Simplex::new(vec![1, 3]),
        Simplex::new(vec![2, 3]),
    ];
    let faces = vec![
        Simplex::new(vec![0, 1, 2]),
        Simplex::new(vec![0, 1, 3]),
        Simplex::new(vec![0, 2, 3]),
        Simplex::new(vec![1, 2, 3]),
    ];
    let sk1 = Skeleton::new(1, edges);
    let sk2 = Skeleton::new(2, faces);
    let sk3 = Skeleton::new(
        3,
        vec![
            Simplex::new(vec![0, 1, 2, 3]),
            Simplex::new(vec![0, 1, 2, 3]),
        ],
    );

    let d1 = CsrMatrix::from_triplets(
        4,
        6,
        &[
            (0, 0, -1i8),
            (1, 0, 1),
            (0, 1, -1),
            (2, 1, 1),
            (0, 2, -1),
            (3, 2, 1),
            (1, 3, -1),
            (2, 3, 1),
            (1, 4, -1),
            (3, 4, 1),
            (2, 5, -1),
            (3, 5, 1),
        ],
    )
    .unwrap();
    // Edge-face incidence: [0,1,2] = e0,e1,e3; [0,1,3] = e0,e2,e4;
    // [0,2,3] = e1,e2,e5; [1,2,3] = e3,e4,e5.
    let d2 = CsrMatrix::from_triplets(
        6,
        4,
        &[
            (0, 0, 1i8),
            (1, 0, -1),
            (3, 0, 1),
            (0, 1, 1),
            (2, 1, -1),
            (4, 1, 1),
            (1, 2, 1),
            (2, 2, -1),
            (5, 2, 1),
            (3, 3, 1),
            (4, 3, -1),
            (5, 3, 1),
        ],
    )
    .unwrap();
    // Every face carries two top cells, so no bone reads as a boundary bone.
    let d3 = CsrMatrix::from_triplets(
        4,
        2,
        &[
            (0, 0, 1i8),
            (0, 1, -1),
            (1, 0, 1),
            (1, 1, -1),
            (2, 0, 1),
            (2, 1, -1),
            (3, 0, 1),
            (3, 1, -1),
        ],
    )
    .unwrap();

    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![verts(4), sk1, sk2, sk3],
        vec![d1, d2, d3],
        vec![],
        vec![],
    );

    // Lengths in 1-skeleton order: [0,1], [0,2], [0,3], [1,2], [1,3], [2,3].
    let err = geometry(vec![1.0, 2.0, 2.0, 1.0, 1.0, 0.0])
        .calculate_ricci_curvature(&complex)
        .unwrap_err();
    match err.0 {
        deep_causality_topology::TopologyErrorEnum::ManifoldError(msg) => {
            assert_eq!(msg, "Degenerate face in tetrahedron");
        }
        other => panic!("expected ManifoldError, got {other:?}"),
    }
}

/// The dihedral angle has closed forms for triangles and tetrahedra only. A top
/// cell with five vertices (a 4-simplex) is not implemented, so it contributes a
/// zero angle and every bone keeps the full 2π as its deficit.
#[test]
fn four_dimensional_top_cell_contributes_no_angle() {
    let sk1 = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let sk2 = Skeleton::new(
        2,
        vec![
            Simplex::new(vec![0, 1, 2, 3, 4]),
            Simplex::new(vec![0, 1, 2, 3, 5]),
        ],
    );
    let d1 = CsrMatrix::from_triplets(6, 1, &[(0, 0, -1i8), (1, 0, 1)]).unwrap();
    let d2 = CsrMatrix::from_triplets(1, 2, &[(0, 0, 1i8), (0, 1, -1)]).unwrap();

    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![verts(6), sk1, sk2], vec![d1, d2], vec![], vec![]);

    let curvature = geometry(vec![1.0])
        .calculate_ricci_curvature(&complex)
        .expect("an unimplemented dimension is not an error");

    assert_eq!(curvature.shape(), vec![6]);
    for &val in curvature.data() {
        assert!(
            (val - 2.0 * PI).abs() < 1e-12,
            "no angle was subtracted, so the deficit is the full turn; got {val}"
        );
    }
}
