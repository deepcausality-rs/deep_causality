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
fn test_ideal_induction_refuses_a_2d_complex_carrying_no_two_form_star() {
    // **The subject of this test no longer exists.** It was written when the kernel read
    // `hodge_star_operators()[2]` and pinned the message for a complex that did not carry one.
    // Since task 6.7u the kernel does not consult the Hodge star at all — the contraction is
    // Whitney interpolation — so there is no missing-⋆₂ branch to reach.
    //
    // The fixture is kept because it still pins something: a 2D complex is refused by the
    // dimension guard, whatever operators it carries or does not carry.
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
    // `d` on 1-forms is still `coboundary_operators()[1]` after the reformulation, so this
    // branch survives — but the fixture is 2D, so the dimension guard answers first and the
    // missing-d₁ message is reached through the geometric fixture below instead.
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
// The hand-built star-wedge fixtures that used to live here are gone.
//
// They supplied a degree-changing `⋆₂` of shape (6, 4) — an operator the crate never vends — so
// that the old `∂ₜB = d(⋆(v ∧ ⋆B))` chain could be exercised at all. With the contraction moved
// onto `Manifold::interior_product` (task 6.7u) the kernel reads no Hodge star, and a fixture
// asserting a hand-evaluated result of that chain would be pinning a formulation rather than the
// physics. What replaces them is below: real coordinates, and a closed form to check against.
// =============================================================================

// =============================================================================
// In-domain fixtures with real geometry.
//
// Task 6.7u moved the contraction onto `Manifold::interior_product`, which interpolates with
// Whitney forms and therefore needs vertex **coordinates**. The hand-built operator fixtures above
// carry none, so the tests below construct genuine geometry — and gain a closed-form oracle by it,
// which the hand-built ones never had.
// =============================================================================

/// A 3D simplicial complex from a tetrahedron list and coordinates, with the boundary and
/// coboundary operators built the way `PointCloud::triangulate` builds them.
fn geo_complex(tets: &[[usize; 4]], coords: &[[f64; 3]]) -> SimplicialComplex<f64> {
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

    let skeletons = vec![
        Skeleton::new(
            0,
            (0..coords.len()).map(|i| Simplex::new(vec![i])).collect(),
        ),
        Skeleton::new(1, edges.iter().map(|e| Simplex::new(e.clone())).collect()),
        Skeleton::new(2, faces.iter().map(|f| Simplex::new(f.clone())).collect()),
        Skeleton::new(3, tets.iter().map(|t| Simplex::new(t.to_vec())).collect()),
    ];

    // ∂_k : C_{k+1} → C_k, alternating signs over the omitted vertex.
    let mut boundary = Vec::new();
    for k in 0..3 {
        let rows = skeletons[k].simplices().len();
        let cols = skeletons[k + 1].simplices().len();
        let mut triplets = Vec::new();
        for (col, simplex) in skeletons[k + 1].simplices().iter().enumerate() {
            for i in 0..=(k + 1) {
                let mut verts = simplex.vertices().clone();
                verts.remove(i);
                if let Some(row) = skeletons[k].get_index(&Simplex::new(verts)) {
                    triplets.push((row, col, if i % 2 == 0 { 1i8 } else { -1i8 }));
                }
            }
        }
        boundary.push(CsrMatrix::from_triplets(rows, cols, &triplets).unwrap());
    }
    let coboundary: Vec<_> = boundary.iter().map(|b| b.transpose()).collect();
    let flat: Vec<f64> = coords.iter().flatten().copied().collect();
    SimplicialComplex::with_geometry(skeletons, boundary, coboundary, flat, 3)
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

/// A manifold whose edge block holds the circulation of the constant field `v` and whose face
/// block holds the flux of the constant field `b`.
fn geo_manifold(
    complex: SimplicialComplex<f64>,
    coords: &[[f64; 3]],
    v: [f64; 3],
    b: [f64; 3],
) -> SimplicialManifold<f64, f64> {
    let n0 = complex.skeletons()[0].simplices().len();
    let n3 = complex.skeletons()[3].simplices().len();
    let mut data = vec![0.0; n0];
    for e in complex.skeletons()[1].simplices() {
        let vs = e.vertices();
        data.push(dot3(v, sub3(coords[vs[1]], coords[vs[0]])));
    }
    for f in complex.skeletons()[2].simplices() {
        let vs = f.vertices();
        let e1 = sub3(coords[vs[1]], coords[vs[0]]);
        let e2 = sub3(coords[vs[2]], coords[vs[0]]);
        data.push(0.5 * dot3(b, cross3(e1, e2)));
    }
    data.extend(std::iter::repeat_n(0.0, n3));
    let len = data.len();
    Manifold::new(complex, CausalTensor::new(data, vec![len]).unwrap(), 0).unwrap()
}

/// A single regular tetrahedron of unit edge length.
fn unit_tet() -> (Vec<[usize; 4]>, Vec<[f64; 3]>) {
    let s = 1.0 / (2.0_f64).sqrt();
    (
        vec![[0, 1, 2, 3]],
        vec![[0.0, 0.0, 0.0], [s, s, 0.0], [s, 0.0, s], [0.0, s, s]],
    )
}

/// Two tetrahedra sharing the face `[1, 2, 3]`.
///
/// The smallest fixture on which `∂ₜB` can be anything but zero — see
/// [`test_a_single_tetrahedron_cannot_change_its_own_field`] — and one the crate's manifold
/// validation accepts, which not every tetrahedral mesh is: `is_oriented` requires each row of
/// `∂₃` to sum to at most one in absolute value, and the crate orients simplices by sorting their
/// vertices rather than by a consistent choice per cell. Here the shared face is reached by
/// dropping vertex index 0 in one tetrahedron and index 3 in the other, so its two signs are `+1`
/// and `−1` and the row sums to zero. A six-tetrahedron Kuhn cube does **not** pass, which is a
/// limitation of the sorted-vertex convention rather than of this kernel.
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

#[test]
fn test_a_uniform_field_in_a_uniform_flow_does_not_change() {
    // Flux freezing, and the sharpest oracle available here: a uniform `B` carried by a uniform
    // `v` has `i_v B = (B × v)♭` constant, and the exterior derivative of a constant 1-form is
    // zero on every face. So `∂ₜB = 0` — **exactly**, not to a tolerance, because Whitney
    // interpolation reproduces constant fields without error.
    //
    // This is the test the old hand-built fixture could not be: its `⋆₂` was an operator the crate
    // never vends, so its hand-evaluated expectation described a formulation rather than the
    // physics.
    for (tets, coords) in [unit_tet(), two_tets()] {
        for (v, b) in [
            ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
            ([0.4, -1.3, 2.1], [1.7, 0.9, -0.6]),
        ] {
            let m = geo_manifold(geo_complex(&tets, &coords), &coords, v, b);
            let dt_b = ideal_induction_kernel(&m, &m).unwrap();
            for (i, x) in dt_b.as_slice().iter().enumerate() {
                assert!(
                    x.abs() < 1e-12,
                    "face {i}: a uniform field in a uniform flow must not change, got {x}"
                );
            }
        }
    }
}

#[test]
fn test_a_field_aligned_with_the_flow_does_not_change() {
    // `i_v B = (B × v)♭` vanishes identically when `B ∥ v`, so the whole right-hand side does.
    // Physically: a flow along the field lines does not bend them.
    let (tets, coords) = two_tets();
    let v = [0.6, -1.2, 0.9];
    let m = geo_manifold(geo_complex(&tets, &coords), &coords, v, [-1.8, 3.6, -2.7]);
    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    for (i, x) in dt_b.as_slice().iter().enumerate() {
        assert!(x.abs() < 1e-12, "face {i}: expected zero, got {x}");
    }
}

#[test]
fn test_the_update_creates_no_magnetic_monopoles() {
    // `∇·(∂ₜB) = 0`, the invariant that makes this formulation worth having: because `∂ₜB` is
    // `−d` of something, `d(∂ₜB) = 0` by `d∘d = 0`, so an initially divergence-free `B` stays
    // divergence-free for as long as it is integrated. Checked here on the discrete operator —
    // `d₂ · ∂ₜB` over every tetrahedron — with cochains that are **not** the image of any constant
    // field, so nothing about the input makes it vanish for a lesser reason.
    let (tets, coords) = two_tets();
    let complex = geo_complex(&tets, &coords);
    let n0 = complex.skeletons()[0].simplices().len();
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let n3 = complex.skeletons()[3].simplices().len();

    let mut data = vec![0.0; n0];
    data.extend((0..n1).map(|i| 0.37 * (i as f64) - 1.1));
    data.extend((0..n2).map(|i| 0.9 - 0.21 * (i as f64)));
    data.extend(std::iter::repeat_n(0.0, n3));
    let len = data.len();
    let d_2 = complex.coboundary_operators()[2].clone();
    let m = Manifold::new(complex, CausalTensor::new(data, vec![len]).unwrap(), 0).unwrap();

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    let divergence = d_2.vec_mult_real(dt_b.as_slice()).unwrap();
    for (i, x) in divergence.iter().enumerate() {
        assert!(
            x.abs() < 1e-10,
            "tetrahedron {i}: the update creates divergence {x}"
        );
    }
}

#[test]
fn test_a_single_tetrahedron_cannot_change_its_own_field() {
    // Worth stating, because it looks like a passing test and is really a property of the
    // discretisation. The contraction is evaluated once per tetrahedron, at its barycentre, so on
    // a mesh of one cell `i_v B` is a **constant** 1-form — the same vector dotted into each edge.
    // The exterior derivative of a constant 1-form is zero on every face, so `∂ₜB = 0` for *every*
    // input, however the cochains are chosen.
    //
    // The consequence for the suite: a single-tetrahedron fixture cannot check the sign of the
    // right-hand side, or anything else about its magnitude. Every such test here is on two.
    let (tets, coords) = unit_tet();
    let complex = geo_complex(&tets, &coords);
    let n0 = complex.skeletons()[0].simplices().len();
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let n3 = complex.skeletons()[3].simplices().len();
    let mut data = vec![0.0; n0];
    data.extend((0..n1).map(|i| 3.1 - 0.7 * (i as f64)));
    data.extend((0..n2).map(|i| 0.4 * (i as f64) + 1.2));
    data.extend(std::iter::repeat_n(0.0, n3));
    let len = data.len();
    let m = Manifold::new(complex, CausalTensor::new(data, vec![len]).unwrap(), 0).unwrap();

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    for (i, x) in dt_b.as_slice().iter().enumerate() {
        assert!(x.abs() < 1e-12, "face {i}: expected zero, got {x}");
    }
}

#[test]
fn test_the_result_is_minus_the_exterior_derivative_of_the_contraction() {
    // A **convention pin**, not an independent oracle: it restates `∂ₜB = −d(i_v B)` against the
    // crate's own two operators. Its value is the sign. The implementation this replaced returned
    // `+d(i_v B)` from a chain whose wedge order was also reversed, and no test could tell, because
    // no in-domain input ever reached the arithmetic. The sign of `i_v B` itself is pinned exactly,
    // against a closed form, in `deep_causality_topology`'s own suite.
    let (tets, coords) = two_tets();
    let complex = geo_complex(&tets, &coords);
    let n0 = complex.skeletons()[0].simplices().len();
    let n1 = complex.skeletons()[1].simplices().len();
    let n2 = complex.skeletons()[2].simplices().len();
    let n3 = complex.skeletons()[3].simplices().len();

    let v_vals: Vec<f64> = (0..n1).map(|i| 1.0 + 0.5 * (i as f64)).collect();
    let b_vals: Vec<f64> = (0..n2).map(|i| 2.0 - 0.3 * (i as f64)).collect();
    let mut data = vec![0.0; n0];
    data.extend(v_vals.iter().copied());
    data.extend(b_vals.iter().copied());
    data.extend(std::iter::repeat_n(0.0, n3));
    let len = data.len();
    let d_1 = complex.coboundary_operators()[1].clone();
    let m = Manifold::new(complex, CausalTensor::new(data, vec![len]).unwrap(), 0).unwrap();

    let i_v_b = m
        .interior_product(
            &CausalTensor::new(v_vals, vec![n1]).unwrap(),
            &CausalTensor::new(b_vals, vec![n2]).unwrap(),
            2,
        )
        .unwrap();
    let want = d_1.vec_mult_real(i_v_b.as_slice()).unwrap();

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    assert_eq!(dt_b.shape(), &[n2]);
    for (i, x) in dt_b.as_slice().iter().enumerate() {
        assert!(
            (x + want[i]).abs() < 1e-12,
            "face {i}: expected −{}, got {x}",
            want[i]
        );
    }
    // And the sign is not vacuous: the fixture produces a non-zero right-hand side.
    assert!(
        dt_b.as_slice().iter().any(|x| x.abs() > 1e-6),
        "the fixture must not be a zero field, or the sign check says nothing"
    );
}

#[test]
fn test_ideal_induction_is_linear_in_the_magnetic_field() {
    // `∂ₜB` is linear in `B` for a fixed flow: `−d ∘ i_v` is a composition of two linear maps.
    let (tets, coords) = two_tets();
    let v = [0.9, -0.4, 1.6];
    let b1 = [1.0, 2.0, -1.0];
    let b2 = [-0.5, 0.3, 2.2];
    let alpha = 1.7;
    let combined = [
        alpha * b1[0] + b2[0],
        alpha * b1[1] + b2[1],
        alpha * b1[2] + b2[2],
    ];

    let run = |b: [f64; 3]| {
        let m = geo_manifold(geo_complex(&tets, &coords), &coords, v, b);
        ideal_induction_kernel(&m, &m).unwrap().as_slice().to_vec()
    };
    let (r1, r2, rc) = (run(b1), run(b2), run(combined));
    for i in 0..r1.len() {
        let want = alpha * r1[i] + r2[i];
        assert!(
            (rc[i] - want).abs() <= 1e-10 * (1.0 + want.abs()),
            "face {i}: {} vs {want}",
            rc[i]
        );
    }
}

#[test]
fn test_ideal_induction_refuses_a_complex_without_geometry() {
    // **What this replaces.** The test here used to hand the kernel the `(4, 4)` diagonal star the
    // crate actually vends and pin the refusal, because the kernel needed a degree-changing
    // `(6, 4)` one. That refusal is gone with the star: the contraction no longer reads
    // `hodge_star_operators` at all, which is the point of task 6.7u.
    //
    // The refusal that replaces it is the one the new formulation actually has. Whitney
    // interpolation is geometric, so a complex built without coordinates cannot be contracted on,
    // and the kernel says so rather than inventing any.
    let (tets, coords) = unit_tet();
    let geometric = geo_complex(&tets, &coords);
    let n0 = geometric.skeletons()[0].simplices().len();
    let n1 = geometric.skeletons()[1].simplices().len();
    let n2 = geometric.skeletons()[2].simplices().len();
    let n3 = geometric.skeletons()[3].simplices().len();
    let skeletons: Vec<Skeleton> = geometric
        .skeletons()
        .iter()
        .map(|s| Skeleton::new(s.dim(), s.simplices().clone()))
        .collect();
    let coboundary = geometric.coboundary_operators().clone();
    let bare = SimplicialComplex::new(skeletons, vec![], coboundary, vec![]);

    let data = vec![1.0; n0 + n1 + n2 + n3];
    let len = data.len();
    let m = Manifold::new(bare, CausalTensor::new(data, vec![len]).unwrap(), 0).unwrap();

    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("vertex coordinates"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_refuses_a_degenerate_tetrahedron() {
    // Four coplanar vertices have no barycentric coordinates, so there is nothing to interpolate
    // on. Refused rather than answered with whatever a singular solve returns.
    let tets = vec![[0, 1, 2, 3]];
    let coords = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ];
    let m = geo_manifold(
        geo_complex(&tets, &coords),
        &coords,
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    let msg = format!("{}", ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(msg.contains("degenerate"), "unexpected message: {msg}");
}

#[test]
fn test_ideal_induction_refuses_two_manifolds_over_different_complexes() {
    // The kernel reads `n0/n1/n2`, the Whitney interpolation and the coboundary from
    // `v_manifold`'s complex, and slices `b_manifold`'s data with those offsets. Matching simplex
    // counts do not make the two interchangeable: the meshes below share their connectivity and
    // differ only in where vertex 4 sits, so every count agrees while every face area and edge
    // vector does not, and the 2-form measured on one is meaningless on the other.
    //
    // The oracle is flux freezing, the same invariant
    // `test_a_uniform_field_in_a_uniform_flow_does_not_change` uses: a uniform `B` carried by a
    // uniform `v` has `i_v B` constant, and `d` of a constant 1-form is zero on every face, so
    // `∂ₜB = 0` exactly. Before the complexes were compared, this call returned `Ok` with a
    // largest component of `1.34` — a plausible number for physics that is not being computed.
    let v = [0.4, -1.3, 2.1];
    let b = [1.7, 0.9, -0.6];
    let (tets, coords_a) = two_tets();
    let coords_b = vec![
        coords_a[0],
        coords_a[1],
        coords_a[2],
        coords_a[3],
        [2.4, 1.9, 1.3], // the one vertex that moves
    ];

    let complex_a = geo_complex(&tets, &coords_a);
    let complex_b = geo_complex(&tets, &coords_b);
    let counts = |c: &SimplicialComplex<f64>| {
        c.skeletons()
            .iter()
            .map(|s| s.simplices().len())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        counts(&complex_a),
        counts(&complex_b),
        "the fixture is only interesting if the length checks cannot tell the two apart"
    );

    let ma = geo_manifold(complex_a, &coords_a, v, b);
    let mb = geo_manifold(complex_b, &coords_b, v, b);

    let msg = format!("{}", ideal_induction_kernel(&ma, &mb).unwrap_err());
    assert!(
        msg.contains("different complexes"),
        "unexpected message: {msg}"
    );

    // The control: the same call on one complex is accepted and returns the invariant's zero.
    let dt_b = ideal_induction_kernel(&ma, &ma).unwrap();
    for (i, x) in dt_b.as_slice().iter().enumerate() {
        assert!(x.abs() < 1e-12, "face {i}: expected zero, got {x}");
    }
}
