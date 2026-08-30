/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator impls that carry `DenseVector` into the tower.
//!
//! These are what `Ring` and `Module<R>` read. The compile-time pins say the memberships hold; these
//! say the operations behind them compute the right thing.

use deep_causality_linear::DenseVector;
use deep_causality_num::Zero;

#[test]
fn test_zero_is_empty_and_recognises_itself() {
    let z: DenseVector<f64> = DenseVector::zero();
    assert!(z.is_zero());
    assert_eq!(z.len(), 0);
    assert!(DenseVector::from_vec(vec![0.0, 0.0]).is_zero());
    assert!(!DenseVector::from_vec(vec![0.0, 1.0]).is_zero());
}

#[test]
fn test_the_add_operator() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![10.0, 20.0]);
    assert_eq!((a + b).as_slice(), &[11.0, 22.0]);
}

#[test]
fn test_the_sub_operator() {
    let a = DenseVector::from_vec(vec![10.0, 20.0]);
    let b = DenseVector::from_vec(vec![1.0, 2.0]);
    assert_eq!((a - b).as_slice(), &[9.0, 18.0]);
}

#[test]
fn test_the_neg_operator() {
    let a = DenseVector::from_vec(vec![1.0, -2.0]);
    assert_eq!((-a).as_slice(), &[-1.0, 2.0]);
}

#[test]
fn test_negation_over_the_integers() {
    let a = DenseVector::from_vec(vec![3i64, -4]);
    assert_eq!((-a).as_slice(), &[-3, 4]);
}

#[test]
fn test_scalar_multiplication_is_what_module_reads() {
    let a = DenseVector::from_vec(vec![1.0, -2.0]);
    assert_eq!((a * 3.0).as_slice(), &[3.0, -6.0]);
    let b = DenseVector::from_vec(vec![1i64, -2]);
    assert_eq!((b * 3i64).as_slice(), &[3, -6]);
}

#[test]
fn test_scalar_multiply_assign() {
    let mut a = DenseVector::from_vec(vec![1.0, -2.0]);
    a *= 4.0;
    assert_eq!(a.as_slice(), &[4.0, -8.0]);
}

#[test]
fn test_scaling_by_zero_annihilates() {
    let a = DenseVector::from_vec(vec![7.0, -9.0]);
    assert!((a * 0.0).is_zero());
}

#[test]
#[should_panic(expected = "length mismatch in add")]
fn test_the_add_operator_rejects_a_length_mismatch() {
    let a = DenseVector::from_vec(vec![1.0]);
    let b = DenseVector::from_vec(vec![1.0, 2.0]);
    let _ = a + b;
}

#[test]
#[should_panic(expected = "length mismatch in sub")]
fn test_the_sub_operator_rejects_a_length_mismatch() {
    let a = DenseVector::from_vec(vec![1.0]);
    let b = DenseVector::from_vec(vec![1.0, 2.0]);
    let _ = a - b;
}

#[test]
fn test_the_additive_group_laws_on_worked_values() {
    let a = DenseVector::from_vec(vec![1.0, 2.0]);
    let b = DenseVector::from_vec(vec![3.0, 5.0]);
    let c = DenseVector::from_vec(vec![7.0, 11.0]);
    // associativity
    assert_eq!(
        ((a.clone() + b.clone()) + c.clone()).as_slice(),
        (a.clone() + (b.clone() + c)).as_slice()
    );
    // commutativity
    assert_eq!(
        (a.clone() + b.clone()).as_slice(),
        (b.clone() + a.clone()).as_slice()
    );
    // inverse
    assert!((a.clone() + (-a)).is_zero());
}
