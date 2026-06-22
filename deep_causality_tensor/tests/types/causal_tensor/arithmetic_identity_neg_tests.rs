/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{One, Zero};
use deep_causality_tensor::CausalTensor;

// --- identity/mod.rs: Zero / One trait impls ---

#[test]
fn test_zero_trait_impl() {
    let z: CausalTensor<f64> = Zero::zero();
    // Scalar zero tensor has empty shape but holds one element.
    assert_eq!(z.shape(), &[] as &[usize]);
    assert_eq!(z.as_slice(), &[0.0]);
}

#[test]
fn test_zero_is_zero_true() {
    let t = CausalTensor::new(vec![0.0, 0.0, 0.0], vec![3]).unwrap();
    assert!(t.is_zero());
}

#[test]
fn test_zero_is_zero_false() {
    let t = CausalTensor::new(vec![0.0, 1.0, 0.0], vec![3]).unwrap();
    assert!(!t.is_zero());
}

#[test]
fn test_one_trait_impl() {
    let o: CausalTensor<f64> = One::one();
    assert_eq!(o.shape(), &[] as &[usize]);
    assert_eq!(o.as_slice(), &[1.0]);
}

#[test]
fn test_one_is_one_true() {
    let t = CausalTensor::new(vec![1.0, 1.0], vec![2]).unwrap();
    assert!(t.is_one());
}

#[test]
fn test_one_is_one_false() {
    let t = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    assert!(!t.is_one());
}

// --- neg/mod.rs: Neg for owned and reference ---

#[test]
fn test_neg_owned() {
    let t = CausalTensor::new(vec![1.0, -2.0, 3.0], vec![3]).unwrap();
    let n = -t;
    assert_eq!(n.as_slice(), &[-1.0, 2.0, -3.0]);
    assert_eq!(n.shape(), &[3]);
}

#[test]
fn test_neg_reference() {
    let t = CausalTensor::new(vec![4.0, -5.0], vec![2]).unwrap();
    let n = -&t;
    assert_eq!(n.as_slice(), &[-4.0, 5.0]);
    // Original is preserved because we negated a reference.
    assert_eq!(t.as_slice(), &[4.0, -5.0]);
}

#[test]
fn test_neg_owned_2d() {
    let t = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let n = -t;
    assert_eq!(n.as_slice(), &[-1.0, -2.0, -3.0, -4.0]);
    assert_eq!(n.shape(), &[2, 2]);
}
