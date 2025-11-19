/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{DixonAlgebra, Metric, PGA3DMultiVector, RealMultiVector};
use deep_causality_num::Complex;

#[test]
fn test_pga3d_new_point() {
    // Point (x, y, z) -> x*e032 + y*e013 + z*e021 + e123
    let p = PGA3DMultiVector::new_point(1.0, 2.0, 3.0);

    // Check metric
    assert_eq!(p.metric.dimension(), 4);

    // Check coefficients
    // e123 (14) -> 1.0
    assert_eq!(p.data[14], 1.0);
    // e032 (13) -> 1.0
    assert_eq!(p.data[13], 1.0);
    // e013 (11) -> 2.0
    assert_eq!(p.data[11], 2.0);
    // e021 (7) -> 3.0
    assert_eq!(p.data[7], 3.0);
}

#[test]
fn test_pga3d_translator() {
    // Translator T = 1 - 0.5(x*e01 + y*e02 + z*e03)
    let t = PGA3DMultiVector::translator(2.0, 4.0, 6.0);

    // Scalar (0) -> 1.0
    assert_eq!(t.data[0], 1.0);

    // e01 (3) -> -0.5 * 2.0 = -1.0
    assert_eq!(t.data[3], -1.0);

    // e02 (5) -> -0.5 * 4.0 = -2.0
    assert_eq!(t.data[5], -2.0);

    // e03 (9) -> -0.5 * 6.0 = -3.0
    assert_eq!(t.data[9], -3.0);
}

#[test]
fn test_real_complex_number() {
    let c = RealMultiVector::new_complex_number(1.0, 2.0);
    // Metric: NonEuclidean(1) -> e1^2 = -1
    assert_eq!(c.metric, Metric::NonEuclidean(1));
    assert_eq!(c.data, vec![1.0, 2.0]);
}

#[test]
fn test_real_quaternion() {
    let q = RealMultiVector::new_quaternion(1.0, 2.0, 3.0, 4.0);
    // Metric: NonEuclidean(2) -> e1^2 = -1, e2^2 = -1
    assert_eq!(q.metric, Metric::NonEuclidean(2));
    assert_eq!(q.data, vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_real_split_complex() {
    let sc = RealMultiVector::new_split_complex(1.0, 2.0);
    // Metric: Euclidean(1) -> e1^2 = +1
    assert_eq!(sc.metric, Metric::Euclidean(1));
    assert_eq!(sc.data, vec![1.0, 2.0]);
}

#[test]
fn test_real_aps_vector() {
    let data = vec![0.0; 8];
    let aps = RealMultiVector::new_aps_vector(data.clone());
    assert_eq!(aps.metric, Metric::Euclidean(3));
    assert_eq!(aps.data.len(), 8);
}

#[test]
fn test_real_spacetime_vector() {
    let data = vec![0.0; 16];
    let sta = RealMultiVector::new_spacetime_vector(data.clone());
    assert_eq!(sta.metric, Metric::Minkowski(4));
    assert_eq!(sta.data.len(), 16);
}

#[test]
fn test_real_cga_vector() {
    let data = vec![0.0; 32];
    let cga = RealMultiVector::new_cga_vector(data.clone());

    // Metric: Custom { dim: 5, neg_mask: 16, zero_mask: 0 }
    match cga.metric {
        Metric::Custom {
            dim,
            neg_mask,
            zero_mask,
        } => {
            assert_eq!(dim, 5);
            assert_eq!(neg_mask, 16);
            assert_eq!(zero_mask, 0);
        }
        _ => panic!("Expected Custom metric for CGA"),
    }
    assert_eq!(cga.data.len(), 32);
}

#[test]
fn test_dixon_algebra() {
    // Dixon Algebra is Cl_C(6) -> 64 dimensions (complex)
    let data = vec![Complex::new(0.0, 0.0); 64];
    let d = DixonAlgebra::new_dixon_algebra(data);

    assert_eq!(d.metric.dimension(), 6);
    match d.metric {
        Metric::Euclidean(6) => {}
        _ => panic!("Dixon Algebra should be Euclidean(6)"),
    }
}
