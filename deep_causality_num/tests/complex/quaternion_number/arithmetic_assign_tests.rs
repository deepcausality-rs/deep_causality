/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Quaternion;

#[test]
fn test_add_assign() {
    let mut q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let other = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    q += other;
    let expected = Quaternion::new(6.0, 8.0, 10.0, 12.0);
    assert_eq!(q, expected);
}

#[test]
fn test_sub_assign() {
    let mut q = Quaternion::new(5.0, 6.0, 7.0, 8.0);
    let other = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    q -= other;
    let expected = Quaternion::new(4.0, 4.0, 4.0, 4.0);
    assert_eq!(q, expected);
}

#[test]
fn test_mul_assign() {
    let mut q = Quaternion::new(1.0, 0.0, 0.0, 0.0); // 1
    let i = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let j = Quaternion::new(0.0, 0.0, 1.0, 0.0); // j

    q *= i; // q becomes i
    assert_eq!(q, i);

    q *= j; // q becomes i * j = k
    let expected = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    assert_eq!(q, expected);
}

#[test]
fn test_div_assign() {
    let mut q = Quaternion::new(1.0, 0.0, 0.0, 0.0); // 1
    let i = Quaternion::new(0.0, 1.0, 0.0, 0.0); // i
    let j = Quaternion::new(0.0, 0.0, 1.0, 0.0); // j

    q /= i; // q becomes 1 / i = -i
    let expected_neg_i = Quaternion::new(0.0, -1.0, 0.0, 0.0);
    assert_eq!(q, expected_neg_i);

    q /= j; // q becomes -i / j = -(-k) = k
    let expected_k = Quaternion::new(0.0, 0.0, 0.0, 1.0);
    assert_eq!(q, expected_k);
}
