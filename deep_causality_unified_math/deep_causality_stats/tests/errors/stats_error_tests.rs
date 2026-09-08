/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the crate's error type.
//!
//! `Display` is what a caller sees when a fit fails in a log, so an implementation returning an
//! empty string would be silently useless. Nothing asserted its output until mutation testing
//! replaced the whole `fmt` body with `Ok(())` and every suite stayed green.

use deep_causality_stats::{StatsError, StatsErrorEnum};

/// Every variant renders its own prefix and carries its message through.
#[test]
fn display_names_the_variant_and_keeps_the_message() {
    let cases: [(StatsError, &str); 10] = [
        (StatsError::EmptyInput("no rows"), "Empty input: no rows"),
        (
            StatsError::InsufficientSamples("needs two"),
            "Insufficient samples: needs two",
        ),
        (
            StatsError::NegativeProbability("below zero"),
            "Negative probability: below zero",
        ),
        (
            StatsError::DimensionMismatch("ragged"),
            "Dimension mismatch: ragged",
        ),
        (
            StatsError::NonPositiveScale("sigma squared"),
            "Non-positive scale: sigma squared",
        ),
        (
            StatsError::RankDeficient("pivot vanished"),
            "Rank deficient: pivot vanished",
        ),
        (
            StatsError::NotConverged(17, "step too large"),
            "Did not converge after 17 iterations: step too large",
        ),
        (
            StatsError::InvalidBinCount("below two"),
            "Invalid bin count: below two",
        ),
        (
            StatsError::NonFiniteInput("a NaN"),
            "Non-finite input: a NaN",
        ),
        (
            StatsError::ConversionFailed("a count"),
            "Conversion failed: a count",
        ),
    ];
    for (err, expected) in cases {
        assert_eq!(err.to_string(), expected);
    }
}

/// `NotConverged` reports the count it was built with.
///
/// The count is the reason this variant carries a field: it distinguishes a cap that is merely too
/// low from a problem that does not converge at all, and a `Display` that dropped it would lose
/// exactly that.
#[test]
fn not_converged_reports_its_iteration_count() {
    for n in [1usize, 2, 100, 10_000] {
        let rendered = StatsError::NotConverged(n, "detail").to_string();
        assert!(
            rendered.contains(&n.to_string()),
            "the iteration count must appear, got {rendered:?}"
        );
    }
}

/// The named constructors build the variants they name.
#[test]
fn each_constructor_builds_its_own_variant() {
    assert!(matches!(
        StatsError::EmptyInput("x").kind(),
        StatsErrorEnum::EmptyInput(_)
    ));
    assert!(matches!(
        StatsError::RankDeficient("x").kind(),
        StatsErrorEnum::RankDeficient(_)
    ));
    assert!(matches!(
        StatsError::NotConverged(3, "x").kind(),
        StatsErrorEnum::NotConverged { iterations: 3, .. }
    ));
    // And a constructor's message reaches the variant rather than being dropped.
    match StatsError::NonFiniteInput("the message").kind() {
        StatsErrorEnum::NonFiniteInput(m) => assert_eq!(m, "the message"),
        other => panic!("wrong variant: {other:?}"),
    }
}

/// The type carries the derives a caller needs to compare and log one.
#[test]
fn the_error_is_debug_clone_and_comparable() {
    let a = StatsError::EmptyInput("same");
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, StatsError::EmptyInput("different"));
    assert_ne!(a, StatsError::NonFiniteInput("same"));
    assert!(!format!("{a:?}").is_empty());
}
