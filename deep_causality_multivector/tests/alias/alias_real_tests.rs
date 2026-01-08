/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{Metric, RealMultiVector};

#[test]
fn test_real_euclidean_algrebra() {
    let e = RealMultiVector::new_euclidean(vec![1.0, 2.0], 1);
    // Metric: NonEuclidean(1) -> e1^2 = -1
    assert_eq!(e.metric(), Metric::Euclidean(1));
    assert_eq!(e.data(), &vec![1.0, 2.0]);
}

#[test]
fn test_real_complex_number() {
    let c = RealMultiVector::new_complex_number(1.0, 2.0);
    // Metric: NonEuclidean(1) -> e1^2 = -1
    assert_eq!(c.metric(), Metric::NonEuclidean(1));
    assert_eq!(c.data(), &vec![1.0, 2.0]);
}

#[test]
fn test_real_split_complex() {
    let sc = RealMultiVector::new_split_complex(1.0, 2.0);
    // Metric: Euclidean(1) -> e1^2 = +1
    assert_eq!(sc.metric(), Metric::Euclidean(1));
    assert_eq!(sc.data(), &vec![1.0, 2.0]);
}

#[test]
fn test_real_quaternion() {
    let q = RealMultiVector::new_quaternion(1.0, 2.0, 3.0, 4.0);
    // Metric: NonEuclidean(2) -> e1^2 = -1, e2^2 = -1
    assert_eq!(q.metric(), Metric::NonEuclidean(2));
    assert_eq!(q.data(), &vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_real_split_quaternion() {
    let q = RealMultiVector::new_split_quaternion(1.0, 2.0, 3.0, 4.0);
    // Metric: NonEuclidean(2) -> e1^2 = -1, e2^2 = -1
    assert_eq!(q.metric(), Metric::Euclidean(2));
    assert_eq!(q.data(), &vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_real_aps_vector() {
    let data = vec![0.0; 8];
    let aps = RealMultiVector::new_aps_vector(data.clone());
    assert_eq!(aps.metric(), Metric::Euclidean(3));
    assert_eq!(aps.data().len(), 8);
}

#[test]
fn test_real_spacetime_algebra_1_3() {
    let data = vec![0.0; 16];
    let sta = RealMultiVector::new_spacetime_algebra_1_3(data.clone());
    assert_eq!(sta.metric(), Metric::Minkowski(4));
    assert_eq!(sta.data().len(), 16);
}

#[test]
fn test_real_spacetime_algebra_3_1() {
    let data = vec![0.0; 16];
    let sta = RealMultiVector::new_spacetime_algebra_3_1(data.clone());
    assert_eq!(
        sta.metric(),
        Metric::Custom {
            dim: 4,
            neg_mask: 1,
            zero_mask: 0,
        }
    );
    assert_eq!(sta.data().len(), 16);
}

#[test]
fn test_real_cga_vector() {
    let data = vec![0.0; 32];
    let cga = RealMultiVector::new_cga_vector(data.clone());

    // Metric (+ + + + -) = Generic { p: 4, q: 1, r: 0 }
    match cga.metric() {
        Metric::Generic { p, q, r } => {
            assert_eq!(p, 4);
            assert_eq!(q, 1);
            assert_eq!(r, 0);
        }
        _ => panic!("Expected Custom metric for CGA"),
    }
    assert_eq!(cga.data().len(), 32);
}
