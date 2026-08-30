/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Degenerate-shape branches of the manifold validation helpers behind
//! `ManifoldTopology` (`is_oriented`, `satisfies_link_condition`,
//! `has_boundary`): a complex with no simplices above dimension 0, a complex
//! whose boundary operator was never supplied, a link-condition rejection at
//! construction, and a closed surface.

use deep_causality_linear::CsrMatrix;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, ManifoldTopology, Simplex, SimplicialComplex, Skeleton, TopologyErrorEnum,
};

fn vertices(n: usize) -> Skeleton {
    Skeleton::new(0, (0..n).map(|i| Simplex::new(vec![i])).collect())
}

fn zeros(n: usize) -> CausalTensor<f64> {
    CausalTensor::new(vec![0.0; n], vec![n]).unwrap()
}

/// A complex of isolated points is a 0-manifold: it is oriented, it satisfies
/// the link condition vacuously, and it has no boundary.
#[test]
fn point_cloud_complex_is_a_zero_dimensional_manifold_without_boundary() {
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![vertices(3)], vec![], vec![], vec![]);
    let m = Manifold::new(complex, zeros(3), 0).expect("three isolated points form a 0-manifold");

    assert!(m.is_oriented(), "points carry no orientation to violate");
    assert!(m.satisfies_link_condition());
    assert!(
        !m.has_boundary(),
        "a 0-manifold has no codimension-1 faces, so no boundary"
    );
    assert_eq!(m.euler_characteristic(), 3);
}

/// With no boundary operator supplied the top-grade incidence matrix is empty,
/// so the orientation test has no row sum to reject and the boundary test has no
/// row to find a single coface in. Both answer from the empty matrix instead of
/// indexing into it.
#[test]
fn complex_without_boundary_operators_reads_as_oriented_and_boundaryless() {
    // Two vertices and one edge, but the 1-skeleton's incidence matrix is absent.
    let edges = Skeleton::new(1, vec![Simplex::new(vec![0, 1])]);
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![vertices(2), edges], vec![], vec![], vec![]);
    let m = Manifold::new(complex, zeros(3), 0).expect("segment passes both manifold checks");

    assert!(m.is_oriented());
    assert!(!m.has_boundary());
}

/// The link condition rejects a vertex whose link is neither a sphere nor a
/// disk. In a 1-complex the link of a vertex is its incident-edge count, which
/// must be 2 (sphere) or 1 (disk); a degree-3 hub is neither, so the manifold
/// constructor refuses the complex.
#[test]
fn degree_three_hub_fails_the_link_condition() {
    let edges = Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![0, 3]),
        ],
    );
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::new(vec![vertices(4), edges], vec![], vec![], vec![]);

    let err = Manifold::new(complex, zeros(7), 0).expect_err("a degree-3 hub is not a 1-manifold");
    match err.0 {
        TopologyErrorEnum::ManifoldError(msg) => {
            assert!(
                msg.contains("manifold properties"),
                "unexpected message: {msg}"
            );
        }
        other => panic!("expected ManifoldError, got {other:?}"),
    }
}

/// The boundary of a tetrahedron is a closed surface: every edge is shared by
/// exactly two triangles, so no edge is a boundary face and `has_boundary` walks
/// the whole incidence matrix before answering `false`.
#[test]
fn closed_surface_has_no_boundary() {
    // Vertices 0..3; edges [01],[02],[03],[12],[13],[23]; faces [012],[013],[023],[123].
    let edges = Skeleton::new(
        1,
        vec![
            Simplex::new(vec![0, 1]),
            Simplex::new(vec![0, 2]),
            Simplex::new(vec![0, 3]),
            Simplex::new(vec![1, 2]),
            Simplex::new(vec![1, 3]),
            Simplex::new(vec![2, 3]),
        ],
    );
    let faces = Skeleton::new(
        2,
        vec![
            Simplex::new(vec![0, 1, 2]),
            Simplex::new(vec![0, 1, 3]),
            Simplex::new(vec![0, 2, 3]),
            Simplex::new(vec![1, 2, 3]),
        ],
    );

    // d1: 4 vertices x 6 edges, standard endpoint incidence.
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

    // d2: 6 edges x 4 faces. Each edge sits in exactly two faces, signed so the
    // two contributions cancel — a consistently oriented closed surface.
    let d2 = CsrMatrix::from_triplets(
        6,
        4,
        &[
            (0, 0, 1i8),
            (0, 1, -1), // edge [0,1] in faces [0,1,2] and [0,1,3]
            (1, 0, 1),
            (1, 2, -1), // edge [0,2] in faces [0,1,2] and [0,2,3]
            (2, 1, 1),
            (2, 2, -1), // edge [0,3] in faces [0,1,3] and [0,2,3]
            (3, 0, 1),
            (3, 3, -1), // edge [1,2] in faces [0,1,2] and [1,2,3]
            (4, 1, 1),
            (4, 3, -1), // edge [1,3] in faces [0,1,3] and [1,2,3]
            (5, 2, 1),
            (5, 3, -1), // edge [2,3] in faces [0,2,3] and [1,2,3]
        ],
    )
    .unwrap();

    let complex: SimplicialComplex<f64> = SimplicialComplex::new(
        vec![vertices(4), edges, faces],
        vec![d1, d2],
        vec![],
        vec![],
    );
    let m = Manifold::new(complex, zeros(14), 0).expect("oriented closed surface");

    assert!(m.is_oriented());
    assert!(
        !m.has_boundary(),
        "every edge of the tetrahedron boundary carries two faces"
    );
    // V - E + F = 4 - 6 + 4 = 2, the Euler characteristic of the 2-sphere.
    assert_eq!(m.euler_characteristic(), 2);
    assert!(m.is_manifold());
}
