/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Density, PhysicalField, alfven_speed_kernel, ideal_induction_kernel, magnetic_pressure_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, SimplicialManifold};

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
//   * ideal.rs:134-136 — "v_manifold data too small". `Manifold` enforces
//     `data().len() == total_simplices >= n0 + n1 + n2` at construction, so the
//     data slab is never shorter than n0 + n1 + n2.
//   * ideal.rs:156-158 — "Hodge star operator for 2-forms not available"
//     (`hodge_ops.len() <= 2`). Reaching this requires the earlier
//     `skeletons.len() >= 3` check (line 122) to pass, i.e. max_dim >= 2, which
//     always yields >= 3 Hodge operators (dims 0..=2). The two conditions are
//     mutually exclusive.
//   * ideal.rs:175-177 — "Coboundary operator for 1-forms not available"
//     (`coboundary_operators().len() <= 1`). Same argument: a complex with
//     2-simplices yields >= 2 coboundary operators, so this never fires.
//   * ideal.rs:265-267 — `wedge_product_1form_1form`'s own `skeletons.len() < 3`
//     guard. The only caller (`ideal_induction_kernel`) has already validated
//     `skeletons.len() >= 3` before invoking it.
//   * ideal.rs:287-288 — `verts.len() != 3` for a face. Every 2-simplex (face)
//     of a simplicial complex has exactly 3 vertices, so the zero-push branch
//     is unreachable.
//   * ideal.rs:309-310 — the edge-lookup `else` push-zero. A face [v0,v1,v2]
//     always has its boundary edges (v0,v1) and (v1,v2) present in the complex's
//     edge set, so both lookups succeed and the else is never taken.

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
