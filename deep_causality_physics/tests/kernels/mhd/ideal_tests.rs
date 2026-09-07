/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::CsrMatrix;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, PhysicalField, alfven_speed_kernel, ideal_induction_kernel, magnetic_pressure_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    Manifold, PointCloud, Simplex, SimplicialComplex, SimplicialManifold, Skeleton,
};

fn create_dummy_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_alfven_speed() {
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::<f64>::new(b_vec);
    let rho = Density::<f64>::new(1.0).unwrap();
    let mu0 = 1.0;

    let res = alfven_speed_kernel(&b_field, &rho, mu0);
    assert!(res.is_ok());
    // vA = |B| / sqrt(mu0 * rho) = 1 / 1 = 1
    assert!((res.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_alfven_speed_errors() {
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::<f64>::new(b_vec);
    let rho_valid = Density::<f64>::new(1.0).unwrap();

    // Permeability error
    assert!(alfven_speed_kernel(&b_field, &rho_valid, 0.0).is_err());
    assert!(alfven_speed_kernel(&b_field, &rho_valid, -1.0).is_err());

    // Density error (zero)
    let rho_zero = Density::<f64>::new_unchecked(0.0);
    assert!(alfven_speed_kernel(&b_field, &rho_zero, 1.0).is_err());
}

#[test]
fn test_alfven_speed_negative_density_error() {
    // Density::new rejects negatives, so use new_unchecked to feed a negative
    // rho into the kernel and trip the `rho < 0` guard (ideal.rs:42-46), which
    // is distinct from the `rho == 0` Singularity guard.
    let b_vec = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::<f64>::new(b_vec);
    let rho_neg = Density::<f64>::new_unchecked(-1.0);
    assert!(alfven_speed_kernel(&b_field, &rho_neg, 1.0).is_err());
}

#[test]
fn test_magnetic_pressure() {
    let b_vec = CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::<f64>::new(b_vec);
    let mu0 = 1.0;

    let res = magnetic_pressure_kernel(&b_field, mu0);
    assert!(res.is_ok());
    // P = B^2 / 2mu0 = 4 / 2 = 2
    assert!((res.unwrap().value() - 2.0).abs() < 1e-10);
}

#[test]
fn test_magnetic_pressure_error() {
    let b_vec = CausalMultiVector::new(vec![0.0, 2.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();
    let b_field = PhysicalField::<f64>::new(b_vec);
    assert!(magnetic_pressure_kernel(&b_field, 0.0).is_err());
}

#[test]
fn test_ideal_induction_refuses_a_2d_complex_from_real_geometry() {
    // This test asserted `is_ok()` until the silent skips were removed, and it was never able to
    // see anything: `create_dummy_manifold` is a single triangle in a 2D ambient space filled with
    // `0.0` at every simplex, so the kernel's answer was zero whether or not the algebra was right.
    // Two silent-default idioms kept the shape error invisible — `if col < vector.len()` in the
    // CSR product, and `.get(i).unwrap_or(&zero)` in the wedge — and the all-zero data made the
    // truncated result indistinguishable from the correct one.
    //
    // The kernel is 3D-only: `i_v B = *(v ^ *B)` applies `*` on 2-forms and on `(n-1)`-forms as a
    // single operator, which holds only at `n = 3`. A triangle is 2D, so the refusal is correct.
    let m = create_dummy_manifold();
    let err = ideal_induction_kernel(&m, &m).unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("needs a 3D complex") && msg.contains("2-dimensional"),
        "unexpected message: {msg}"
    );
}

// NOTE on defensively-unreachable ideal-MHD branches (all in
// `ideal_induction_kernel` / its private helper `wedge_product_1form_1form`):
//   * ideal.rs:134-136 — "v_manifold data too small". `Manifold::new` rejects
//     any data tensor whose length differs from the complex's total simplex
//     count, and that total is at least n0 + n1 + n2, so the data slab is never
//     shorter than the slices the kernel takes.
//   * ideal.rs:265-267 — `wedge_product_1form_1form`'s own `skeletons.len() < 3`
//     guard. The helper is private and its only caller
//     (`ideal_induction_kernel`) has already validated `skeletons.len() >= 3`
//     before invoking it.

#[test]
fn test_ideal_induction_dimension_error() {
    // Manifold with only 0 and 1 skeletons (1D manifold/graph)
    // Points for a single line segment
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let point_cloud = PointCloud::new(
        points,
        CausalTensor::new(vec![0.0, 0.0], vec![2]).unwrap(),
        0,
    )
    .unwrap();
    let complex = point_cloud.triangulate(1.5).unwrap();
    let num = complex.total_simplices();
    let m = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap();

    let res = ideal_induction_kernel(&m, &m);
    assert!(res.is_err());
}

// =============================================================================
// Hand-built simplicial fixtures for the guard branches of
// `ideal_induction_kernel`.
//
// `PointCloud::triangulate` only ever yields well-formed complexes with a full
// operator set, so the kernel's structural guards are only observable on a
// complex assembled directly through `SimplicialComplex::new`, which takes the
// skeletons and the operator vectors verbatim.
// =============================================================================

/// Three vertices {0,1,2} as the 0-skeleton, with the 1- and 2-skeletons
/// supplied by the caller so one fixture serves the well-formed triangle and
/// its degenerate variants.
fn skeletons_of(edges: &[&[usize]], faces: &[&[usize]]) -> Vec<Skeleton> {
    let vertices: Vec<Simplex> = (0..3usize).map(|v| Simplex::new(vec![v])).collect();
    let edges: Vec<Simplex> = edges.iter().map(|e| Simplex::new(e.to_vec())).collect();
    let faces: Vec<Simplex> = faces.iter().map(|f| Simplex::new(f.to_vec())).collect();
    vec![
        Skeleton::new(0, vertices),
        Skeleton::new(1, edges),
        Skeleton::new(2, faces),
    ]
}

/// A structurally empty operator, used where the kernel never reads the matrix.
fn empty_op<T>() -> CsrMatrix<T> {
    CsrMatrix::new()
}

/// ⋆ on 2-forms: sends the single face value `b` to the edge 1-form
/// `(1·b, 2·b, 3·b)`. The three weights differ so an index slip anywhere in the
/// pipeline changes the result.
fn hodge_star_2() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(3, 1, &[(0, 0, 1.0), (1, 0, 2.0), (2, 0, 3.0)]).unwrap()
}

/// d on 1-forms: the single face reads edge0 − edge1 + edge2.
fn coboundary_1() -> CsrMatrix<i8> {
    CsrMatrix::from_triplets(1, 3, &[(0, 0, 1i8), (0, 1, -1i8), (0, 2, 1i8)]).unwrap()
}

/// Manifold data laid out as [3 vertices | 3 edges | 1 face]. The velocity
/// 1-form is v = (2, 7, 5) on the edges and the magnetic 2-form is B = 0.5 on
/// the face.
fn fixture_manifold(complex: SimplicialComplex<f64>) -> SimplicialManifold<f64, f64> {
    let data = vec![0.0, 0.0, 0.0, 2.0, 7.0, 5.0, 0.5];
    Manifold::new(complex, CausalTensor::new(data, vec![7]).unwrap(), 0).unwrap()
}

#[test]
fn test_ideal_induction_rejects_hodge_star_without_2_form_operator() {
    // The kernel needs ⋆ on 2-forms, i.e. `hodge_star_operators()[2]`. A complex
    // that carries only the dim-0 and dim-1 operators passes the skeleton-count
    // check and then fails on the operator count (ideal.rs:156-159).
    let complex = SimplicialComplex::new(
        skeletons_of(&[&[0, 1], &[0, 2], &[1, 2]], &[&[0, 1, 2]]),
        vec![],
        vec![empty_op::<i8>(), coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>()],
    );
    let m = fixture_manifold(complex);

    // 2D fixture: the kernel is 3D-only, so the dimension guard answers before the
    // branch this test was written for. It pinned the missing-⋆₂ message, which is now unreachable from a 2D complex.
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("needs a 3D complex"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_rejects_missing_coboundary_operator() {
    // d on 1-forms is `coboundary_operators()[1]`. With the Hodge ⋆ surface
    // complete but no coboundary operators, the kernel gets as far as ⋆(v ∧ ⋆B)
    // and then fails on the exterior derivative (ideal.rs:175-178).
    let complex = SimplicialComplex::new(
        skeletons_of(&[&[0, 1], &[0, 2], &[1, 2]], &[&[0, 1, 2]]),
        vec![],
        vec![],
        vec![empty_op::<f64>(), empty_op::<f64>(), hodge_star_2()],
    );
    let m = fixture_manifold(complex);

    // 2D fixture: the kernel is 3D-only, so the dimension guard answers before the
    // branch this test was written for. It pinned the missing-d₁ message, which is now unreachable from a 2D complex.
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("needs a 3D complex"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_refuses_a_2d_triangle_with_hand_built_operators() {
    // A 2D fixture. The kernel is 3D-only, so the dimension guard now answers first
    // and this fixture can no longer reach the branch it was written for.
    // It pinned dtB = 1.0 for the well-formed triangle with hand-built (3,1) operators.
    // The same property is covered in-domain by the tetrahedron fixtures below.

    // What it formerly asserted, kept as the record of the 2D convention it encoded.
    // Note the fixture's ⋆₂ is (3, 1), which on a triangle is ambiguous between faces→edges
    // (the 3D reading this kernel wants) and faces→vertices (the correct 2D one), because
    // n0 = n1 = 3. That ambiguity is why the wrong convention went unnoticed here.
    //   ∂ₜB = d(⋆(v ∧ ⋆B)) with the fixture operators:
    //   ⋆B          = (1, 2, 3)·0.5              = (0.5, 1.0, 1.5)
    //   (v ∧ ⋆B)[F] = v[0,1]·⋆B[1,2] − ⋆B[0,1]·v[1,2]
    //               = 2·1.5 − 0.5·5              = 0.5
    //   ⋆(v ∧ ⋆B)   = (1, 2, 3)·0.5              = (0.5, 1.0, 1.5)
    //   d(...)      = 0.5 − 1.0 + 1.5            = 1.0
    let complex = SimplicialComplex::new(
        skeletons_of(&[&[0, 1], &[0, 2], &[1, 2]], &[&[0, 1, 2]]),
        vec![],
        vec![empty_op::<i8>(), coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>(), hodge_star_2()],
    );
    let m = fixture_manifold(complex);
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("needs a 3D complex"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_refuses_a_2d_complex_with_a_non_triangular_face() {
    // A 2D fixture. The kernel is 3D-only, so the dimension guard now answers first
    // and this fixture can no longer reach the branch it was written for.
    // It pinned that a 4-vertex 2-skeleton entry contributes zero to the wedge.
    // The same property is covered in-domain by the tetrahedron fixtures below.

    // The wedge of two 1-forms is defined on triangles. A 2-skeleton entry with
    // four vertices carries no such value, so it contributes zero to v ∧ ⋆B and
    // the whole induction collapses to zero (ideal.rs:287-289) — against 1.0 for
    // the genuine triangle above, with identical data and operators.
    let complex = SimplicialComplex::new(
        skeletons_of(&[&[0, 1], &[0, 2], &[1, 2]], &[&[0, 1, 2, 3]]),
        vec![],
        vec![empty_op::<i8>(), coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>(), hodge_star_2()],
    );
    let m = fixture_manifold(complex);
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("needs a 3D complex"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_refuses_a_2d_complex_with_an_absent_boundary_edge() {
    // A 2D fixture. The kernel is 3D-only, so the dimension guard now answers first
    // and this fixture can no longer reach the branch it was written for.
    // It pinned that a face whose [v1,v2] edge is absent contributes zero.
    // The same property is covered in-domain by the tetrahedron fixtures below.

    // The cup product reads α on [v0,v1] and β on [v1,v2]. Here edge [1,2] is
    // absent from the 1-skeleton, so the face [0,1,2] has no [v1,v2] slot to
    // read and contributes zero (ideal.rs:309-311) — again against 1.0 for the
    // complete triangle, with identical data and operators.
    let complex = SimplicialComplex::new(
        skeletons_of(&[&[0, 1], &[0, 2], &[1, 3]], &[&[0, 1, 2]]),
        vec![],
        vec![empty_op::<i8>(), coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>(), hodge_star_2()],
    );
    let m = fixture_manifold(complex);
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("needs a 3D complex"),
        "unexpected message: {msg}"
    );
}

// =============================================================================
// 3D fixtures — a single tetrahedron.
//
// The kernel is 3D-only, so these are the first fixtures that exercise its
// arithmetic inside its own domain. Every 2D fixture above can now only reach
// the dimension guard.
//
// Vertices {0,1,2,3}. Edges in sorted order:
//   e0=(0,1) e1=(0,2) e2=(0,3) e3=(1,2) e4=(1,3) e5=(2,3)
// Faces in sorted order:
//   f0=(0,1,2) f1=(0,1,3) f2=(0,2,3) f3=(1,2,3)
// =============================================================================

/// The four skeletons of one tetrahedron, with the 1- and 2-skeletons supplied
/// so one fixture serves the well-formed cell and its degenerate variants.
fn tetra_skeletons(edges: &[&[usize]], faces: &[&[usize]]) -> Vec<Skeleton> {
    let vertices: Vec<Simplex> = (0..4usize).map(|v| Simplex::new(vec![v])).collect();
    let edges: Vec<Simplex> = edges.iter().map(|e| Simplex::new(e.to_vec())).collect();
    let faces: Vec<Simplex> = faces.iter().map(|f| Simplex::new(f.to_vec())).collect();
    let cells = vec![Simplex::new(vec![0, 1, 2, 3])];
    vec![
        Skeleton::new(0, vertices),
        Skeleton::new(1, edges),
        Skeleton::new(2, faces),
        Skeleton::new(3, cells),
    ]
}

const TETRA_EDGES: [&[usize]; 6] = [&[0, 1], &[0, 2], &[0, 3], &[1, 2], &[1, 3], &[2, 3]];
const TETRA_FACES: [&[usize]; 4] = [&[0, 1, 2], &[0, 1, 3], &[0, 2, 3], &[1, 2, 3]];

/// `⋆` on 2-forms in 3D: `Λ² → Λ¹`, so shape `(n1, n2) = (6, 4)`.
///
/// This is the degree-changing star the kernel's identity needs, and the shape
/// `deep_causality_topology` does **not** vend — its lumped-mass star is the
/// diagonal `(4, 4)`. The weights differ per column so an index slip anywhere in
/// the pipeline changes the result.
fn tetra_hodge_star_2() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(
        6,
        4,
        &[
            (0, 0, 2.0),
            (3, 0, 1.0),
            (4, 1, 2.0),
            (5, 2, 3.0),
            (0, 3, 5.0),
        ],
    )
    .unwrap()
}

/// `d` on 1-forms: `Λ¹ → Λ²`, shape `(n2, n1) = (4, 6)`.
///
/// The true coboundary of the tetrahedron: `∂f = (v1,v2) − (v0,v2) + (v0,v1)`
/// for each face, transposed.
fn tetra_coboundary_1() -> CsrMatrix<i8> {
    CsrMatrix::from_triplets(
        4,
        6,
        &[
            (0, 0, 1i8),
            (0, 1, -1i8),
            (0, 3, 1i8),
            (1, 0, 1i8),
            (1, 2, -1i8),
            (1, 4, 1i8),
            (2, 1, 1i8),
            (2, 2, -1i8),
            (2, 5, 1i8),
            (3, 3, 1i8),
            (3, 4, -1i8),
            (3, 5, 1i8),
        ],
    )
    .unwrap()
}

/// Data laid out as `[4 vertices | 6 edges | 4 faces | 1 cell]`, fifteen values.
/// The velocity 1-form is `v = (2, 0, 0, 0, 0, 0)` on the edges and the magnetic
/// 2-form is `B = (1, 0, 0, 0)` on the faces. Non-zero, unlike the 2D fixture,
/// so a wrong answer cannot coincide with the right one.
fn tetra_manifold(complex: SimplicialComplex<f64>) -> SimplicialManifold<f64, f64> {
    let data = vec![
        0.0, 0.0, 0.0, 0.0, // vertices
        2.0, 0.0, 0.0, 3.0, 0.0, 0.0, // edges: v
        1.0, 0.0, 0.0, 0.0, // faces: B
        0.0, // cell
    ];
    Manifold::new(complex, CausalTensor::new(data, vec![15]).unwrap(), 0).unwrap()
}

fn tetra_complex(edges: &[&[usize]], faces: &[&[usize]]) -> SimplicialComplex<f64> {
    SimplicialComplex::new(
        tetra_skeletons(edges, faces),
        vec![],
        vec![empty_op::<i8>(), tetra_coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>(), tetra_hodge_star_2()],
    )
}

#[test]
fn test_ideal_induction_on_a_tetrahedron() {
    // Hand-evaluated along the documented pipeline `∂ₜB = d(⋆(v ∧ ⋆B))`.
    //
    // Both wedge terms are non-zero here, deliberately. An earlier version of this fixture had
    // `⋆B` reaching only one edge, which left `β[e01]·α[e12] = 0` on every face — and a wedge that
    // ADDS its two terms instead of subtracting them gives the same answer when one is zero. A
    // mutant flipping that sign survived. Making `⋆₂`'s first column reach `e0` as well as `e3`
    // puts a non-zero value on both sides of the subtraction.
    //
    //   ⋆B          = column 0 of ⋆₂, scaled by B[f0] = 1  = (2,0,0,1,0,0)
    //   v                                                  = (2,0,0,3,0,0)
    //   (v ∧ ⋆B)[f] = v[e01]·⋆B[e12] − ⋆B[e01]·v[e12]
    //     f0 = (0,1,2): e01=e0, e12=e3 → 2·1 − 2·3 = −4
    //     f1 = (0,1,3): e01=e0, e12=e4 → 2·0 − 2·0 =  0
    //     f2 = (0,2,3): e01=e1, e12=e5 → 0·0 − 0·0 =  0
    //     f3 = (1,2,3): e01=e3, e12=e5 → 3·0 − 1·0 =  0
    //   v ∧ ⋆B      = (−4,0,0,0)
    //   ⋆(v ∧ ⋆B)   = column 0 of ⋆₂, scaled by −4         = (−8,0,0,−4,0,0)
    //   d(...)      row f0: +e0·(−8) −e1·0 +e3·(−4) = −12
    //               row f1: +e0·(−8) −e2·0 +e4·0    =  −8
    //               row f2: +e1·0    −e2·0 +e5·0    =   0
    //               row f3: +e3·(−4) −e4·0 +e5·0    =  −4
    let m = tetra_manifold(tetra_complex(&TETRA_EDGES, &TETRA_FACES));
    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    assert_eq!(dt_b.shape(), &[4]);
    let got = dt_b.as_slice();
    for (i, want) in [-12.0f64, -8.0, 0.0, -4.0].into_iter().enumerate() {
        assert!(
            (got[i] - want).abs() < 1e-12,
            "component {i}: expected {want}, got {}",
            got[i]
        );
    }
}

#[test]
fn test_ideal_induction_refuses_the_diagonal_hodge_star() {
    // The shape the crate actually vends: `⋆₂` diagonal on 2-cells, `(4, 4)`,
    // where this kernel needs the degree-changing `(6, 4)`. This is the exact
    // mismatch a complex built from real geometry produces, isolated from the
    // dimension guard by using a genuine 3D fixture.
    let diagonal_star =
        CsrMatrix::from_triplets(4, 4, &[(0, 0, 1.0), (1, 1, 1.0), (2, 2, 1.0), (3, 3, 1.0)])
            .unwrap();
    let complex = SimplicialComplex::new(
        tetra_skeletons(&TETRA_EDGES, &TETRA_FACES),
        vec![],
        vec![empty_op::<i8>(), tetra_coboundary_1()],
        vec![empty_op::<f64>(), empty_op::<f64>(), diagonal_star],
    );
    let m = tetra_manifold(complex);
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("degree-changing Hodge star") && msg.contains("(4, 4)"),
        "unexpected message: {msg}"
    );
}
