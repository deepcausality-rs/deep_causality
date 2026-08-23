/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! ℚ is a totally ordered field.

use core::cmp::Ordering;
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
fn ordering_compares_by_continued_fraction() {
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

#[test]
fn ordering_does_not_overflow_at_the_top_of_the_range() {
    // The regression this pins: `Ord` used to compare `a/b` against `c/d` as `a·d` against `c·b`.
    // For `i64::MAX/1` against `1/2` that forms `i64::MAX · 2`, which panics in debug and wraps to
    // `-2` in release — reporting `i64::MAX` as the *smaller* of the two. A comparison that
    // silently inverts corrupts every `sort`, `max`, and binary search built on it.
    let huge = Rational::from_integer(i64::MAX);
    let half = Rational::new(1_i64, 2);

    assert_eq!(huge.cmp(&half), Ordering::Greater);
    assert_eq!(half.cmp(&huge), Ordering::Less);
    assert!(huge > half);

    // And at the bottom, where the numerator is the most negative one a canonical form can hold.
    let tiny = Rational::from_integer(i64::MIN + 1);
    assert_eq!(tiny.cmp(&half), Ordering::Less);
    assert_eq!(tiny.cmp(&huge), Ordering::Less);
    assert_eq!(tiny.cmp(&tiny), Ordering::Equal);
    assert_eq!(huge.cmp(&huge), Ordering::Equal);
}

#[test]
fn ordering_separates_values_whose_cross_products_do_not_fit() {
    // `n/(n-1)` strictly decreases in `n`, so this pair is ordered — but both cross products are
    // around `2¹²⁶` and settle nothing inside an `i64`. The continued fraction reaches the answer
    // after one step, comparing `i64::MAX - 2` against `i64::MAX - 1`.
    let a = Rational::new(i64::MAX, i64::MAX - 1);
    let b = Rational::new(i64::MAX - 1, i64::MAX - 2);
    assert!(a < b);
    assert!(b > a);
    assert_ne!(a, b);

    // Sorting mixed extremes stays consistent with those pairwise answers.
    let mut v = [b, Rational::from_integer(i64::MIN + 1), a, Rational::zero()];
    v.sort();
    assert_eq!(
        v,
        [Rational::from_integer(i64::MIN + 1), Rational::zero(), a, b]
    );
}

#[test]
fn ordering_agrees_with_exact_cross_multiplication_in_a_wider_width() {
    // `i128` has room for the products an `i32` does not, so this checks the *value* of every
    // comparison rather than merely that it does not panic — which is what catches the release
    // build, where the old implementation wrapped instead of aborting.
    let vals = [
        (0_i32, 1_i32),
        (1, 1),
        (-1, 1),
        (i32::MAX, 1),
        (i32::MIN + 1, 1),
        (i32::MAX, 2),
        (i32::MIN + 1, 2),
        (1, i32::MAX),
        (-1, i32::MAX),
        (i32::MAX, i32::MAX - 1),
        (i32::MAX - 1, i32::MAX - 2),
        (22, 7),
        (355, 113),
        (-355, 113),
    ];
    for (an, ad) in vals {
        for (bn, bd) in vals {
            let a = Rational::new(an, ad);
            let b = Rational::new(bn, bd);
            let expected = (i128::from(*a.numer()) * i128::from(*b.denom()))
                .cmp(&(i128::from(*b.numer()) * i128::from(*a.denom())));
            assert_eq!(a.cmp(&b), expected, "disagreed on {an}/{ad} vs {bn}/{bd}");
            assert_eq!(a.partial_cmp(&b), Some(expected));
        }
    }
}
