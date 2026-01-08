/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Octonion;
use deep_causality_num::utils_tests::utils_octonion_tests;

// Test AddAssign
#[test]
fn test_octonion_add_assign() {
    let mut o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let other = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
    o += other;
    utils_octonion_tests::assert_octonion_approx_eq(
        o,
        Octonion::new(10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0),
        1e-9,
    );
}

// Test SubAssign
#[test]
fn test_octonion_sub_assign() {
    let mut o = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
    let other = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    o -= other;
    utils_octonion_tests::assert_octonion_approx_eq(
        o,
        Octonion::new(8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0),
        1e-9,
    );
}

// Test MulAssign
#[test]
fn test_octonion_mul_assign() {
    let mut o = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1
    let o_e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e1
    let o_e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e2
    let o_e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // e3 (e1 * e2)

    o *= o_e1; // o becomes e1
    utils_octonion_tests::assert_octonion_approx_eq(o, o_e1, 1e-9);

    o *= o_e2; // o becomes e1 * e2 = e3
    utils_octonion_tests::assert_octonion_approx_eq(o, o_e3, 1e-9);
}

// Test DivAssign
#[test]
fn test_octonion_div_assign() {
    let mut o = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1
    let o_e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e1
    let o_e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e2
    let o_e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // e3

    o /= o_e1; // o becomes 1 / e1 = -e1
    utils_octonion_tests::assert_octonion_approx_eq(o, -o_e1, 1e-9);

    o /= o_e2; // o becomes (-e1) / e2 = e3
    utils_octonion_tests::assert_octonion_approx_eq(o, o_e3, 1e-9);
}

// Test MulAssign scalar
#[test]
fn test_octonion_mul_assign_scalar() {
    let mut o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let scalar = 2.0;
    o *= scalar;
    utils_octonion_tests::assert_octonion_approx_eq(
        o,
        Octonion::new(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0),
        1e-9,
    );
}

// Test DivAssign scalar
#[test]
fn test_octonion_div_assign_scalar() {
    let mut o = Octonion::new(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0);
    let scalar = 2.0;
    o /= scalar;
    utils_octonion_tests::assert_octonion_approx_eq(
        o,
        Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        1e-9,
    );
}
