/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_uncertain::utils_tests::utils_tests_node;
use deep_causality_uncertain::{ArithmeticOperator, ComparisonOperator, LogicalOperator};

#[test]
fn test_debug_leaf_f64() {
    let node = utils_tests_node::create_leaf_f64(1.23);
    let expected = format!("LeafF64 {{ node_id: {:?}, dist: Point(1.23) }}", node.id());
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_leaf_bool() {
    let node = utils_tests_node::create_leaf_bool(true);
    let expected = format!("LeafBool {{ node_id: {:?}, dist: Point(true) }}", node.id(),);
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_arithmetic_op() {
    let lhs = utils_tests_node::create_leaf_f64(1.0);
    let rhs = utils_tests_node::create_leaf_f64(2.0);
    let node =
        utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs.clone(), rhs.clone());
    let expected = format!(
        "ArithmeticOp {{ node_id: {:?}, op: Add, lhs: LeafF64 {{ node_id: {:?}, dist: Point(1.0) }}, rhs: LeafF64 {{ node_id: {:?}, dist: Point(2.0) }} }}",
        node.id(),
        lhs.id(),
        rhs.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_comparison_op() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node = utils_tests_node::create_comparison_op(
        ComparisonOperator::GreaterThan,
        0.5,
        operand.clone(),
    );
    let expected = format!(
        "ComparisonOp {{ node_id: {:?}, op: GreaterThan, threshold: 0.5, operand: LeafF64 {{ node_id: {:?}, dist: Point(1.0) }} }}",
        node.id(),
        operand.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_logical_op() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let node =
        utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1.clone(), op2.clone()]);
    let expected = format!(
        "LogicalOp {{ node_id: {:?}, op: And, operands: [LeafBool {{ node_id: {:?}, dist: Point(true) }}, LeafBool {{ node_id: {:?}, dist: Point(false) }}] }}",
        node.id(),
        op1.id(),
        op2.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_function_op_f64() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node = utils_tests_node::create_function_op_f64(operand.clone());
    let expected = format!(
        "FunctionOp {{ node_id: {:?}, func:  Fn(f64) -> bool, operand: LeafF64 {{ node_id: {:?}, dist: Point(1.0) }} }}",
        node.id(),
        operand.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_function_op_bool() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node = utils_tests_node::create_function_op_bool(operand.clone());
    let expected = format!(
        "FunctionOpBool {{ node_id: {:?}, func:  Fn(f64) -> bool, operand: LeafF64 {{ node_id: {:?}, dist: Point(1.0) }} }}",
        node.id(),
        operand.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_negation_op() {
    let operand = utils_tests_node::create_leaf_bool(true);
    let node = utils_tests_node::create_negation_op(operand.clone());
    let expected = format!(
        "NegationOp {{ node_id: {:?}, operand: LeafBool {{ node_id: {:?}, dist: Point(true) }} }}",
        node.id(),
        operand.id(),
    );
    assert_eq!(format!("{:?}", node), expected);
}

#[test]
fn test_debug_conditional_op() {
    let condition = utils_tests_node::create_leaf_bool(true);
    let if_true = utils_tests_node::create_leaf_f64(1.0);
    let if_false = utils_tests_node::create_leaf_f64(0.0);
    let node = utils_tests_node::create_conditional_op(
        condition.clone(),
        if_true.clone(),
        if_false.clone(),
    );
    let expected = format!(
        "ConditionalOp {{ node_id: {:?}, condition: LeafBool {{ node_id: {:?}, dist: Point(true) }}, if_true: LeafF64 {{ node_id: {:?}, dist: Point(1.0) }}, if_false: LeafF64 {{ node_id: {:?}, dist: Point(0.0) }} }}",
        node.id(),
        condition.id(),
        if_true.id(),
        if_false.id()
    );
    assert_eq!(format!("{:?}", node), expected);
}
