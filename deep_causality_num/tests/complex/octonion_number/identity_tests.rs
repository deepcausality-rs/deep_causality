/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{ConstOne, ConstZero, Octonion, One, Zero};

#[test]
fn test_octonion_zero() {
    let o = Octonion::<f64>::zero();
    assert_eq!(o.s, 0.0);
    assert_eq!(o.e1, 0.0);
    assert!(o.is_zero());
}

#[test]
fn test_octonion_is_zero() {
    let o1 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(o1.is_zero());
    let o2 = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(!o2.is_zero());
    let o3 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(!o3.is_zero());
}

#[test]
fn test_octonion_const_zero() {
    let o = Octonion::<f64>::ZERO;
    assert_eq!(o, Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}

#[test]
fn test_octonion_one() {
    let o = Octonion::<f64>::one();
    assert_eq!(o.s, 1.0);
    assert_eq!(o.e1, 0.0);
    assert!(o.is_one());
}

#[test]
fn test_octonion_is_one() {
    let o1 = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(o1.is_one());
    let o2 = Octonion::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert!(!o2.is_one());
}

#[test]
fn test_octonion_const_one() {
    let o = Octonion::<f64>::ONE;
    assert_eq!(o, Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
}
