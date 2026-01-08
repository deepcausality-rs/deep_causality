/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_physics::{energy_momentum_tensor_em_kernel, relativistic_current_kernel};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud};

#[test]
fn test_relativistic_current_kernel_4d() {
    // 1. Create 5 points in 4D (Pentatope vertices) working with Euclidean metric for distance
    // P0: (0,0,0,0)
    // P1: (1,0,0,0)
    // P2: (0,1,0,0)
    // P3: (0,0,1,0)
    // P4: (0,0,0,1)
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();

    // 2. Create PointCloud and Triangulate
    // Radius > sqrt(2) ~ 1.414 ensures all unit points connect to each other.
    let cloud = PointCloud::new(point_tensor.clone(), CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();

    // Check we have enough structure
    let skeletons = complex.skeletons();
    assert!(
        skeletons.len() >= 4,
        "Need at least 3-simplices (skeletons 0..3)"
    );

    // 3. Create Manifold with Fake EM Data
    // We need data for 0, 1, and 2-simplices.
    let n0 = skeletons[0].simplices().len();
    let n1 = skeletons[1].simplices().len();
    let n2 = skeletons[2].simplices().len();
    let n3 = skeletons[3].simplices().len();
    let total_simplices = complex.total_simplices();

    let mut data = vec![0.0; total_simplices];

    // inject some "field" into 2-simplices (indices n0 + n1 .. n0 + n1 + n2)
    for i in 0..n2 {
        data[n0 + n1 + i] = (i as f64) * 0.1;
    }

    let manifold = Manifold::new(
        complex,
        CausalTensor::new(data, vec![total_simplices]).unwrap(),
        0,
    )
    .unwrap();

    // 4. Metric
    let metric = EastCoastMetric::minkowski_4d();

    // 5. Run Kernel
    let result = relativistic_current_kernel(&manifold, &metric);
    assert!(
        result.is_ok(),
        "Kernel execution failed: {:?}",
        result.err()
    );

    let j = result.unwrap();
    // J is returned as a dual 1-form on 3-simplices
    assert_eq!(j.shape(), &[n3]);
}

#[test]
fn test_energy_momentum_tensor() {
    // Flat space 2D. F = [[0, E], [-E, 0]].
    let e = 1.0;
    let f_data = vec![0.0, e, -e, 0.0];
    let em = CausalTensor::new(f_data, vec![2, 2]).unwrap();

    // Metric diag(-1, 1) (Spacelike convention to get positive energy with standard formula)
    let g_data = vec![-1.0, 0.0, 0.0, 1.0];
    let metric = CausalTensor::new(g_data, vec![2, 2]).unwrap();

    let res = energy_momentum_tensor_em_kernel(&em, &metric);
    assert!(res.is_ok());

    let t = res.unwrap();
    // T00 = 0.5 * E^2
    let t00 = t.data()[0];
    assert!((t00 - 0.5).abs() < 1e-10);
}

#[test]
fn test_energy_momentum_tensor_dimension_error() {
    let em = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();
    assert!(energy_momentum_tensor_em_kernel(&em, &metric).is_err());
}
