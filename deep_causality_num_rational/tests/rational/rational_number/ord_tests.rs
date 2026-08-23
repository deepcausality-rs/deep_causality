/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! ℚ is a totally ordered field.

use deep_causality_num::Zero;
use deep_causality_num_rational::Rational;

#[test]
fn equality_is_structural_because_the_form_is_canonical() {
    assert_eq!(Rational::new(1_i64, 2), Rational::new(2_i64, 4));
    assert_eq!(Rational::new(1_i64, 2), Rational::new(50_i64, 100));
    assert_ne!(Rational::new(1_i64, 2), Rational::new(1_i64, 3));
    assert_ne!(Rational::new(1_i64, 2), Rational::new(-1_i64, 2));
}

#[test]
fn ordering_compares_by_cross_multiplication() {
    assert!(Rational::new(1_i64, 3) < Rational::new(1_i64, 2));
    assert!(Rational::new(2_i64, 3) > Rational::new(1_i64, 2));
    assert!(Rational::new(-1_i64, 2) < Rational::zero());
    assert!(Rational::new(1_i64, 2) > Rational::new(-1_i64, 2));
}

#[test]
fn ordering_is_total() {
    let mut v = [
        Rational::new(3_i64, 4),
        Rational::new(1_i64, 2),
        Rational::new(-1_i64, 2),
        Rational::new(2_i64, 3),
        Rational::new(0_i64, 5),
    ];
    v.sort();
    assert_eq!(
        v,
        [
            Rational::new(-1_i64, 2),
            Rational::zero(),
            Rational::new(1_i64, 2),
            Rational::new(2_i64, 3),
            Rational::new(3_i64, 4),
        ]
    );
}

#[test]
fn ordering_agrees_with_the_integers_it_embeds() {
    for a in -5_i64..=5 {
        for b in -5_i64..=5 {
            let ra = Rational::from_integer(a);
            let rb = Rational::from_integer(b);
            assert_eq!(ra.cmp(&rb), a.cmp(&b), "disagreed on {a} vs {b}");
        }
    }
}

#[test]
fn density_between_any_two_rationals() {
    // ℚ is dense: the mean of two distinct rationals lies strictly between them.
    let a = Rational::new(1_i64, 3);
    let b = Rational::new(1_i64, 2);
    let mid = (a + b) / Rational::from_integer(2);
    assert!(a < mid && mid < b);
}
