/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the lazy lumped-mass Hodge ⋆ population.

use deep_causality_topology::{Simplex, SimplicialComplex, Skeleton};

#[test]
fn test_lumped_mass_gives_a_vertexless_cell_zero_volume() {
    // A simplex with no vertices spans nothing, so its Cayley-Menger volume is
    // zero. Its lumped-mass dual volume is then zero as well, and a zero is not
    // stored in the sparse diagonal.
    let skeletons = vec![Skeleton::new(0, vec![Simplex::new(vec![])])];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), Vec::new(), 2);

    let ops = complex
        .hodge_star_operators()
        .expect("a zero-dimensional complex has no top-volume degeneracy to reject");

    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0].shape(), (1, 1));
    assert!(
        ops[0].values().is_empty(),
        "a cell of zero volume carries no lumped mass"
    );
}

#[test]
fn test_lumped_mass_vertex_dual_volume_is_the_incident_edge_length_halved() {
    // A single edge of length 3 between two vertices. Each endpoint's dual
    // volume is the incident primal volume divided by max_dim + 1 = 2, so both
    // vertex entries are 1.5, and the top-grade entry is 1 / 3.
    let skeletons = vec![
        Skeleton::new(0, vec![Simplex::new(vec![0]), Simplex::new(vec![1])]),
        Skeleton::new(1, vec![Simplex::new(vec![0, 1])]),
    ];
    let coords = vec![0.0, 0.0, 3.0, 0.0];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 2);

    let ops = complex.hodge_star_operators().expect("edge length 3 > 0");

    assert_eq!(ops.len(), 2);
    assert_eq!(ops[0].shape(), (2, 2));
    assert_eq!(ops[0].values(), &[1.5, 1.5]);
    assert_eq!(ops[1].shape(), (1, 1));
    assert!((ops[1].values()[0] - 1.0 / 3.0).abs() < 1e-12);
}

// =============================================================================
// Intermediate-grade masses
//
// The expectations below are closed forms of the lumped Whitney mass,
//     M[sigma] = sum over cells T of   integral over T of  W_sigma . W_sigma  dV,
// evaluated by hand on reference simplices and cross-checked against an independent numpy
// implementation of the same formula.
// =============================================================================

/// The diagonal of a grade's operator, in simplex order, with absent entries read as zero.
fn diagonal(op: &deep_causality_linear::CsrMatrix<f64>) -> Vec<f64> {
    let (rows, _) = op.shape();
    (0..rows)
        .map(|i| {
            let (s, e) = (op.row_indices()[i], op.row_indices()[i + 1]);
            (s..e)
                .find(|&idx| op.col_indices()[idx] == i)
                .map(|idx| op.values()[idx])
                .unwrap_or(0.0)
        })
        .collect()
}

/// The reference triangle (0,0), (1,0), (0,1), scaled by `h`.
fn reference_triangle(h: f64) -> SimplicialComplex<f64> {
    let skeletons = vec![
        Skeleton::new(
            0,
            vec![
                Simplex::new(vec![0]),
                Simplex::new(vec![1]),
                Simplex::new(vec![2]),
            ],
        ),
        Skeleton::new(
            1,
            vec![
                Simplex::new(vec![0, 1]),
                Simplex::new(vec![0, 2]),
                Simplex::new(vec![1, 2]),
            ],
        ),
        Skeleton::new(2, vec![Simplex::new(vec![0, 1, 2])]),
    ];
    let coords = vec![0.0, 0.0, h, 0.0, 0.0, h];
    SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 2)
}

/// The reference tetrahedron (0,0,0), (1,0,0), (0,1,0), (0,0,1), scaled by `h`.
fn reference_tetrahedron(h: f64) -> SimplicialComplex<f64> {
    let verts: Vec<Simplex> = (0..4).map(|v| Simplex::new(vec![v])).collect();
    let mut edges = Vec::new();
    let mut faces = Vec::new();
    for i in 0..4 {
        for j in (i + 1)..4 {
            edges.push(Simplex::new(vec![i, j]));
            for k in (j + 1)..4 {
                faces.push(Simplex::new(vec![i, j, k]));
            }
        }
    }
    edges.sort_by_key(|s| s.vertices().to_vec());
    faces.sort_by_key(|s| s.vertices().to_vec());
    let skeletons = vec![
        Skeleton::new(0, verts),
        Skeleton::new(1, edges),
        Skeleton::new(2, faces),
        Skeleton::new(3, vec![Simplex::new(vec![0, 1, 2, 3])]),
    ];
    let coords = vec![0.0, 0.0, 0.0, h, 0.0, 0.0, 0.0, h, 0.0, 0.0, 0.0, h];
    SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 3)
}

#[test]
fn test_edge_mass_on_the_reference_triangle_is_the_whitney_mass() {
    // With grad(l0) = (-1,-1), grad(l1) = (1,0), grad(l2) = (0,1) and |T| = 1/2,
    //     M[e_ij] = (2|T| / 12) (|grad l_i|^2 + |grad l_j|^2 - grad l_i . grad l_j)
    // giving 1/3, 1/3 and 1/6 for e01, e02 and e12. The edge lengths, 1, 1 and sqrt(2), are a
    // different quantity and the message below names them so a failure is easy to read.
    let complex = reference_triangle(1.0);
    let ops = complex.hodge_star_operators().unwrap();
    let got = diagonal(&ops[1]);
    let expected = [1.0 / 3.0, 1.0 / 3.0, 1.0 / 6.0];
    for (i, (g, e)) in got.iter().zip(expected.iter()).enumerate() {
        assert!(
            (g - e).abs() < 1e-12,
            "edge {i}: got {g}, expected {e} (the edge length would be {})",
            if i == 2 { 2.0_f64.sqrt() } else { 1.0 }
        );
    }
}

#[test]
fn test_intermediate_masses_on_the_reference_tetrahedron() {
    // |T| = 1/6. Edges of the reference tetrahedron come in two classes: the three meeting the
    // origin and the three opposite it.
    //     k = 1: 1/12 for e01, 1/30 for e12
    //     k = 2: 8/15 for f012, 1/5 for f123
    let complex = reference_tetrahedron(1.0);
    let ops = complex.hodge_star_operators().unwrap();

    let edges = diagonal(&ops[1]);
    // Edge order is lexicographic: 01, 02, 03, 12, 13, 23.
    for (i, e) in [0usize, 1, 2].iter().enumerate() {
        assert!(
            (edges[*e] - 1.0 / 12.0).abs() < 1e-12,
            "edge {i} at the origin: got {}, expected 1/12",
            edges[*e]
        );
    }
    for e in [3usize, 4, 5] {
        assert!(
            (edges[e] - 1.0 / 30.0).abs() < 1e-12,
            "edge {e} opposite the origin: got {}, expected 1/30",
            edges[e]
        );
    }

    // Face order is lexicographic: 012, 013, 023, 123.
    let faces = diagonal(&ops[2]);
    for f in [0usize, 1, 2] {
        assert!(
            (faces[f] - 8.0 / 15.0).abs() < 1e-12,
            "face {f} at the origin: got {}, expected 8/15",
            faces[f]
        );
    }
    assert!(
        (faces[3] - 1.0 / 5.0).abs() < 1e-12,
        "face 123: got {}, expected 1/5",
        faces[3]
    );
}

#[test]
fn test_intermediate_masses_scale_as_h_to_the_n_minus_two_k() {
    // The lumped Whitney mass carries dimension h^(n-2k). Refining the mesh by h must move
    // every grade by exactly that power, and the grade the code got wrong -- faces in three
    // dimensions -- is the one where the exponent is negative.
    //
    // At k = 1 in three dimensions the exponent is 1, which is also the exponent of an edge
    // length, so the defect was invisible to a refinement study there. Only the closed forms
    // above separate the two at that grade; this test records the coincidence rather than
    // relying on it.
    let h = 3.0;

    // n = 2: M_1 is dimensionless, so scaling the triangle leaves it unchanged.
    let tri_one = reference_triangle(1.0);
    let tri_h = reference_triangle(h);
    let flat = diagonal(&tri_one.hodge_star_operators().unwrap()[1]);
    let scaled = diagonal(&tri_h.hodge_star_operators().unwrap()[1]);
    for (i, (a, b)) in flat.iter().zip(scaled.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-12,
            "2D edge {i}: mass moved from {a} to {b} under scaling; h^(2-2) = 1"
        );
    }

    // n = 3: edges go as h^1, faces as h^-1.
    let tet_one = reference_tetrahedron(1.0);
    let tet_h = reference_tetrahedron(h);
    let one = tet_one.hodge_star_operators().unwrap();
    let big = tet_h.hodge_star_operators().unwrap();

    let (e1, eh) = (diagonal(&one[1]), diagonal(&big[1]));
    for (i, (a, b)) in e1.iter().zip(eh.iter()).enumerate() {
        assert!(
            (a * h - b).abs() < 1e-12,
            "3D edge {i}: expected {} = h * {a}, got {b}",
            a * h
        );
    }

    let (f1, fh) = (diagonal(&one[2]), diagonal(&big[2]));
    for (i, (a, b)) in f1.iter().zip(fh.iter()).enumerate() {
        assert!(
            (a / h - b).abs() < 1e-12,
            "3D face {i}: expected {} = {a} / h, got {b}; the primal area would have scaled as h^2",
            a / h
        );
    }
}

#[test]
fn test_vertex_and_edge_masses_sum_over_every_cell_carrying_the_simplex() {
    // Two triangles sharing edge 12: T1 = (0,0), (1,0), (0,1) and T2 = (1,0), (0,1), (2,2).
    // T2 is not congruent to T1, so the shared edge's two contributions differ, and the edges
    // each carried by a single cell differ from the shared one.
    //     T1: e01 = 1/3, e02 = 1/3, e12 = 1/6
    //     T2: e12 = 7/18, e13 = 2/9, e23 = 2/9
    // from M[e_ij] = (2|T| / 12)(|grad l_i|^2 + |grad l_j|^2 - grad l_i . grad l_j), cross-checked
    // with numpy. Edge 12 carries 1/6 + 7/18 = 5/9. Vertex 1 is the first vertex of edges 12 and
    // 13 and lies on both cells, so for edge 13 one of the cells at its first vertex does not
    // carry it and must contribute nothing.
    //
    // Vertex masses are the incident areas over n + 1 = 3, with |T1| = 1/2 and |T2| = 3/2:
    //     v0 = 1/6, v1 = v2 = 2/3, v3 = 1/2, and v4, on no cell, 0.
    let skeletons = vec![
        Skeleton::new(0, (0..5).map(|v| Simplex::new(vec![v])).collect()),
        Skeleton::new(
            1,
            vec![
                Simplex::new(vec![0, 1]),
                Simplex::new(vec![0, 2]),
                Simplex::new(vec![1, 2]),
                Simplex::new(vec![1, 3]),
                Simplex::new(vec![2, 3]),
            ],
        ),
        Skeleton::new(
            2,
            vec![Simplex::new(vec![0, 1, 2]), Simplex::new(vec![1, 2, 3])],
        ),
    ];
    let coords = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0, 2.0, 5.0, 5.0];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 2);
    let ops = complex.hodge_star_operators().unwrap();

    let vertices = diagonal(&ops[0]);
    let expected = [1.0 / 6.0, 2.0 / 3.0, 2.0 / 3.0, 1.0 / 2.0, 0.0];
    for (i, (g, e)) in vertices.iter().zip(expected.iter()).enumerate() {
        assert!((g - e).abs() < 1e-12, "vertex {i}: got {g}, expected {e}");
    }

    let edges = diagonal(&ops[1]);
    let expected = [1.0 / 3.0, 1.0 / 3.0, 5.0 / 9.0, 2.0 / 9.0, 2.0 / 9.0];
    for (i, (g, e)) in edges.iter().zip(expected.iter()).enumerate() {
        assert!((g - e).abs() < 1e-12, "edge {i}: got {g}, expected {e}");
    }
}
