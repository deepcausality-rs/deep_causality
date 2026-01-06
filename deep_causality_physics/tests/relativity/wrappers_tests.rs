/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    chronometric_volume, einstein_tensor, geodesic_deviation, spacetime_interval,
    time_dilation_angle,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_einstein_tensor_wrapper_success() {
    let ricci = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let scalar_r = 2.0;

    let effect = einstein_tensor(&ricci, scalar_r, &metric);
    assert!(effect.is_ok());
}

#[test]
fn test_geodesic_deviation_wrapper_success() {
    // Riemann tensor [4, 4, 4, 4]
    let riemann = CausalTensor::new(vec![0.0f64; 256], vec![4, 4, 4, 4]).unwrap();
    // Velocity and separation as slices
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let effect = geodesic_deviation(&riemann, &velocity, &separation);
    assert!(effect.is_ok());
}

#[test]
fn test_geodesic_deviation_wrapper_error() {
    // Wrong Riemann rank should propagate error
    let riemann = CausalTensor::new(vec![1.0f64; 4], vec![2, 2]).unwrap(); // Rank 2
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let effect = geodesic_deviation(&riemann, &velocity, &separation);
    assert!(effect.is_err());
}

#[test]
fn test_spacetime_interval_wrapper_success() {
    let mv = CausalMultiVector::new(
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let metric = Metric::Minkowski(4);

    let effect = spacetime_interval(&mv, &metric);
    assert!(effect.is_ok());
}

#[test]
fn test_spacetime_interval_wrapper_metric_mismatch_error() {
    let mv = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let metric = Metric::Minkowski(4);

    let effect = spacetime_interval(&mv, &metric);
    assert!(effect.is_err());
}

#[test]
fn test_time_dilation_angle_wrapper_success() {
    let t1 = CausalMultiVector::new(
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let t2 = CausalMultiVector::new(
        vec![
            0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();

    let effect = time_dilation_angle(&t1, &t2);
    assert!(effect.is_ok());
}

#[test]
fn test_chronometric_volume_wrapper_success() {
    let a = CausalMultiVector::new(
        vec![
            0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();
    let c = CausalMultiVector::new(
        vec![
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
        Metric::Minkowski(4),
    )
    .unwrap();

    let effect = chronometric_volume(&a, &b, &c);
    assert!(effect.is_ok());
}
