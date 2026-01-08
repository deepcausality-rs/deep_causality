/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::math_utils;

#[test]
fn test_sum_float() {
    let v = vec![1.0, 2.0, 3.0, 4.0];
    let res = math_utils::sum(v);

    assert_eq!(res, 10.0);
}

#[test]
fn test_sum_int() {
    let v = vec![1, 2, 3, 4];
    let res = math_utils::sum(v);

    assert_eq!(res, 10);
}

#[test]
fn test_abs_num_neg() {
    let n = -1.0;
    let res = math_utils::abs_num(n);
    assert_eq!(res, 1.0);
}

#[test]
fn test_abs_num_pos() {
    let n = 1.0;
    let res = math_utils::abs_num(n);
    assert_eq!(res, 1.0);
}
