/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{ConstOne, ConstZero, Gf2, One, Zero};

#[test]
fn test_zero() {
    let z: Gf2 = Zero::zero();
    assert_eq!(z, Gf2::ZERO);
    assert!(z.is_zero());
    assert!(!Gf2::ONE.is_zero());
}

#[test]
fn test_set_zero() {
    let mut a = Gf2::ONE;
    a.set_zero();
    assert!(a.is_zero());
}

#[test]
fn test_const_zero() {
    assert_eq!(<Gf2 as ConstZero>::ZERO, Gf2::ZERO);
}

#[test]
fn test_one() {
    let o: Gf2 = One::one();
    assert_eq!(o, Gf2::ONE);
    assert!(o.is_one());
    assert!(!Gf2::ZERO.is_one());
}

#[test]
fn test_set_one() {
    let mut a = Gf2::ZERO;
    a.set_one();
    assert!(a.is_one());
}

#[test]
fn test_const_one() {
    assert_eq!(<Gf2 as ConstOne>::ONE, Gf2::ONE);
}

#[test]
fn test_zero_and_one_are_distinct() {
    // Non-triviality: an integral domain needs 1 != 0.
    assert_ne!(Gf2::zero(), Gf2::one());
}

#[test]
fn test_from_bool() {
    assert_eq!(Gf2::from(false), Gf2::ZERO);
    assert_eq!(Gf2::from(true), Gf2::ONE);
}

#[test]
fn test_into_bool() {
    assert!(!bool::from(Gf2::ZERO));
    assert!(bool::from(Gf2::ONE));
}
