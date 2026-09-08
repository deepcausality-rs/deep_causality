/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the independent oracles.
//!
//! An oracle a suite trusts has to be checked itself, against values obtained without it. These
//! expectations are hand arithmetic, written out in the comment above each assertion, and the
//! sample family is checked for the property that makes it a family worth running over.

use deep_causality_num::{Float106, lift};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::oracles::pairwise_variance_oracle;
use deep_causality_stats::utils_tests::samples::FAMILY;

/// `Σ_{i<j}(xᵢ − xⱼ)² / (n(n − 1))` on samples whose pair differences are small integers.
fn check_oracle_against_hand_arithmetic<
    T: deep_causality_algebra::RealField
        + deep_causality_num::FromPrimitive
        + deep_causality_num::ToPrimitive,
>(
    tol: f64,
) {
    // [1, 2, 3]: pair differences 1, 2, 1; squares 1 + 4 + 1 = 6; over n(n−1) = 6; so 1.
    let got = pairwise_variance_oracle(&lift_array::<T>(&[1.0, 2.0, 3.0]));
    assert!(
        (got - lift::<T>(1.0)).abs() <= lift::<T>(tol),
        "pairwise variance of [1,2,3] is 6/6 = 1"
    );

    // [0, 4]: one pair, difference 4, square 16; over n(n−1) = 2; so 8.
    let got = pairwise_variance_oracle(&lift_array::<T>(&[0.0, 4.0]));
    assert!(
        (got - lift::<T>(8.0)).abs() <= lift::<T>(tol),
        "pairwise variance of [0,4] is 16/2 = 8"
    );

    // A constant sample has no pair difference at all, so the oracle is exactly zero.
    let got = pairwise_variance_oracle(&lift_array::<T>(&[7.0, 7.0, 7.0, 7.0]));
    assert!(
        got == T::zero(),
        "a constant sample has zero dispersion, exactly"
    );
}

#[test]
fn oracle_matches_hand_arithmetic_f32() {
    check_oracle_against_hand_arithmetic::<f32>(1e-6);
}

#[test]
fn oracle_matches_hand_arithmetic_f64() {
    check_oracle_against_hand_arithmetic::<f64>(1e-12);
}

#[test]
fn oracle_matches_hand_arithmetic_float106() {
    check_oracle_against_hand_arithmetic::<Float106>(1e-25);
}

/// The oracle is translation invariant, as a dispersion must be.
///
/// It never forms the mean, so this is not true by construction the way it is for the definition:
/// it follows from every term being a difference of two observations.
#[test]
fn the_oracle_is_translation_invariant() {
    let base = lift_array::<f64>(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
    let shifted: Vec<f64> = base.iter().map(|x| x + 1000.0).collect();
    let a = pairwise_variance_oracle(&base);
    let b = pairwise_variance_oracle(&shifted);
    assert!(
        (a - b).abs() <= 1e-9,
        "shifting every observation leaves the dispersion unchanged: {a} against {b}"
    );
}

/// Scaling by `k` scales the oracle by `k²`.
#[test]
fn the_oracle_scales_quadratically() {
    let base = lift_array::<f64>(&[1.0, 2.0, 3.0, 5.0]);
    let scaled: Vec<f64> = base.iter().map(|x| x * 3.0).collect();
    let a = pairwise_variance_oracle(&base);
    let b = pairwise_variance_oracle(&scaled);
    assert!(
        (b - 9.0 * a).abs() <= 1e-9,
        "scaling by 3 scales the dispersion by 9: {a} against {b}"
    );
}

/// The sample family is what its doc says it is.
///
/// The cases exist to be different from each other, so the properties that run over them are
/// exercised at more than one shape. If they collapse to one kind of sample, every property test
/// that uses the family gets weaker without any of them failing.
#[test]
fn the_sample_family_covers_the_shapes_it_claims() {
    assert_eq!(FAMILY.len(), 5, "five samples");
    for (i, s) in FAMILY.iter().enumerate() {
        assert!(
            s.len() >= 2,
            "sample {i} must have at least two observations, or variance is undefined on it"
        );
    }
    // A sample with repeats, so a tie is exercised.
    assert!(
        FAMILY.iter().any(|s| {
            let mut seen = s.to_vec();
            seen.sort_by(|a, b| a.partial_cmp(b).unwrap());
            seen.windows(2).any(|w| w[0] == w[1])
        }),
        "one sample repeats a value"
    );
    // A sample straddling zero, so a sign change is exercised.
    assert!(
        FAMILY
            .iter()
            .any(|s| s.iter().any(|&x| x < 0.0) && s.iter().any(|&x| x > 0.0)),
        "one sample straddles zero: without a negative observation the sign handling is never \
         reached, and the minus signs in the fixtures would be free to vanish"
    );
    // A constant sample, where the dispersion is exactly zero.
    assert!(
        FAMILY.iter().any(|s| s.windows(2).all(|w| w[0] == w[1])),
        "one sample is constant"
    );
    // A two-element sample, the smallest on which the corrected variance is defined.
    assert!(
        FAMILY.iter().any(|s| s.len() == 2),
        "one sample is the two-element boundary case"
    );
}
