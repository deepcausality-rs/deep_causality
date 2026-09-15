/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::ComparisonOperator;

#[test]
fn test_comparison_operator_greater_than() {
    let op = ComparisonOperator::GreaterThan;
    assert!(op.apply(5.0, 3.0));
    assert!(!op.apply(3.0, 5.0));
    assert!(!op.apply(5.0, 5.0));
    assert!(op.apply(0.1, 0.0));
    assert!(!op.apply(0.0, 0.0));
    assert!(op.apply(f64::INFINITY, f64::MAX));
    assert!(!op.apply(f64::NEG_INFINITY, 0.0));
    assert!(!op.apply(f64::NAN, 1.0)); // NaN comparisons are always false
}

#[test]
fn test_comparison_operator_less_than() {
    let op = ComparisonOperator::LessThan;
    assert!(op.apply(3.0, 5.0));
    assert!(!op.apply(5.0, 3.0));
    assert!(!op.apply(5.0, 5.0));
    assert!(!op.apply(0.1, 0.0));
    assert!(op.apply(0.0, 0.1));
    assert!(!op.apply(f64::MIN, f64::NEG_INFINITY)); // MIN is greater than NEG_INFINITY
    assert!(op.apply(f64::MAX, f64::INFINITY));
    assert!(!op.apply(f64::NAN, 1.0)); // NaN comparisons are always false
}

#[test]
fn test_comparison_operator_equal_to() {
    let op = ComparisonOperator::EqualTo;
    assert!(op.apply(5.0, 5.0));
    assert!(!op.apply(5.0, 3.0));
    assert!(op.apply(0.0, -0.0)); // 0.0 and -0.0 are considered equal
    assert!(op.apply(1.0, 1.0 + f64::EPSILON / 2.0)); // Within epsilon
    assert!(!op.apply(1.0, 1.0 + f64::EPSILON * 2.0)); // Outside epsilon
    assert!(!op.apply(f64::NAN, f64::NAN)); // NaN is never equal to itself
    assert!(op.apply(f64::INFINITY, f64::INFINITY)); // Infinity is equal to itself
    assert!(!op.apply(f64::INFINITY, f64::MAX));
    assert!(op.apply(f64::NEG_INFINITY, f64::NEG_INFINITY)); // Negative infinity is equal to itself
}

#[test]
fn test_comparison_operator_display() {
    assert_eq!(format!("{}", ComparisonOperator::GreaterThan), ">");
    assert_eq!(format!("{}", ComparisonOperator::LessThan), "<");
    assert_eq!(format!("{}", ComparisonOperator::EqualTo), "==");
}

#[test]
fn test_comparison_operator_debug_clone_copy() {
    let op = ComparisonOperator::GreaterThan;

    // Test Debug
    assert_eq!(format!("{:?}", op), "GreaterThan");

    // Test Clone
    let cloned_op = op;
    assert_eq!(cloned_op, op);

    // Test Copy (by assignment)
    let copied_op = op;
    assert_eq!(copied_op, op);
}

/// The two non-strict operators are **not** the negations of the strict ones, and the difference
/// is exactly a `NaN` operand.
///
/// `!(a < b)` is `true` for a `NaN` where `a >= b` is `false`. That is why `within_range` is built
/// from `>=` and `<=` stated directly rather than from negated strict comparisons: the negated form
/// reports a `NaN` draw as inside every range.
#[test]
fn the_non_strict_operators_are_not_negated_strict_ones() {
    use deep_causality_uncertain::ComparisonOperator::*;

    // On ordinary values the two forms agree.
    for (a, b) in [
        (1.0f64, 2.0),
        (2.0, 1.0),
        (1.0, 1.0),
        (-1.0, 1.0),
        (-2.0, -1.0),
    ] {
        assert_eq!(
            GreaterThanOrEqual.apply(a, b),
            !LessThan.apply(a, b),
            "{a} >= {b}"
        );
        assert_eq!(
            LessThanOrEqual.apply(a, b),
            !GreaterThan.apply(a, b),
            "{a} <= {b}"
        );
    }

    // On a NaN they do not, and the direct form is the correct one.
    let nan = f64::NAN;
    assert!(!GreaterThanOrEqual.apply(nan, 0.0), "NaN >= 0 is false");
    assert!(!LessThanOrEqual.apply(nan, 0.0), "NaN <= 0 is false");
    assert!(!LessThan.apply(nan, 0.0), "NaN < 0 is false");
    assert!(!GreaterThan.apply(nan, 0.0), "NaN > 0 is false");
    assert!(
        !LessThan.apply(nan, 0.0) && !GreaterThanOrEqual.apply(nan, 0.0),
        "both are false for a NaN, so one is not the negation of the other"
    );
}

/// The non-strict operators admit the boundary, which is what separates them from the strict ones.
#[test]
fn the_non_strict_operators_admit_the_boundary() {
    use deep_causality_uncertain::ComparisonOperator::*;

    assert!(GreaterThanOrEqual.apply(3.0f64, 3.0), "3 >= 3");
    assert!(LessThanOrEqual.apply(3.0f64, 3.0), "3 <= 3");
    assert!(!GreaterThan.apply(3.0f64, 3.0), "3 > 3 is false");
    assert!(!LessThan.apply(3.0f64, 3.0), "3 < 3 is false");

    // Infinity is its own boundary.
    assert!(GreaterThanOrEqual.apply(f64::INFINITY, f64::INFINITY));
    assert!(LessThanOrEqual.apply(f64::NEG_INFINITY, f64::NEG_INFINITY));
}

/// Every operator renders as the symbol a reader expects in a graph dump.
#[test]
fn every_operator_renders_as_its_symbol() {
    use deep_causality_uncertain::ComparisonOperator::*;

    assert_eq!(format!("{GreaterThan}"), ">");
    assert_eq!(format!("{LessThan}"), "<");
    assert_eq!(format!("{EqualTo}"), "==");
    assert_eq!(format!("{GreaterThanOrEqual}"), ">=");
    assert_eq!(format!("{LessThanOrEqual}"), "<=");
}
