/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::CsrMatrix;
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, PhysicalField, PhysicsError, PhysicsErrorEnum, alfven_speed_kernel,
    ideal_induction_kernel, magnetic_pressure_kernel,
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
fn test_ideal_induction() {
    let m = create_dummy_manifold();
    let res = ideal_induction_kernel(&m, &m);
    assert!(
        res.is_ok(),
        "Ideal induction kernel failed: {:?}",
        res.err()
    );
    let tensor = res.unwrap();
    assert!(!tensor.is_empty());
    // For zero inputs (create_dummy_manifold inits with 0), output should be 0.
    for val in tensor.as_slice() {
        assert_eq!(*val, 0.0);
    }
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

fn calculation_error_message(err: PhysicsError) -> String {
    match err.0 {
        PhysicsErrorEnum::CalculationError(msg) => msg,
        other => panic!("expected CalculationError, got {other:?}"),
    }
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

    let msg = calculation_error_message(ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("Hodge star operator for 2-forms"),
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

    let msg = calculation_error_message(ideal_induction_kernel(&m, &m).unwrap_err());
    assert!(
        msg.contains("Coboundary operator for 1-forms"),
        "unexpected message: {msg}"
    );
}

#[test]
fn test_ideal_induction_on_a_single_triangle() {
    // Reference value derived from the documented pipeline
    // ∂ₜB = d(⋆(v ∧ ⋆B)) with the fixture operators:
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

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    assert_eq!(dt_b.shape(), &[1]);
    assert!(
        (dt_b.as_slice()[0] - 1.0).abs() < 1e-12,
        "expected 1.0, got {}",
        dt_b.as_slice()[0]
    );
}

#[test]
fn test_ideal_induction_ignores_a_2_skeleton_entry_that_is_not_a_triangle() {
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

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    assert_eq!(dt_b.as_slice(), &[0.0]);
}

#[test]
fn test_ideal_induction_ignores_a_face_whose_boundary_edge_is_absent() {
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

    let dt_b = ideal_induction_kernel(&m, &m).unwrap();
    assert_eq!(dt_b.as_slice(), &[0.0]);
}
