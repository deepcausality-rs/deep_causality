/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

fn a() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap()
}

fn b() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0), (1, 0, 4.0)]).unwrap()
}

// ----- Add: 4 ownership variants -----

#[test]
fn test_add_owned_owned() {
    let c = a() + b();
    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), 3.0);
    assert_eq!(c.get_value_at(1, 0), 4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
}

#[test]
fn test_add_ref_ref() {
    let x = a();
    let y = b();
    let c = &x + &y;
    assert_eq!(c.get_value_at(0, 1), 3.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_add_owned_ref() {
    let y = b();
    let c = a() + &y;
    assert_eq!(c.get_value_at(1, 0), 4.0);
}

#[test]
fn test_add_ref_owned() {
    let x = a();
    let c = &x + b();
    assert_eq!(c.get_value_at(1, 1), 2.0);
}

// ----- AddAssign -----

#[test]
fn test_add_assign_owned() {
    let mut x = a();
    x += b();
    assert_eq!(x.get_value_at(0, 1), 3.0);
    assert_eq!(x.get_value_at(1, 0), 4.0);
}

#[test]
fn test_add_assign_ref() {
    let mut x = a();
    let y = b();
    x += &y;
    assert_eq!(x.get_value_at(0, 1), 3.0);
}

// ----- Sub: 4 ownership variants -----

#[test]
fn test_sub_owned_owned() {
    let c = a() - a();
    assert!(c.values().is_empty());
}

#[test]
fn test_sub_ref_ref() {
    let x = a();
    let y = b();
    let c = &x - &y;
    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), -3.0);
    assert_eq!(c.get_value_at(1, 0), -4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
}

#[test]
fn test_sub_owned_ref() {
    let y = b();
    let c = a() - &y;
    assert_eq!(c.get_value_at(0, 1), -3.0);
}

#[test]
fn test_sub_ref_owned() {
    let x = a();
    let c = &x - b();
    assert_eq!(c.get_value_at(1, 0), -4.0);
}

// ----- SubAssign -----

#[test]
fn test_sub_assign_owned() {
    let mut x = a();
    x -= b();
    assert_eq!(x.get_value_at(0, 1), -3.0);
}

#[test]
fn test_sub_assign_ref() {
    let mut x = a();
    let y = b();
    x -= &y;
    assert_eq!(x.get_value_at(1, 0), -4.0);
}

// ----- Neg: 2 ownership variants -----

#[test]
fn test_neg_owned() {
    let c = -a();
    assert_eq!(c.get_value_at(0, 0), -1.0);
    assert_eq!(c.get_value_at(1, 1), -2.0);
}

#[test]
fn test_neg_ref() {
    let x = a();
    let c = -&x;
    assert_eq!(c.get_value_at(0, 0), -1.0);
    assert_eq!(c.get_value_at(1, 1), -2.0);
}

// ----- Matrix Mul: 4 ownership variants -----

fn mm_a() -> CsrMatrix<f64> {
    // 2x3
    CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap()
}

fn mm_b() -> CsrMatrix<f64> {
    // 3x2
    CsrMatrix::from_triplets(3, 2, &[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]).unwrap()
}

#[test]
fn test_mul_owned_owned() {
    let c = mm_a() * mm_b();
    assert_eq!(c.get_value_at(0, 0), 16.0);
    assert_eq!(c.get_value_at(1, 1), 15.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_mul_ref_ref() {
    let x = mm_a();
    let y = mm_b();
    let c = &x * &y;
    assert_eq!(c.get_value_at(0, 0), 16.0);
}

#[test]
fn test_mul_owned_ref() {
    let y = mm_b();
    let c = mm_a() * &y;
    assert_eq!(c.get_value_at(1, 1), 15.0);
}

#[test]
fn test_mul_ref_owned() {
    let x = mm_a();
    let c = &x * mm_b();
    assert_eq!(c.get_value_at(0, 0), 16.0);
}

// ----- MulAssign -----

#[test]
fn test_mul_assign_owned() {
    let mut x = mm_a();
    x *= mm_b();
    assert_eq!(x.get_value_at(0, 0), 16.0);
    assert_eq!(x.shape(), (2, 2));
}

#[test]
fn test_mul_assign_ref() {
    let mut x = mm_a();
    let y = mm_b();
    x *= &y;
    assert_eq!(x.get_value_at(1, 1), 15.0);
}

// ----- Scalar Mul -----

#[test]
fn test_scalar_mul_owned() {
    let c = a() * 3.0f64;
    assert_eq!(c.get_value_at(0, 0), 3.0);
    assert_eq!(c.get_value_at(1, 1), 6.0);
}

#[test]
fn test_scalar_mul_ref() {
    let x = a();
    let c = &x * 3.0f64;
    assert_eq!(c.get_value_at(0, 0), 3.0);
    assert_eq!(c.get_value_at(1, 1), 6.0);
    // original unchanged
    assert_eq!(x.get_value_at(0, 0), 1.0);
}

#[test]
fn test_scalar_mul_assign() {
    let mut x = a();
    x *= 4.0f64;
    assert_eq!(x.get_value_at(0, 0), 4.0);
    assert_eq!(x.get_value_at(1, 1), 8.0);
}
