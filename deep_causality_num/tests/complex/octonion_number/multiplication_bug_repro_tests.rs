/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Octonion;

#[test]
fn test_octonion_multiplication_bug_repro() {
    // Test Case 1: e2 * e5
    // Expected: -e7
    // Buggy implementation returns: e7
    let e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let e5 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
    let expected_neg_e7 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0);
    let result1 = e2 * e5;

    assert!(
        approx_eq(&result1, &expected_neg_e7),
        "e2 * e5 should be -e7, but got {:?}",
        result1
    );

    // Test Case 2: e5 * e2
    // Expected: e7 (anti-commutative)
    // Buggy implementation returns: -e7
    let expected_e7 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
    let result2 = e5 * e2;

    assert!(
        approx_eq(&result2, &expected_e7),
        "e5 * e2 should be e7, but got {:?}",
        result2
    );

    // Test Case 3: e3 * e5
    // Expected: e6
    // Buggy implementation returns: -e6
    let e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    let expected_e6 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    let result3 = e3 * e5;

    assert!(
        approx_eq(&result3, &expected_e6),
        "e3 * e5 should be e6, but got {:?}",
        result3
    );

    // Test Case 4: e5 * e3
    // Expected: -e6 (anti-commutative)
    // Buggy implementation returns: e6
    let expected_neg_e6 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0);
    let result4 = e5 * e3;

    assert!(
        approx_eq(&result4, &expected_neg_e6),
        "e5 * e3 should be -e6, but got {:?}",
        result4
    );
}

fn approx_eq(a: &Octonion<f64>, b: &Octonion<f64>) -> bool {
    let eps = 1e-9;
    (a.s - b.s).abs() < eps
        && (a.e1 - b.e1).abs() < eps
        && (a.e2 - b.e2).abs() < eps
        && (a.e3 - b.e3).abs() < eps
        && (a.e4 - b.e4).abs() < eps
        && (a.e5 - b.e5).abs() < eps
        && (a.e6 - b.e6).abs() < eps
        && (a.e7 - b.e7).abs() < eps
}
