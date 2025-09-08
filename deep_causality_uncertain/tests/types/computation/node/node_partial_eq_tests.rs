/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_uncertain::utils_tests::utils_tests_node;
use deep_causality_uncertain::*;
use std::sync::Arc;

#[test]
fn test_leaf_f64_eq() {
    let node1 = utils_tests_node::create_leaf_f64(1.0);
    let node2 = node1.clone();
    assert_eq!(node1, node2);
}

#[test]
fn test_leaf_f64_ne_dist() {
    let node1 = utils_tests_node::create_leaf_f64(1.0);
    let node2 = utils_tests_node::create_leaf_f64(2.0);
    assert_ne!(node1, node2);
}

#[test]
fn test_leaf_bool_eq() {
    let node1 = utils_tests_node::create_leaf_bool(true);
    let node2 = node1.clone();
    assert_eq!(node1, node2);
}
#[test]
fn test_leaf_bool_ne_dist() {
    let node1 = utils_tests_node::create_leaf_bool(true);
    let node2 = utils_tests_node::create_leaf_bool(false);
    assert_ne!(node1, node2);
}

#[test]
fn test_arithmetic_op_eq() {
    let lhs = utils_tests_node::create_leaf_f64(1.0);
    let rhs = utils_tests_node::create_leaf_f64(2.0);
    let node1 =
        utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs.clone(), rhs.clone());
    let node2 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs, rhs);
    assert_eq!(node1, node2);
}

#[test]
fn test_arithmetic_op_ne_op() {
    let lhs = utils_tests_node::create_leaf_f64(1.0);
    let rhs = utils_tests_node::create_leaf_f64(2.0);
    let node1 =
        utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs.clone(), rhs.clone());
    let node2 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Sub, lhs, rhs);
    assert_ne!(node1, node2);
}

#[test]
fn test_arithmetic_op_ne_lhs() {
    let lhs1 = utils_tests_node::create_leaf_f64(1.0);
    let lhs2 = utils_tests_node::create_leaf_f64(3.0);
    let rhs = utils_tests_node::create_leaf_f64(2.0);
    let node1 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs1, rhs.clone());
    let node2 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs2, rhs);
    assert_ne!(node1, node2);
}

#[test]
fn test_arithmetic_op_ne_rhs() {
    let lhs = utils_tests_node::create_leaf_f64(1.0);
    let rhs1 = utils_tests_node::create_leaf_f64(2.0);
    let rhs2 = utils_tests_node::create_leaf_f64(4.0);
    let node1 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs.clone(), rhs1);
    let node2 = utils_tests_node::create_arithmetic_op(ArithmeticOperator::Add, lhs, rhs2);
    assert_ne!(node1, node2);
}

#[test]
fn test_comparison_op_eq() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = utils_tests_node::create_comparison_op(
        ComparisonOperator::GreaterThan,
        0.5,
        operand.clone(),
    );
    let node2 =
        utils_tests_node::create_comparison_op(ComparisonOperator::GreaterThan, 0.5, operand);
    assert_eq!(node1, node2);
}

#[test]
fn test_comparison_op_ne_op() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = utils_tests_node::create_comparison_op(
        ComparisonOperator::GreaterThan,
        0.5,
        operand.clone(),
    );
    let node2 = utils_tests_node::create_comparison_op(ComparisonOperator::LessThan, 0.5, operand);
    assert_ne!(node1, node2);
}

#[test]
fn test_comparison_op_ne_threshold() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = utils_tests_node::create_comparison_op(
        ComparisonOperator::GreaterThan,
        0.5,
        operand.clone(),
    );
    let node2 =
        utils_tests_node::create_comparison_op(ComparisonOperator::GreaterThan, 0.6, operand);
    assert_ne!(node1, node2);
}

#[test]
fn test_comparison_op_ne_operand() {
    let operand1 = utils_tests_node::create_leaf_f64(1.0);
    let operand2 = utils_tests_node::create_leaf_f64(2.0);
    let node1 =
        utils_tests_node::create_comparison_op(ComparisonOperator::GreaterThan, 0.5, operand1);
    let node2 =
        utils_tests_node::create_comparison_op(ComparisonOperator::GreaterThan, 0.5, operand2);
    assert_ne!(node1, node2);
}

#[test]
fn test_logical_op_eq() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let node1 =
        utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1.clone(), op2.clone()]);
    let node2 = utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1, op2]);
    assert_eq!(node1, node2);
}

#[test]
fn test_logical_op_ne_op() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let node1 =
        utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1.clone(), op2.clone()]);
    let node2 = utils_tests_node::create_logical_op(LogicalOperator::Or, vec![op1, op2]);
    assert_ne!(node1, node2);
}

#[test]
fn test_logical_op_ne_operands_len() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let node1 =
        utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1.clone(), op2.clone()]);
    let node2 = utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1]);
    assert_ne!(node1, node2);
}

#[test]
fn test_logical_op_ne_operands_content() {
    let op1 = utils_tests_node::create_leaf_bool(true);
    let op2 = utils_tests_node::create_leaf_bool(false);
    let op3 = utils_tests_node::create_leaf_bool(true);
    let node1 = utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1.clone(), op2]);
    let node2 = utils_tests_node::create_logical_op(LogicalOperator::And, vec![op1, op3]);
    assert_ne!(node1, node2);
}

#[test]
fn test_function_op_f64_eq() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = utils_tests_node::create_function_op_f64(operand.clone());
    let node2 = utils_tests_node::create_function_op_f64(operand);
    assert_eq!(node1, node2);
}

#[test]
fn test_function_op_f64_ne_operand() {
    let operand1 = utils_tests_node::create_leaf_f64(1.0);
    let operand2 = utils_tests_node::create_leaf_f64(2.0);
    let node1 = utils_tests_node::create_function_op_f64(operand1);
    let node2 = utils_tests_node::create_function_op_f64(operand2);
    assert_ne!(node1, node2);
}

#[test]
fn test_function_op_f64_eq_diff_func() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = ComputationNode::FunctionOpF64 {
        node_id: NodeId::new(),
        func: Arc::new(|x| x * 2.0),
        operand: Box::new(operand.clone()),
    };
    let node2 = ComputationNode::FunctionOpF64 {
        node_id: node1.id(),         // Same ID
        func: Arc::new(|x| x + 1.0), // Different func
        operand: Box::new(operand),
    };
    assert_eq!(node1, node2);
}

#[test]
fn test_function_op_bool_eq() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = utils_tests_node::create_function_op_bool(operand.clone());
    let node2 = utils_tests_node::create_function_op_bool(operand);
    assert_eq!(node1, node2);
}

#[test]
fn test_function_op_bool_ne_operand() {
    let operand1 = utils_tests_node::create_leaf_f64(1.0);
    let operand2 = utils_tests_node::create_leaf_f64(2.0);
    let node1 = utils_tests_node::create_function_op_bool(operand1);
    let node2 = utils_tests_node::create_function_op_bool(operand2);
    assert_ne!(node1, node2);
}

#[test]
fn test_function_op_bool_eq_diff_func() {
    let operand = utils_tests_node::create_leaf_f64(1.0);
    let node1 = ComputationNode::FunctionOpBool {
        node_id: NodeId::new(),
        func: Arc::new(|x| x > 0.5),
        operand: Box::new(operand.clone()),
    };
    let node2 = ComputationNode::FunctionOpBool {
        node_id: node1.id(),         // Same ID
        func: Arc::new(|x| x < 0.5), // Different func
        operand: Box::new(operand),
    };
    assert_eq!(node1, node2);
}

#[test]
fn test_negation_op_eq() {
    let operand = utils_tests_node::create_leaf_bool(true);
    let node1 = utils_tests_node::create_negation_op(operand.clone());
    let node2 = utils_tests_node::create_negation_op(operand);
    assert_eq!(node1, node2);
}

#[test]
fn test_negation_op_ne_operand() {
    let operand1 = utils_tests_node::create_leaf_bool(true);
    let operand2 = utils_tests_node::create_leaf_bool(false);
    let node1 = utils_tests_node::create_negation_op(operand1);
    let node2 = utils_tests_node::create_negation_op(operand2);
    assert_ne!(node1, node2);
}

#[test]
fn test_conditional_op_eq() {
    let condition = utils_tests_node::create_leaf_bool(true);
    let if_true = utils_tests_node::create_leaf_f64(1.0);
    let if_false = utils_tests_node::create_leaf_f64(0.0);
    let node1 = utils_tests_node::create_conditional_op(
        condition.clone(),
        if_true.clone(),
        if_false.clone(),
    );
    let node2 = utils_tests_node::create_conditional_op(condition, if_true, if_false);
    assert_eq!(node1, node2);
}

#[test]
fn test_conditional_op_ne_if_true() {
    let condition = utils_tests_node::create_leaf_bool(true);
    let if_true1 = utils_tests_node::create_leaf_f64(1.0);
    let if_true2 = utils_tests_node::create_leaf_f64(2.0);
    let if_false = utils_tests_node::create_leaf_f64(0.0);
    let node1 =
        utils_tests_node::create_conditional_op(condition.clone(), if_true1, if_false.clone());
    let node2 = utils_tests_node::create_conditional_op(condition, if_true2, if_false);
    assert_ne!(node1, node2);
}

#[test]
fn test_conditional_op_ne_if_false() {
    let condition = utils_tests_node::create_leaf_bool(true);
    let if_true = utils_tests_node::create_leaf_f64(1.0);
    let if_false1 = utils_tests_node::create_leaf_f64(0.0);
    let if_false2 = utils_tests_node::create_leaf_f64(3.0);
    let node1 =
        utils_tests_node::create_conditional_op(condition.clone(), if_true.clone(), if_false1);
    let node2 = utils_tests_node::create_conditional_op(condition, if_true, if_false2);
    assert_ne!(node1, node2);
}

#[test]
fn test_different_node_types_ne() {
    let node1 = utils_tests_node::create_leaf_f64(1.0);
    let node2 = utils_tests_node::create_leaf_bool(true);
    assert_ne!(node1, node2);

    let node3 = utils_tests_node::create_arithmetic_op(
        ArithmeticOperator::Add,
        utils_tests_node::create_leaf_f64(1.0),
        utils_tests_node::create_leaf_f64(2.0),
    );
    assert_ne!(node1, node3);
    assert_ne!(node2, node3);

    let node4 = utils_tests_node::create_comparison_op(
        ComparisonOperator::GreaterThan,
        0.5,
        utils_tests_node::create_leaf_f64(1.0),
    );
    assert_ne!(node1, node4);
    assert_ne!(node3, node4);

    let node5 = utils_tests_node::create_logical_op(
        LogicalOperator::And,
        vec![utils_tests_node::create_leaf_bool(true)],
    );
    assert_ne!(node1, node5);
    assert_ne!(node4, node5);

    let node6 = utils_tests_node::create_function_op_f64(utils_tests_node::create_leaf_f64(1.0));
    assert_ne!(node1, node6);
    assert_ne!(node5, node6);

    let node7 = utils_tests_node::create_function_op_bool(utils_tests_node::create_leaf_f64(1.0));
    assert_ne!(node6, node7);

    let node8 = utils_tests_node::create_negation_op(utils_tests_node::create_leaf_bool(true));
    assert_ne!(node7, node8);

    let node9 = utils_tests_node::create_conditional_op(
        utils_tests_node::create_leaf_bool(true),
        utils_tests_node::create_leaf_f64(1.0),
        utils_tests_node::create_leaf_f64(0.0),
    );
    assert_ne!(node8, node9);
}
