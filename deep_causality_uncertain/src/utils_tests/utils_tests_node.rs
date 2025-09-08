/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ArithmeticOperator, ComparisonOperator, ComputationNode, DistributionEnum, LogicalOperator,
    NodeId,
};
use std::sync::Arc;

// Helper functions to create ComputationNode variants
pub fn create_leaf_f64(value: f64) -> ComputationNode {
    ComputationNode::LeafF64 {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(value),
    }
}

pub fn create_leaf_bool(value: bool) -> ComputationNode {
    ComputationNode::LeafBool {
        node_id: NodeId::new(),
        dist: DistributionEnum::Point(value),
    }
}

pub fn create_arithmetic_op(
    op: ArithmeticOperator,
    lhs: ComputationNode,
    rhs: ComputationNode,
) -> ComputationNode {
    ComputationNode::ArithmeticOp {
        node_id: NodeId::new(),
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

pub fn create_comparison_op(
    op: ComparisonOperator,
    threshold: f64,
    operand: ComputationNode,
) -> ComputationNode {
    ComputationNode::ComparisonOp {
        node_id: NodeId::new(),
        op,
        threshold,
        operand: Box::new(operand),
    }
}

pub fn create_logical_op(op: LogicalOperator, operands: Vec<ComputationNode>) -> ComputationNode {
    ComputationNode::LogicalOp {
        node_id: NodeId::new(),
        op,
        operands: operands.into_iter().map(Box::new).collect(),
    }
}

pub fn create_function_op_f64(operand: ComputationNode) -> ComputationNode {
    ComputationNode::FunctionOpF64 {
        node_id: NodeId::new(),
        func: Arc::new(|x| x * 2.0),
        operand: Box::new(operand),
    }
}

pub fn create_function_op_bool(operand: ComputationNode) -> ComputationNode {
    ComputationNode::FunctionOpBool {
        node_id: NodeId::new(),
        func: Arc::new(|x| x > 0.5),
        operand: Box::new(operand),
    }
}

pub fn create_negation_op(operand: ComputationNode) -> ComputationNode {
    ComputationNode::NegationOp {
        node_id: NodeId::new(),
        operand: Box::new(operand),
    }
}

pub fn create_conditional_op(
    condition: ComputationNode,
    if_true: ComputationNode,
    if_false: ComputationNode,
) -> ComputationNode {
    ComputationNode::ConditionalOp {
        node_id: NodeId::new(),
        condition: Box::new(condition),
        if_true: Box::new(if_true),
        if_false: Box::new(if_false),
    }
}
