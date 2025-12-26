/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::energy_momentum_tensor_em_kernel;
use deep_causality_tensor::CausalTensor;

// Note: relativistic_current_kernel now requires Manifold<f64> and LorentzianMetric
// See wrappers_tests.rs for Manifold-based tests

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
