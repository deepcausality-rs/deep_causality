/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Displacement, Energy, Momentum, PhysicsErrorEnum, Ratio, Speed, Stiffness, TwistAngle,
    bistritzer_macdonald_kernel, foppl_von_karman_strain_kernel,
    foppl_von_karman_strain_simple_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, SimplicialManifold};

#[test]
fn test_bistritzer_macdonald_shape() {
    let theta = TwistAngle::from_degrees(1.1);
    let w = Energy::new(0.11).unwrap(); // 110 meV
    let vf = Speed::new(1e6).unwrap(); // 10^6 m/s
    let k = Momentum::new(CausalMultiVector::new(vec![0.0; 8], Metric::Euclidean(3)).unwrap());

    let res = bistritzer_macdonald_kernel(theta, w, vf, k, 1);
    assert!(res.is_ok());
    let ham = res.unwrap();
    assert_eq!(ham.shape(), vec![8, 8]);
}

#[test]
fn test_bistritzer_macdonald_cutoff_error() {
    let theta = TwistAngle::new(0.1).unwrap();
    let w = Energy::new(0.1).unwrap();
    let vf = Speed::new(1e6).unwrap();
    let k = Momentum::default();

    let res = bistritzer_macdonald_kernel(theta, w, vf, k, 2);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::CalculationError { .. }
        ),
        "expected a CalculationError refusal"
    );
}

#[test]
fn test_foppl_von_karman_strain_simple() {
    // Strain eps = diag(1, 1) (2x2)
    let eps_data = vec![1.0, 0.0, 0.0, 1.0];
    let eps_tensor = CausalTensor::new(eps_data, vec![2, 2]).unwrap();
    let disp_u = Displacement::new(eps_tensor);
    // disp_w removed

    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.5).unwrap();

    let res = foppl_von_karman_strain_simple_kernel(&disp_u, e, nu);
    assert!(res.is_ok());

    let sigma = res.unwrap();
    // Expected: sigma = E/(1-nu) * I = 100/0.5 * I = 200 * I
    let data = sigma.data();
    assert!((data[0] - 200.0).abs() < 1e-10);
    assert!((data[3] - 200.0).abs() < 1e-10);
}

/// The triangular manifold with caller-supplied vertex data, so the strain is not identically
/// zero. `create_flat_manifold` fills every simplex with `0.0`, where `du` and `dw` both vanish
/// and any implementation of the strain returns zero.
fn manifold_with_data(data: Vec<f64>) -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    assert_eq!(data.len(), num, "data must match the simplex count");
    Manifold::new(complex, CausalTensor::new(data, vec![num]).unwrap(), 0).unwrap()
}

// Helper for manifold tests
fn create_flat_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    // Data initialized to 0.0
    Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn test_foppl_von_karman_strain_simple_rank_error() {
    // Strain tensor with Rank 1 (not Rank 2) trips the DimensionMismatch
    // guard in foppl_von_karman_strain_simple_kernel (moire.rs:199-201).
    let eps_tensor = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![4]).unwrap();
    let disp_u = Displacement::new(eps_tensor);

    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.5).unwrap();

    let res = foppl_von_karman_strain_simple_kernel(&disp_u, e, nu);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

// Build a manifold from a point cloud with a configurable number of vertices,
// so two manifolds can differ in vertex/edge count and thus produce
// exterior-derivative fields of mismatched shape.
fn create_line_manifold() -> SimplicialManifold<f64, f64> {
    // Two points => 1 edge, fewer simplices than the triangular manifold.
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0], vec![2, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 2], vec![2]).unwrap(), 0).unwrap();
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
fn test_foppl_von_karman_strain_full_shape_mismatch() {
    // u_manifold (triangle, 3 vertices) and w_manifold (line, 2 vertices) produce
    // gradient fields of different shape, tripping the DimensionMismatch guard
    // in foppl_von_karman_strain_kernel (moire.rs:277-279).
    let u_man = create_flat_manifold(); // 3 vertices
    let w_man = create_line_manifold(); // 2 vertices
    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let res = foppl_von_karman_strain_kernel(&u_man, &w_man, e, nu);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_foppl_von_karman_strain_full_vanishes_on_a_flat_manifold() {
    // The flat limit: with zero displacement both `du` and `dw` vanish, so the strain is zero for
    // any implementation. It pins nothing on its own and sits beside the two tests below.
    let u_man = create_flat_manifold();
    let w_man = create_flat_manifold();
    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let sigma = foppl_von_karman_strain_kernel(&u_man, &w_man, e, nu).unwrap();
    for val in sigma.data() {
        assert!((val - 0.0).abs() < 1e-10);
    }
}

#[test]
fn test_foppl_von_karman_strain_full_is_non_zero_for_a_displaced_manifold() {
    let u_man = manifold_with_data(vec![1.0, -2.0, 3.0, 0.0, 0.0, 0.0, 0.0]);
    let w_man = create_flat_manifold();
    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let sigma = foppl_von_karman_strain_kernel(&u_man, &w_man, e, nu).unwrap();
    assert!(
        sigma.data().iter().any(|v: &f64| v.abs() > 1e-9),
        "an in-plane displacement must produce a non-zero stress"
    );
}

#[test]
fn test_foppl_von_karman_out_of_plane_term_is_quadratic_in_the_deflection() {
    // eps = du + (1/2)(dw)^2, so the out-of-plane contribution is quadratic in w while the
    // in-plane one is linear in u. Holding u fixed and doubling w must quadruple the difference
    // from the w = 0 case:
    //
    //     sigma(u, 2w) - sigma(u, 0) = 4 [sigma(u, w) - sigma(u, 0)]
    //
    // A strain that dropped the square, or took `dw` linearly, only doubles it.
    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();
    let u_man = manifold_with_data(vec![0.5, -1.0, 1.5, 0.0, 0.0, 0.0, 0.0]);
    let w = vec![1.0, -2.0, 3.0, 0.0, 0.0, 0.0, 0.0];
    let w2: Vec<f64> = w.iter().map(|v| v * 2.0).collect();

    let flat = foppl_von_karman_strain_kernel(&u_man, &create_flat_manifold(), e, nu).unwrap();
    let single = foppl_von_karman_strain_kernel(&u_man, &manifold_with_data(w), e, nu).unwrap();
    let doubled = foppl_von_karman_strain_kernel(&u_man, &manifold_with_data(w2), e, nu).unwrap();

    let base: &[f64] = flat.data();
    let one: &[f64] = single.data();
    let two: &[f64] = doubled.data();

    let mut saw_a_contribution = false;
    for i in 0..base.len() {
        let from_w = one[i] - base[i];
        let from_2w = two[i] - base[i];
        if from_w.abs() > 1e-9 {
            saw_a_contribution = true;
        }
        assert!(
            (from_2w - 4.0 * from_w).abs() < 1e-9,
            "component {i}: doubling w gave {from_2w}, expected 4 x {from_w}"
        );
    }
    assert!(
        saw_a_contribution,
        "the out-of-plane term must actually contribute, or the relation above is vacuous"
    );
}
