/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The property family is a fixture, and a fixture that quietly loses a corner case takes the
//! coverage with it while every property test still passes: the properties hold on whatever data
//! they are handed. These cases assert the shape `FAMILY`'s docstring claims, so an edit that drops
//! a sign or shortens a row fails here rather than silently narrowing what the suite runs over.

use deep_causality_stats::utils_tests::samples::FAMILY;

/// Every member carries at least two observations, which is what makes `variance` and `std_dev`
/// defined on all of them — the claim the docstring rests the family on.
#[test]
fn test_family_members_all_admit_the_corrected_variance() {
    for (i, sample) in FAMILY.iter().enumerate() {
        assert!(
            sample.len() >= 2,
            "family member {i} has {} observations; the corrected variance needs two",
            sample.len()
        );
    }
}

/// Every value is finite. A non-finite entry would make the properties vacuous rather than false.
#[test]
fn test_family_values_are_all_finite() {
    for (i, sample) in FAMILY.iter().enumerate() {
        assert!(
            sample.iter().all(|v| v.is_finite()),
            "family member {i} carries a non-finite observation"
        );
    }
}

/// The family spans the sign cases it is built to span. Two members straddle zero — one over three
/// observations, one over two — and losing either minus sign would leave the suite running only on
/// non-negative data while every property still held.
#[test]
fn test_family_spans_both_signs() {
    let straddling = FAMILY
        .iter()
        .filter(|s| s.iter().any(|&v| v < 0.0) && s.iter().any(|&v| v > 0.0))
        .count();
    assert_eq!(
        straddling, 2,
        "the family should carry two samples straddling zero, found {straddling}"
    );
}

/// A constant sample is present: the degenerate case where the variance is zero and every
/// scale-relative assertion has to fall back on an absolute floor.
#[test]
fn test_family_carries_a_constant_sample() {
    let constant = FAMILY
        .iter()
        .filter(|s| s.windows(2).all(|w| w[0] == w[1]))
        .count();
    assert_eq!(
        constant, 1,
        "the family should carry exactly one constant sample, found {constant}"
    );
}

/// A sample with repeated-but-not-constant values is present, so ties are exercised without
/// collapsing to the constant case.
#[test]
fn test_family_carries_a_sample_with_repeats() {
    let repeats = FAMILY.iter().any(|s| {
        let distinct = s
            .iter()
            .filter(|v| s.iter().filter(|u| u == v).count() > 1)
            .count();
        distinct > 0 && !s.windows(2).all(|w| w[0] == w[1])
    });
    assert!(
        repeats,
        "the family should carry a sample with repeated values that is not constant"
    );
}
