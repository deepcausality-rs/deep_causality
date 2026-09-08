/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The simplicial interior product, by Whitney interpolation.
//!
//! # The oracle
//!
//! Whitney forms reproduce **constant** vector fields exactly. So for constant `V` and `B` the whole
//! chain is exact and the expected answer is a closed form that never touches the implementation:
//!
//! ```text
//! v[e_ab]     = V · (p_b − p_a)                      (circulation along an edge)
//! B[f_ijk]    = B · ½ (p_j − p_i) × (p_k − p_i)      (flux through a face)
//! (i_V B)[e]  = (B × V) · (p_b − p_a)                (what the operator must return)
//! ```
//!
//! `i_V B = (B × V)♭` is read off the components of the contraction — see the module docs on the
//! implementation — and the three lines above are evaluated by hand in the test, not by calling
//! anything under test. Every other test here is an algebraic invariant: bilinearity, the vanishing
//! of the contraction when the two fields are parallel, and independence of the mesh used.

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, Simplex, SimplicialComplex, Skeleton, TopologyErrorEnum};

// =============================================================================
// Fixtures
// =============================================================================

/// Builds a 3D simplicial complex from a tetrahedron list and vertex coordinates, enumerating the
/// edge and face skeletons in sorted order.
fn tet_complex(tets: &[[usize; 4]], coords: &[[f64; 3]]) -> SimplicialComplex<f64> {
    let mut edges: Vec<Vec<usize>> = Vec::new();
    let mut faces: Vec<Vec<usize>> = Vec::new();
    for t in tets {
        for i in 0..4 {
            for j in (i + 1)..4 {
                edges.push(vec![t[i], t[j]]);
                for k in (j + 1)..4 {
                    faces.push(vec![t[i], t[j], t[k]]);
                }
            }
        }
    }
    edges.sort();
    edges.dedup();
    faces.sort();
    faces.dedup();

    let verts: Vec<Simplex> = (0..coords.len()).map(|i| Simplex::new(vec![i])).collect();
    let skeletons = vec![
        Skeleton::new(0, verts),
        Skeleton::new(1, edges.iter().map(|e| Simplex::new(e.clone())).collect()),
        Skeleton::new(2, faces.iter().map(|f| Simplex::new(f.clone())).collect()),
        Skeleton::new(3, tets.iter().map(|t| Simplex::new(t.to_vec())).collect()),
    ];
    let flat: Vec<f64> = coords.iter().flatten().copied().collect();
    SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), flat, 3)
}

fn manifold(complex: SimplicialComplex<f64>) -> Manifold<SimplicialComplex<f64>, f64> {
    let n: usize = complex
        .skeletons()
        .iter()
        .map(|s| s.simplices().len())
        .sum();
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; n], vec![n]).unwrap(),
        0,
    )
    .unwrap()
}

/// A single regular tetrahedron of edge length 1.
fn one_tet() -> (Vec<[usize; 4]>, Vec<[f64; 3]>) {
    let s = 1.0 / (2.0_f64).sqrt();
    (
        vec![[0, 1, 2, 3]],
        vec![[0.0, 0.0, 0.0], [s, s, 0.0], [s, 0.0, s], [0.0, s, s]],
    )
}

/// Two tetrahedra sharing the face `[1, 2, 3]`, so interior edges take a mean over both.
fn two_tets() -> (Vec<[usize; 4]>, Vec<[f64; 3]>) {
    (
        vec![[0, 1, 2, 3], [1, 2, 3, 4]],
        vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.9, 0.8, 0.7],
        ],
    )
}

/// The unit cube cut into six tetrahedra (the Freudenthal / Kuhn subdivision), an irregular mesh
/// with genuinely shared interior faces and edges.
fn six_tet_cube() -> (Vec<[usize; 4]>, Vec<[f64; 3]>) {
    let coords: Vec<[f64; 3]> = (0..8)
        .map(|i| [(i & 1) as f64, ((i >> 1) & 1) as f64, ((i >> 2) & 1) as f64])
        .collect();
    // Kuhn subdivision: the six paths from 000 to 111 monotone in the bit order.
    let tets = vec![
        [0, 1, 3, 7],
        [0, 1, 5, 7],
        [0, 2, 3, 7],
        [0, 2, 6, 7],
        [0, 4, 5, 7],
        [0, 4, 6, 7],
    ];
    (tets, coords)
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// The 1-cochain of a constant field: its circulation along each edge.
fn constant_one_cochain(c: &SimplicialComplex<f64>, coords: &[[f64; 3]], v: [f64; 3]) -> Vec<f64> {
    c.skeletons()[1]
        .simplices()
        .iter()
        .map(|e| {
            let vs = e.vertices();
            dot3(v, sub3(coords[vs[1]], coords[vs[0]]))
        })
        .collect()
}

/// The 2-cochain of a constant field: its flux through each face, with the orientation induced by
/// the sorted vertex order.
fn constant_two_cochain(c: &SimplicialComplex<f64>, coords: &[[f64; 3]], b: [f64; 3]) -> Vec<f64> {
    c.skeletons()[2]
        .simplices()
        .iter()
        .map(|f| {
            let vs = f.vertices();
            let e1 = sub3(coords[vs[1]], coords[vs[0]]);
            let e2 = sub3(coords[vs[2]], coords[vs[0]]);
            let area = cross3(e1, e2);
            0.5 * dot3(b, area)
        })
        .collect()
}

fn tensor(v: Vec<f64>) -> CausalTensor<f64> {
    let n = v.len();
    CausalTensor::new(v, vec![n]).unwrap()
}

/// Runs the operator on a mesh with constant `V` and `B`, and checks every edge against the closed
/// form `(B × V) · (p_b − p_a)`.
fn assert_constant_field_case(tets: &[[usize; 4]], coords: &[[f64; 3]], v: [f64; 3], b: [f64; 3]) {
    let complex = tet_complex(tets, coords);
    let v_cochain = constant_one_cochain(&complex, coords, v);
    let b_cochain = constant_two_cochain(&complex, coords, b);
    let edges: Vec<Vec<usize>> = complex.skeletons()[1]
        .simplices()
        .iter()
        .map(|e| e.vertices().clone())
        .collect();
    let m = manifold(complex);

    let got = m
        .interior_product(&tensor(v_cochain), &tensor(b_cochain), 2)
        .expect("a well-formed 3D complex with geometry");

    let bxv = cross3(b, v);
    for (idx, e) in edges.iter().enumerate() {
        let want = dot3(bxv, sub3(coords[e[1]], coords[e[0]]));
        let scale = 1.0 + want.abs();
        assert!(
            (got.as_slice()[idx] - want).abs() <= 1e-11 * scale,
            "edge {e:?}: got {}, closed form (B × V)·(p_b − p_a) = {want}",
            got.as_slice()[idx]
        );
    }
}

// =============================================================================
// The closed form, on three meshes
// =============================================================================

#[test]
fn test_constant_fields_on_one_tetrahedron_match_the_closed_form() {
    let (tets, coords) = one_tet();
    assert_constant_field_case(&tets, &coords, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_constant_field_case(&tets, &coords, [0.3, -1.7, 2.2], [-0.9, 0.4, 1.1]);
}

#[test]
fn test_constant_fields_on_two_tetrahedra_match_the_closed_form() {
    // The shared face means four of the edges are averaged over two tetrahedra. Averaging equal
    // contributions must leave them equal, which is what this checks in passing.
    let (tets, coords) = two_tets();
    assert_constant_field_case(&tets, &coords, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    assert_constant_field_case(&tets, &coords, [-2.5, 0.6, 1.9], [1.3, -0.7, 0.2]);
}

#[test]
fn test_constant_fields_on_a_six_tetrahedron_cube_match_the_closed_form() {
    // An irregular mesh: the Kuhn subdivision has tetrahedra of three different shapes and a
    // diagonal edge shared by all six.
    let (tets, coords) = six_tet_cube();
    assert_constant_field_case(&tets, &coords, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    assert_constant_field_case(&tets, &coords, [0.7, 1.4, -0.3], [2.0, -0.5, 0.8]);
    assert_constant_field_case(&tets, &coords, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
}

// =============================================================================
// Algebraic invariants
// =============================================================================

#[test]
fn test_parallel_fields_contract_to_zero() {
    // `i_V B = (B × V)♭`, so a `B` parallel to `V` contracts to nothing at all — on every edge, on
    // every mesh. An invariant that no amount of interpolation error can fake.
    let (tets, coords) = six_tet_cube();
    let v = [0.6, -1.2, 0.9];
    let b = [-1.8, 3.6, -2.7]; // exactly −3 · v
    let complex = tet_complex(&tets, &coords);
    let v_c = constant_one_cochain(&complex, &coords, v);
    let b_c = constant_two_cochain(&complex, &coords, b);
    let m = manifold(complex);

    let got = m.interior_product(&tensor(v_c), &tensor(b_c), 2).unwrap();
    for (i, x) in got.as_slice().iter().enumerate() {
        assert!(x.abs() < 1e-12, "edge {i} is not zero: {x}");
    }
}

#[test]
fn test_the_contraction_is_bilinear() {
    // Linear in each argument separately, over arbitrary cochains — not only the constant-field
    // ones. This is the property that pins the operator as a contraction rather than as any
    // formula that happens to agree on constants.
    let (tets, coords) = six_tet_cube();
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);

    let v1: Vec<f64> = (0..n1).map(|i| 0.3 * (i as f64) - 1.1).collect();
    let v2: Vec<f64> = (0..n1).map(|i| 1.7 - 0.2 * (i as f64)).collect();
    let b1: Vec<f64> = (0..n2).map(|i| 0.9 - 0.4 * (i as f64)).collect();
    let b2: Vec<f64> = (0..n2).map(|i| 0.05 * (i as f64) * (i as f64)).collect();
    let alpha = -2.3;

    // Linear in the contraction field.
    let combined: Vec<f64> = v1.iter().zip(&v2).map(|(a, b)| alpha * a + b).collect();
    let lhs = m
        .interior_product(&tensor(combined), &tensor(b1.clone()), 2)
        .unwrap();
    let r1 = m
        .interior_product(&tensor(v1.clone()), &tensor(b1.clone()), 2)
        .unwrap();
    let r2 = m
        .interior_product(&tensor(v2.clone()), &tensor(b1.clone()), 2)
        .unwrap();
    for i in 0..n1 {
        let want = alpha * r1.as_slice()[i] + r2.as_slice()[i];
        assert!(
            (lhs.as_slice()[i] - want).abs() <= 1e-10 * (1.0 + want.abs()),
            "edge {i}: {} vs {want}",
            lhs.as_slice()[i]
        );
    }

    // Linear in the form.
    let combined_b: Vec<f64> = b1.iter().zip(&b2).map(|(a, b)| alpha * a + b).collect();
    let lhs = m
        .interior_product(&tensor(v1.clone()), &tensor(combined_b), 2)
        .unwrap();
    let s2 = m.interior_product(&tensor(v1), &tensor(b2), 2).unwrap();
    for i in 0..n1 {
        let want = alpha * r1.as_slice()[i] + s2.as_slice()[i];
        assert!(
            (lhs.as_slice()[i] - want).abs() <= 1e-10 * (1.0 + want.abs()),
            "edge {i}: {} vs {want}",
            lhs.as_slice()[i]
        );
    }
}

#[test]
fn test_a_zero_field_contracts_to_zero() {
    let (tets, coords) = two_tets();
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);

    let got = m
        .interior_product(&tensor(vec![0.0; n1]), &tensor(vec![1.0; n2]), 2)
        .unwrap();
    assert!(got.as_slice().iter().all(|x| *x == 0.0));

    let got = m
        .interior_product(&tensor(vec![1.0; n1]), &tensor(vec![0.0; n2]), 2)
        .unwrap();
    assert!(got.as_slice().iter().all(|x| *x == 0.0));
}

#[test]
fn test_the_result_is_a_one_cochain() {
    let (tets, coords) = six_tet_cube();
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);
    let got = m
        .interior_product(&tensor(vec![0.5; n1]), &tensor(vec![0.25; n2]), 2)
        .unwrap();
    assert_eq!(got.len(), n1, "the contraction of a 2-form is a 1-form");
}

// =============================================================================
// Refusals
// =============================================================================

#[test]
fn test_a_grade_other_than_two_is_refused() {
    // Stated rather than approximated: `k = 1` and `k = 3` project onto vertices and onto faces,
    // which are different constructions, and an unchecked operator has no business in a solver.
    let (tets, coords) = one_tet();
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);
    for k in [0usize, 1, 3, 4] {
        let err = m
            .interior_product(&tensor(vec![1.0; n1]), &tensor(vec![1.0; n2]), k)
            .unwrap_err();
        assert!(
            matches!(err.0, TopologyErrorEnum::InvalidGradeOperation(_)),
            "k = {k} gave {err}"
        );
    }
}

#[test]
fn test_a_two_dimensional_complex_is_refused() {
    // A single triangle: three skeletons, so no tetrahedra to interpolate on.
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
    let coords = vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0];
    let complex: SimplicialComplex<f64> =
        SimplicialComplex::with_geometry(skeletons, Vec::new(), Vec::new(), coords, 3);
    let m = manifold(complex);
    let err = m
        .interior_product(&tensor(vec![1.0; 3]), &tensor(vec![1.0; 1]), 2)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::InvalidGradeOperation(_)),
        "got {err}"
    );
}

#[test]
fn test_a_complex_without_coordinates_is_refused() {
    // Whitney interpolation is geometric. A complex built through the legacy path carries no
    // coordinates, and the refusal says so rather than inventing any.
    let (tets, coords) = one_tet();
    let geometric = tet_complex(&tets, &coords);
    let skeletons: Vec<Skeleton> = geometric
        .skeletons()
        .iter()
        .map(|s| Skeleton::new(s.dim(), s.simplices().clone()))
        .collect();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();
    let bare: SimplicialComplex<f64> =
        SimplicialComplex::new(skeletons, Vec::new(), Vec::new(), Vec::new());
    let m = manifold(bare);
    let err = m
        .interior_product(&tensor(vec![1.0; n1]), &tensor(vec![1.0; n2]), 2)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::InvalidInput(_)),
        "got {err}"
    );
}

#[test]
fn test_wrong_cochain_lengths_are_refused() {
    let (tets, coords) = one_tet();
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);

    let err = m
        .interior_product(&tensor(vec![1.0; n1 + 1]), &tensor(vec![1.0; n2]), 2)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::DimensionMismatch(_)),
        "got {err}"
    );

    let err = m
        .interior_product(&tensor(vec![1.0; n1]), &tensor(vec![1.0; n2 - 1]), 2)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::DimensionMismatch(_)),
        "got {err}"
    );
}

#[test]
fn test_a_degenerate_tetrahedron_is_refused() {
    // Four coplanar points have no barycentric coordinates: the interpolation matrix is singular.
    let tets = vec![[0, 1, 2, 3]];
    let coords = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    let complex = tet_complex(&tets, &coords);
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let m = manifold(complex);
    let err = m
        .interior_product(&tensor(vec![1.0; n1]), &tensor(vec![1.0; n2]), 2)
        .unwrap_err();
    assert!(
        matches!(err.0, TopologyErrorEnum::InvalidInput(_)),
        "got {err}"
    );
}
