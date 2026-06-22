/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Diffusivity, magnetic_reconnection_rate_kernel, resistive_diffusion_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

fn create_dummy_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_resistive_diffusion() {
    let m = create_dummy_manifold();
    let eta = Diffusivity::<f64>::new(0.1).unwrap();

    // Should run, result depends on laplacian of zero field -> zero
    let res = resistive_diffusion_kernel(&m, eta);
    assert!(res.is_ok());
}

#[test]
fn test_resistive_diffusion_negative_diffusivity_error() {
    // Diffusivity::new rejects negatives, so use new_unchecked to feed a
    // negative eta straight into the kernel and trip its PhysicalInvariantBroken
    // guard (resistive.rs:25-29).
    let m = create_dummy_manifold();
    let eta = Diffusivity::<f64>::new_unchecked(-0.5);
    let res = resistive_diffusion_kernel(&m, eta);
    assert!(res.is_err());
}

#[test]
fn test_reconnection_rate_non_positive_lundquist_error() {
    // lundquist <= 0 -> Singularity (resistive.rs:51-55).
    let va = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert!(magnetic_reconnection_rate_kernel(va, 0.0).is_err());
    let va2 = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert!(magnetic_reconnection_rate_kernel(va2, -1.0).is_err());
}

#[test]
fn test_reconnection_rate() {
    let va = AlfvenSpeed::<f64>::new(100.0).unwrap();
    let s = 100.0; // Lundquist

    let res = magnetic_reconnection_rate_kernel(va, s);
    assert!(res.is_ok());
    // vin = va / sqrt(S) = 100 / 10 = 10
    assert!((res.unwrap().value() - 10.0).abs() < 1e-10);
}
