/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::LogicalOperator;

#[test]
fn test_logical_operator_and() {
    let op = LogicalOperator::And;
    assert!(!op.apply(false, false));
    assert!(!op.apply(false, true));
    assert!(!op.apply(true, false));
    assert!(op.apply(true, true));
}

#[test]
fn test_logical_operator_or() {
    let op = LogicalOperator::Or;
    assert!(!op.apply(false, false));
    assert!(op.apply(false, true));
    assert!(op.apply(true, false));
    assert!(op.apply(true, true));
}

#[test]
fn test_logical_operator_not() {
    let op = LogicalOperator::Not;
    assert!(op.apply(false, false)); // b is ignored
    assert!(!op.apply(true, false)); // b is ignored
}

#[test]
fn test_logical_operator_nor() {
    let op = LogicalOperator::NOR;
    assert!(op.apply(false, false));
    assert!(!op.apply(false, true));
    assert!(!op.apply(true, false));
    assert!(!op.apply(true, true));
}

#[test]
fn test_logical_operator_xor() {
    let op = LogicalOperator::XOR;
    assert!(!op.apply(false, false));
    assert!(op.apply(false, true));
    assert!(op.apply(true, false));
    assert!(!op.apply(true, true));
}

#[test]
fn test_logical_operator_display() {
    assert_eq!(format!("{}", LogicalOperator::And), "AND");
    assert_eq!(format!("{}", LogicalOperator::Or), "OR");
    assert_eq!(format!("{}", LogicalOperator::Not), "NOT");
    assert_eq!(format!("{}", LogicalOperator::NOR), "NOR");
    assert_eq!(format!("{}", LogicalOperator::XOR), "XOR");
}

#[test]
fn test_logical_operator_debug_clone_copy() {
    let op = LogicalOperator::And;

    // Test Debug
    assert_eq!(format!("{:?}", op), "And");

    // Test Clone
    let cloned_op = op.clone();
    assert_eq!(cloned_op, op);

    // Test Copy (by assignment)
    let copied_op = op;
    assert_eq!(copied_op, op);
}
