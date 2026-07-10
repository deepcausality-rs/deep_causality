/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::ArithmeticOperator;

#[test]
fn test_arithmetic_operator_add() {
    let op = ArithmeticOperator::Add;
    assert_eq!(op.apply(5.0, 3.0), 8.0);
    assert_eq!(op.apply(-5.0, 3.0), -2.0);
    assert_eq!(op.apply(5.0, -3.0), 2.0);
    assert_eq!(op.apply(-5.0, -3.0), -8.0);
    assert_eq!(op.apply(0.0, 0.0), 0.0);
    assert_eq!(op.apply(100.0, 200.0), 300.0);
    assert_eq!(op.apply(0.1, 0.2), 0.30000000000000004); // Floating point precision
}

#[test]
fn test_arithmetic_operator_sub() {
    let op = ArithmeticOperator::Sub;
    assert_eq!(op.apply(5.0, 3.0), 2.0);
    assert_eq!(op.apply(-5.0, 3.0), -8.0);
    assert_eq!(op.apply(5.0, -3.0), 8.0);
    assert_eq!(op.apply(-5.0, -3.0), -2.0);
    assert_eq!(op.apply(0.0, 0.0), 0.0);
    assert_eq!(op.apply(200.0, 100.0), 100.0);
}

#[test]
fn test_arithmetic_operator_mul() {
    let op = ArithmeticOperator::Mul;
    assert_eq!(op.apply(5.0, 3.0), 15.0);
    assert_eq!(op.apply(-5.0, 3.0), -15.0);
    assert_eq!(op.apply(5.0, -3.0), -15.0);
    assert_eq!(op.apply(-5.0, -3.0), 15.0);
    assert_eq!(op.apply(0.0, 5.0), 0.0);
    assert_eq!(op.apply(5.0, 0.0), 0.0);
    assert_eq!(op.apply(10.0, 0.5), 5.0);
}

#[test]
fn test_arithmetic_operator_div() {
    let op = ArithmeticOperator::Div;
    assert_eq!(op.apply(6.0, 3.0), 2.0);
    assert_eq!(op.apply(-6.0, 3.0), -2.0);
    assert_eq!(op.apply(6.0, -3.0), -2.0);
    assert_eq!(op.apply(-6.0, -3.0), 2.0);
    assert_eq!(op.apply(0.0, 5.0), 0.0);
    assert!(op.apply::<f64>(5.0, 0.0).is_infinite()); // Division by zero is positive infinity
    assert!(op.apply::<f64>(-5.0, 0.0).is_infinite()); // Division by zero is negative infinity
    assert!(op.apply::<f64>(0.0, 0.0).is_nan()); // 0/0 is NaN
}

#[test]
fn test_arithmetic_operator_min() {
    let op = ArithmeticOperator::Min;
    assert_eq!(op.apply(2.0, 3.0), 2.0); // a < b -> a
    assert_eq!(op.apply(3.0, 2.0), 2.0); // a > b -> b
    assert_eq!(op.apply(2.0, 2.0), 2.0); // a == b -> a
    assert_eq!(op.apply(-5.0, 3.0), -5.0);
    assert_eq!(op.apply(3.0, -5.0), -5.0);
    assert_eq!(op.apply(0.0, 0.0), 0.0);
}

#[test]
fn test_arithmetic_operator_max() {
    let op = ArithmeticOperator::Max;
    assert_eq!(op.apply(3.0, 2.0), 3.0); // a > b -> a
    assert_eq!(op.apply(2.0, 3.0), 3.0); // a < b -> b
    assert_eq!(op.apply(2.0, 2.0), 2.0); // a == b -> a
    assert_eq!(op.apply(-5.0, 3.0), 3.0);
    assert_eq!(op.apply(3.0, -5.0), 3.0);
    assert_eq!(op.apply(0.0, 0.0), 0.0);
}

#[test]
fn test_arithmetic_operator_display() {
    assert_eq!(format!("{}", ArithmeticOperator::Add), "Add");
    assert_eq!(format!("{}", ArithmeticOperator::Sub), "Sub");
    assert_eq!(format!("{}", ArithmeticOperator::Mul), "Mul");
    assert_eq!(format!("{}", ArithmeticOperator::Div), "Div");
    assert_eq!(format!("{}", ArithmeticOperator::Min), "Min");
    assert_eq!(format!("{}", ArithmeticOperator::Max), "Max");
}

#[test]
fn test_arithmetic_operator_debug_clone_copy() {
    let op = ArithmeticOperator::Add;

    // Test Debug
    assert_eq!(format!("{:?}", op), "Add");

    // Test Clone
    let cloned_op = op;
    assert_eq!(cloned_op, op);

    // Test Copy (by assignment)
    let copied_op = op;
    assert_eq!(copied_op, op);
}
