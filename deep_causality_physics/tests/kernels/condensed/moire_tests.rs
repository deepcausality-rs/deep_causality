/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Displacement, Energy, Momentum, Ratio, Speed, Stiffness, TwistAngle,
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
    assert!(res.is_err());
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
    assert!(res.is_err());
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
    assert!(res.is_err());
}

#[test]
fn test_foppl_von_karman_strain_full() {
    let u_man = create_flat_manifold();
    let w_man = create_flat_manifold();
    let e = Stiffness::<f64>::new(100.0).unwrap();
    let nu = Ratio::new(0.3).unwrap();

    let res = foppl_von_karman_strain_kernel(&u_man, &w_man, e, nu);

    // Expect success even if result is zero (flat manifold, zero data)
    assert!(res.is_ok());
    let sigma = res.unwrap();
    // Should be zero tensor
    for val in sigma.data() {
        assert!((val - 0.0).abs() < 1e-10);
    }
}
