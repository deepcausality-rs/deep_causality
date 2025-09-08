/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::utils_tests::utils_tests_node;
use deep_causality_uncertain::{ArithmeticOperator, ComparisonOperator, LogicalOperator};

#[test]
fn test_is_leaf_f64() {
    let node = utils_tests_node::create_leaf_f64(42.0);
    assert!(node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_leaf_bool() {
    let node = utils_tests_node::create_leaf_bool(true);
    assert!(!node.is_leaf_64());
    assert!(node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_arithmetic_op() {
    let lhs = utils_tests_node::create_leaf_f64(1.0);
    let rhs = utils_tests_node::create_leaf_f64(2.0);
    let node = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs, rhs);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_comparison_op() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node =
        utils_tests_node::create_comparison_op(ComparisonOperator::GreaterThan, 0.5, operand);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_logical_op() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let node = utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1, op2]);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_function_op_f64() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node = utils_tests_node::create_function_op_f64(operand);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_function_op_bool() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node = utils_tests_node::create_function_op_bool(operand);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_negation_op() {
    let operand = utils_tests_node::create_leaf_bool(true);
    let node = utils_tests_node::create_negation_op(operand);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(node.is_negation_op());
    assert!(!node.is_conditional_op());
}

#[test]
fn test_is_conditional_op() {
    let condition = utils_tests_node::create_leaf_bool(true);
    let if_true = utils_tests_node::create_leaf_f64(1.0);
    let if_false = utils_tests_node::create_leaf_f64(0.0);
    let node = utils_tests_node::create_conditional_op(condition, if_true, if_false);
    assert!(!node.is_leaf_64());
    assert!(!node.is_leaf_bool());
    assert!(!node.is_arithmetic_op());
    assert!(!node.is_comparison_op());
    assert!(!node.is_logical_op());
    assert!(!node.is_function_op_f64());
    assert!(!node.is_function_op_bool());
    assert!(!node.is_negation_op());
    assert!(node.is_conditional_op());
}
