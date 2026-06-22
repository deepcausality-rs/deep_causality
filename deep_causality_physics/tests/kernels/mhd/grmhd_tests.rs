/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::{EastCoastMetric, LorentzianMetric};
use deep_causality_physics::{energy_momentum_tensor_em_kernel, relativistic_current_kernel};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry};

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
    let e = 1.0_f64;
    let f_data = vec![0.0, e, -e, 0.0];
    let em: CausalTensor<f64> = CausalTensor::new(f_data, vec![2, 2]).unwrap();

    // Metric diag(-1, 1) (Spacelike convention to get positive energy with standard formula)
    let g_data = vec![-1.0_f64, 0.0, 0.0, 1.0];
    let metric: CausalTensor<f64> = CausalTensor::new(g_data, vec![2, 2]).unwrap();

    let res = energy_momentum_tensor_em_kernel(&em, &metric);
    assert!(res.is_ok());

    let t = res.unwrap();
    // T00 = 0.5 * E^2
    let t00 = t.data()[0];
    assert!((t00 - 0.5).abs() < 1e-10);
}

#[test]
fn test_relativistic_current_kernel_low_dim_metric_error() {
    // Build a valid 4D manifold but pass a metric with dimension < 4
    let points_data = vec![
        0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0,
        0.0, 1.0,
    ];
    let point_tensor = CausalTensor::new(points_data, vec![5, 4]).unwrap();
    let cloud = PointCloud::new(point_tensor, CausalTensor::<f64>::zeros(&[5]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();
    let total = complex.total_simplices();
    let manifold = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; total], vec![total]).unwrap(),
        0,
    )
    .unwrap();

    let metric_3d = EastCoastMetric::new_nd(3).unwrap();
    let r = relativistic_current_kernel(&manifold, &metric_3d);
    assert!(r.is_err());
}

#[test]
fn test_relativistic_current_kernel_low_skeleton_error() {
    // 1D point cloud → triangulation produces only 0- and 1-skeletons, no 2-simplices.
    let points = CausalTensor::new(vec![0.0, 1.0, 2.0], vec![3, 1]).unwrap();
    let cloud = PointCloud::new(points, CausalTensor::<f64>::zeros(&[3]), 0).unwrap();
    let complex = cloud.triangulate(1.5).unwrap();
    let total = complex.total_simplices();
    let manifold = Manifold::new(
        complex,
        CausalTensor::new(vec![0.0; total], vec![total]).unwrap(),
        0,
    )
    .unwrap();

    let metric = EastCoastMetric::minkowski_4d();
    let r = relativistic_current_kernel(&manifold, &metric);
    assert!(r.is_err(), "1D complex must fail for relativistic current");
}

#[test]
fn test_relativistic_current_kernel_insufficient_hodge_ops_error() {
    // A 2D triangular complex has skeletons {0,1,2} (len 3, passes the >=3
    // check) and a 4D metric (passes the dimension>=4 check), but only 3 Hodge
    // star operators (dims 0..=2) — fewer than the 4 required — so the kernel
    // hits the "Missing Hodge star operators" guard (grmhd.rs:69-74).
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let cloud = PointCloud::new(points, CausalTensor::<f64>::zeros(&[3]), 0).unwrap();
    let complex = cloud.triangulate(1.1).unwrap();
    let total = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let metric_regge =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    let manifold = Manifold::with_metric(
        complex,
        CausalTensor::new(vec![1.0; total], vec![total]).unwrap(),
        Some(metric_regge),
        0,
    )
    .unwrap();

    // 4D spacetime metric so the dimension check passes; failure must come from
    // the Hodge-operator count, not the metric dimension.
    let spacetime = EastCoastMetric::minkowski_4d();
    let r = relativistic_current_kernel(&manifold, &spacetime);
    assert!(
        r.is_err(),
        "2D complex must lack the 4 Hodge operators needed for a 4D EM 2-form"
    );
}

// NOTE on defensively-unreachable GRMHD branches:
//   * grmhd.rs:77-80 — "Missing coboundary operators: need 3". To reach it the
//     manifold must first pass the `hodge_ops.len() >= 4` check (lines 69-74),
//     which requires max_dim >= 3. A complex with max_dim >= 3 always yields
//     >= 3 coboundary operators (k -> k+1 for k = 0..max_dim), so the < 3
//     branch can never fire.
//   * grmhd.rs:91-93 — "Manifold data too short for 2-form extraction".
//     `Manifold` enforces `data().len() == total_simplices >= n0 + n1 + n2` at
//     construction, so the data slab is never shorter than the 2-form domain.
//   * grmhd.rs:206 (the `|| (len == 1 && [0] == 1)` operand) and 210-212
//     (the "Scalar contraction failed" else-arm): the double-axis contraction
//     of two rank-2 tensors always yields a scalar whose shape `is_empty()` is
//     true, short-circuiting the `||` and never taking the else. Covered scalar
//     path is exercised by `test_energy_momentum_tensor`.

#[test]
fn test_energy_momentum_tensor_dimension_error() {
    let em = CausalTensor::new(vec![0.0; 4], vec![4]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();
    assert!(energy_momentum_tensor_em_kernel(&em, &metric).is_err());
}
