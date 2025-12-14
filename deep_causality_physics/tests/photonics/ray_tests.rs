/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AbcdMatrix, IndexOfRefraction, RayAngle, RayHeight, lens_maker_kernel, ray_transfer_kernel,
    snells_law_kernel,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_ray_transfer() {
    // Identity matrix
    let m_data = vec![1.0, 0.0, 0.0, 1.0];
    let tensor = CausalTensor::new(m_data, vec![2, 2]).unwrap();
    let matrix = AbcdMatrix::new(tensor);

    let h = RayHeight::new(1.0).unwrap();
    let a = RayAngle::new(0.1).unwrap();

    let res = ray_transfer_kernel(&matrix, h, a);
    assert!(res.is_ok());
    let (h_out, a_out) = res.unwrap();
    assert_eq!(h_out.value(), 1.0);
    assert_eq!(a_out.value(), 0.1);
}

#[test]
fn test_snells_law() {
    let n1 = IndexOfRefraction::new(1.0).unwrap();
    let n2 = IndexOfRefraction::new(1.5).unwrap();
    let theta1 = RayAngle::new(0.5).unwrap(); // Radians

    let res = snells_law_kernel(n1, n2, theta1);
    assert!(res.is_ok());
    let theta2 = res.unwrap();

    // sin(t2) = (1/1.5) * sin(0.5)
    let expected = ((1.0 / 1.5) * 0.5f64.sin()).asin();
    assert!((theta2.value() - expected).abs() < 1e-10);
}

#[test]
fn test_snells_law_tir() {
    let n1 = IndexOfRefraction::new(1.5).unwrap();
    let n2 = IndexOfRefraction::new(1.0).unwrap();
    let theta1 = RayAngle::new(1.0).unwrap(); // Large angle

    // sin(t2) = 1.5 * sin(1.0) = 1.5 * 0.84 = 1.26 > 1
    let res = snells_law_kernel(n1, n2, theta1);
    assert!(res.is_err());
}

#[test]
fn test_lens_maker() {
    let n = IndexOfRefraction::new(1.5).unwrap();
    // Biconvex: R1 > 0, R2 < 0.
    // Spec kernel takes r1: Length, r2: Length? No, I updated kernel to take f64 for signed radii.
    // Wait, let's check what I implemented.
    // I implemented: fn lens_maker_kernel(n: IndexOfRefraction, r1_signed: f64, r2_signed: f64)

    let r1 = 0.5;
    let r2 = -0.5;

    let res = lens_maker_kernel(n, r1, r2);
    assert!(res.is_ok());
    let p = res.unwrap();

    // P = (1.5 - 1) * (1/0.5 - 1/-0.5) = 0.5 * (2 - (-2)) = 0.5 * 4 = 2.0
    assert!((p.value() - 2.0).abs() < 1e-10);
}
