/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, klein_gordon_kernel};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

// Helper to create a simple manifold for Klein-Gordon test with a unit-edge
// metric attached (required by R4.5: `klein_gordon_kernel` calls
// `manifold.laplacian()` which now requires a metric).
fn create_simple_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let initial_data = vec![1.0; num_simplices];
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(initial_data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

// =============================================================================
// Klein-Gordon Kernel Tests
// =============================================================================

// NOTE: The klein_gordon_kernel has a known shape mismatch issue:
// - laplacian(0) returns tensor of shape [num_vertices]
// - psi_manifold.data() returns tensor of shape [total_simplices]
// When the manifold has edges/faces, these shapes differ causing panic on addition.
// This test documents the current behavior.

#[test]
fn test_klein_gordon_kernel_valid() {
    // This manifold has vertices + edges + faces.
    // Previously caused shape mismatch, now should work by slicing.
    let manifold = create_simple_manifold();
    let mass = 1.0;

    let result = klein_gordon_kernel(&manifold, mass);
    assert!(
        result.is_ok(),
        "Klein-Gordon kernel failed: {:?}",
        result.err()
    );
}

#[test]
fn test_klein_gordon_kernel_nan_mass() {
    let manifold = create_simple_manifold();
    let mass = f64::NAN;
    let result = klein_gordon_kernel(&manifold, mass);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

#[test]
fn test_klein_gordon_kernel_inf_mass() {
    let manifold = create_simple_manifold();
    let mass = f64::INFINITY;
    let result = klein_gordon_kernel(&manifold, mass);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

// Builds a triangular manifold whose stored field data is supplied by the
// caller, so non-finite or oversized vertex values can be injected to
// exercise the Klein-Gordon finiteness guards.
fn create_manifold_with_data(data: Vec<f64>) -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num_simplices = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    assert_eq!(data.len(), num_simplices);
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![num_simplices]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_klein_gordon_kernel_nonfinite_laplacian() {
    // NaN vertex data propagates through the exterior derivative into the
    // Hodge-Laplacian, tripping the "Laplacian contains non-finite entries"
    // guard.
    let manifold = create_manifold_with_data(vec![f64::NAN; 7]);
    let result = klein_gordon_kernel(&manifold, 1.0);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

#[test]
fn test_klein_gordon_kernel_m2_psi_overflow() {
    // Finite-but-huge uniform vertex data combined with a huge (finite) mass
    // makes m^2 * psi overflow to +inf, tripping the "m^2 * psi produced
    // non-finite entries" guard. A uniform field yields a finite (~0)
    // laplacian, so the earlier laplacian guard does not fire, and
    // m^2 = (1e60)^2 = 1e120 is finite, so the m^2 guard does not fire either.
    // 1e120 * 1e200 = 1e320 = +inf (> f64::MAX ~ 1.8e308) trips the guard.
    let manifold = create_manifold_with_data(vec![1e200; 7]);
    let result = klein_gordon_kernel(&manifold, 1e60);
    assert!(
        matches!(
            result.as_ref().unwrap_err().0,
            PhysicsErrorEnum::NumericalInstability { .. }
        ),
        "expected a NumericalInstability refusal"
    );
}

// NOTE on three defensively-unreachable Klein-Gordon guards
// (`klein_gordon_kernel`):
//   * "psi_data is smaller than laplacian data".
//     `vertex_count == laplacian.len()` is the number of 0-simplices (vertices),
//     and `Manifold` stores one datum per simplex, so `psi_data.len() ==
//     total_simplices >= vertex_count` always. The guard never fires.
//   * "psi data contains non-finite entries". To reach this, the *laplacian*
//     guard must first pass. But the Hodge-Laplacian of a 0-form is built from
//     the vertex values; a non-finite vertex value propagates into the
//     laplacian and is caught by the earlier guard, so a finite laplacian
//     implies finite vertex psi. The two conditions are mutually exclusive.
//   * "Klein-Gordon result contains non-finite entries" (`laplacian + m2_psi`).
//     Both summands are already guaranteed finite. The sum overflows to ±inf
//     only if both summands are ~1e308; but a uniform field (which keeps m2_psi
//     large) yields laplacian ≈ 0, and a non-uniform field large enough to push
//     the laplacian near 1e308 overflows the laplacian first. No data
//     configuration makes both summands simultaneously near MAX, so the
//     result-overflow guard is unreachable. An offline scan over magnitudes
//     1e150..1e308 confirmed only the laplacian and m2_psi guards ever fire.

#[test]
fn test_klein_gordon_is_affine_in_the_mass_squared() {
    // KG = laplacian(psi) + m^2 psi. The Laplacian does not depend on the mass, so
    //
    //     KG(m)[i] = KG(0)[i] + m^2 psi[i]
    //
    // vertex by vertex. That pins the square on the mass and the sign of the sum without
    // reimplementing the Hodge Laplacian, which is what left this kernel on `is_ok()` alone.
    //
    // A triangle has three vertices, and the kernel reads the first three slab entries as psi.
    let psi = [1.0_f64, 2.0, 3.0];
    let mut data = vec![0.5f64; 7];
    data[..3].copy_from_slice(&psi);
    let manifold = create_manifold_with_data(data);

    let base = klein_gordon_kernel(&manifold, 0.0).unwrap();
    for m in [0.5_f64, 1.5, 3.0] {
        let kg = klein_gordon_kernel(&manifold, m).unwrap();
        let b: &[f64] = base.as_slice();
        let v: &[f64] = kg.as_slice();
        for i in 0..psi.len() {
            let want = b[i] + m * m * psi[i];
            assert!(
                (v[i] - want).abs() < 1e-12,
                "m = {m}, vertex {i}: KG = {}, expected KG(0) + m^2 psi = {want}",
                v[i]
            );
        }
    }
}

#[test]
fn test_klein_gordon_mass_term_is_quadratic_not_linear() {
    // Doubling the mass must quadruple the mass-dependent part; a term built as m + m only
    // doubles it.
    let psi = [1.0_f64, 2.0, 3.0];
    let mut data = vec![0.5f64; 7];
    data[..3].copy_from_slice(&psi);
    let manifold = create_manifold_with_data(data);

    let k0 = klein_gordon_kernel(&manifold, 0.0).unwrap();
    let k1 = klein_gordon_kernel(&manifold, 1.5).unwrap();
    let k2 = klein_gordon_kernel(&manifold, 3.0).unwrap();

    let a: &[f64] = k0.as_slice();
    let b: &[f64] = k1.as_slice();
    let c: &[f64] = k2.as_slice();
    for i in 0..psi.len() {
        let single = b[i] - a[i];
        let doubled = c[i] - a[i];
        assert!(
            (doubled - 4.0 * single).abs() < 1e-12,
            "vertex {i}: doubling the mass gave {doubled}, expected 4 x {single}"
        );
    }
}
