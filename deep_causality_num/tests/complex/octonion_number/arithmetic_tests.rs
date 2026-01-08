/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::ops::Div;
use deep_causality_num::utils_tests::utils_octonion_tests;
use deep_causality_num::{Float, Octonion, Zero};

// Test addition
#[test]
fn test_octonion_add() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
    let sum = o1 + o2;
    utils_octonion_tests::assert_octonion_approx_eq(
        sum,
        Octonion::new(10.0, 12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0),
        1e-9,
    );
}

// Test subtraction
#[test]
fn test_octonion_sub() {
    let o1 = Octonion::new(9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
    let o2 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let diff = o1 - o2;
    utils_octonion_tests::assert_octonion_approx_eq(
        diff,
        Octonion::new(8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0, 8.0),
        1e-9,
    );
}

// Test octonion multiplication (non-commutative, non-associative)
#[test]
fn test_octonion_mul() {
    let o1 = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1
    let o_e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e1
    let o_e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e2
    let o_e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // e3
    let o_e4 = Octonion::new(0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0); // e4
    let o_e5 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0); // e5
    let o_e6 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0); // e6
    let o_e7 = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0); // e7

    // Identity multiplication
    utils_octonion_tests::assert_octonion_approx_eq(o1 * o_e1, o_e1, 1e-9);

    // e1 * e2 = e3
    let e1_e2 = o_e1 * o_e2;
    utils_octonion_tests::assert_octonion_approx_eq(e1_e2, o_e3, 1e-9);

    // e2 * e1 = -e3
    let e2_e1 = o_e2 * o_e1;
    utils_octonion_tests::assert_octonion_approx_eq(e2_e1, -o_e3, 1e-9);

    // Some non-trivial multiplication from Cayley-Dickson table
    // e1 * e3 = -e2
    let e1_e3 = o_e1 * o_e3;
    utils_octonion_tests::assert_octonion_approx_eq(e1_e3, -o_e2, 1e-9);

    // e3 * e1 = e2
    let e3_e1 = o_e3 * o_e1;
    utils_octonion_tests::assert_octonion_approx_eq(e3_e1, o_e2, 1e-9);

    // e4 * e5 = e1
    let e4_e5 = o_e4 * o_e5;
    utils_octonion_tests::assert_octonion_approx_eq(e4_e5, o_e1, 1e-9);

    // e5 * e4 = -e1
    let e5_e4 = o_e5 * o_e4;
    utils_octonion_tests::assert_octonion_approx_eq(e5_e4, -o_e1, 1e-9);

    // e6 * e7 = e1
    let e6_e7 = o_e6 * o_e7;
    utils_octonion_tests::assert_octonion_approx_eq(e6_e7, o_e1, 1e-9);

    // e7 * e6 = -e1
    let e7_e6 = o_e7 * o_e6;
    utils_octonion_tests::assert_octonion_approx_eq(e7_e6, -o_e1, 1e-9);
}

// Test scalar multiplication
#[test]
fn test_octonion_mul_scalar() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let scalar = 2.0;
    let prod = o * scalar;
    utils_octonion_tests::assert_octonion_approx_eq(
        prod,
        Octonion::new(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0),
        1e-9,
    );
}

// Test division
#[test]
fn test_octonion_div() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o2 = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // Identity
    utils_octonion_tests::assert_octonion_approx_eq(o1 / o2, o1, 1e-9);

    // A simple case: (1+e1) / (1+e1) = 1
    let o_one_plus_e1 = Octonion::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let result = o_one_plus_e1 / o_one_plus_e1;
    utils_octonion_tests::assert_octonion_approx_eq(result, Octonion::identity(), 1e-9);

    // A more complex division (e1 / e2 = -e3)
    let o_e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e1
    let o_e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e2
    let o_e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // e3
    let result_e1_div_e2 = o_e1 / o_e2;
    utils_octonion_tests::assert_octonion_approx_eq(result_e1_div_e2, -o_e3, 1e-9);
}

// Test scalar division
#[test]
fn test_octonion_div_scalar() {
    let o = Octonion::new(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0);
    let scalar = 2.0;
    let quot = o / scalar;
    utils_octonion_tests::assert_octonion_approx_eq(
        quot,
        Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0),
        1e-9,
    );

    let o_inf = Octonion::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    let scalar_zero = 0.0;
    let result = o_inf / scalar_zero;
    assert!(result.s.is_infinite());
    assert!(result.e1.is_infinite());
    assert!(result.e7.is_infinite());
}

// Test division by zero octonion
#[test]
fn test_octonion_div_by_zero() {
    let o1 = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let o_zero = Octonion::<f64>::zero();
    let quot = o1.div(o_zero);
    assert!(quot.s.is_nan());
    assert!(quot.e1.is_nan());
    assert!(quot.e7.is_nan());
}

// Test sum of an iterator
#[test]
fn test_octonion_sum() {
    let o1 = Octonion::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    let o2 = Octonion::new(2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0);
    let o3 = Octonion::new(3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0);
    let octonions = vec![o1, o2, o3];
    let sum: Octonion<f64> = octonions.into_iter().sum();
    utils_octonion_tests::assert_octonion_approx_eq(
        sum,
        Octonion::new(6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0, 6.0),
        1e-9,
    );
}

// Test product of an iterator
#[test]
fn test_octonion_product() {
    let o_one = Octonion::identity();
    let o_e1 = Octonion::new(0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let o_e2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let o_e3 = Octonion::new(0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0); // e1 * e2 = e3

    let octonions = vec![o_one, o_e1, o_e2];
    let product: Octonion<f64> = octonions.into_iter().product();
    utils_octonion_tests::assert_octonion_approx_eq(product, o_e3, 1e-9);

    // Test with more general octonions, needs careful calculation based on Cayley-Dickson
    let o_gen1 = Octonion::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1 + e1
    let o_gen2 = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0); // e2
    let result_gen = o_gen1 * o_gen2; // (1+e1) * e2 = e2 + e1*e2 = e2 + e3
    let expected_gen = Octonion::new(0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0);
    utils_octonion_tests::assert_octonion_approx_eq(result_gen, expected_gen, 1e-9);
}

#[test]
fn test_octonion_sum_empty() {
    let v: Vec<Octonion<f64>> = Vec::new();
    let sum: Octonion<f64> = v.into_iter().sum();
    utils_octonion_tests::assert_octonion_approx_eq(sum, Octonion::zero(), 1e-9);
}

#[test]
fn test_octonion_product_empty() {
    let v: Vec<Octonion<f64>> = Vec::new();
    let product: Octonion<f64> = v.into_iter().product();
    utils_octonion_tests::assert_octonion_approx_eq(product, Octonion::identity(), 1e-9);
}
