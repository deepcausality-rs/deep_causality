/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
