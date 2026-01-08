/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Diffusivity, magnetic_reconnection_rate_kernel, resistive_diffusion_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

fn create_dummy_manifold() -> Manifold<f64, f64> {
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
fn test_resistive_diffusion() {
    let m = create_dummy_manifold();
    let eta = Diffusivity::new(0.1).unwrap();

    // Should run, result depends on laplacian of zero field -> zero
    let res = resistive_diffusion_kernel(&m, eta);
    assert!(res.is_ok());
}

#[test]
fn test_reconnection_rate() {
    let va = AlfvenSpeed::new(100.0).unwrap();
    let s = 100.0; // Lundquist

    let res = magnetic_reconnection_rate_kernel(va, s);
    assert!(res.is_ok());
    // vin = va / sqrt(S) = 100 / 10 = 10
    assert!((res.unwrap().value() - 10.0).abs() < 1e-10);
}
