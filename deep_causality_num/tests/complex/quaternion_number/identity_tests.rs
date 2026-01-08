/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;
use deep_causality_num::{ConstOne, ConstZero, One, Zero};

#[test]
fn test_zero() {
    let q = Quaternion::<f64>::zero();
    assert_eq!(q, Quaternion::new(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_is_zero() {
    let q1 = Quaternion::new(0.0, 0.0, 0.0, 0.0);
    assert!(q1.is_zero());
    let q2 = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    assert!(!q2.is_zero());
}

#[test]
fn test_const_zero() {
    let q = Quaternion::<f64>::ZERO;
    assert_eq!(q, Quaternion::new(0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_one() {
    let q = Quaternion::<f64>::one();
    assert_eq!(q, Quaternion::new(1.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_is_one() {
    let q1 = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    assert!(q1.is_one());
    let q2 = Quaternion::new(1.0, 1.0, 0.0, 0.0);
    assert!(!q2.is_one());
}

#[test]
fn test_const_one() {
    let q = Quaternion::<f64>::ONE;
    assert_eq!(q, Quaternion::new(1.0, 0.0, 0.0, 0.0));
}
