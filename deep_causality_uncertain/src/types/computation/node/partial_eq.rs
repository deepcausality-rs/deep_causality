/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ComputationNode;

impl PartialEq for ComputationNode {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ComputationNode::LeafF64 { node_id: _, dist } => {
                matches!(other, ComputationNode::LeafF64 { node_id: _, dist: other_dist } if dist == other_dist)
            }

            ComputationNode::LeafBool { node_id: _, dist } => {
                matches!(other, ComputationNode::LeafBool { node_id: _, dist: other_dist } if dist == other_dist)
            }

            ComputationNode::ArithmeticOp {
                node_id: _,
                op,
                lhs,
                rhs,
            } => match other {
                ComputationNode::ArithmeticOp {
                    node_id: _,
                    op: other_op,
                    lhs: other_lhs,
                    rhs: other_rhs,
                } => op == other_op && lhs == other_lhs && rhs == other_rhs,
                _ => false,
            },
            ComputationNode::ComparisonOp {
                node_id: _,
                op,
                threshold,
                operand,
            } => match other {
                ComputationNode::ComparisonOp {
                    node_id: _,
                    op: other_op,
                    threshold: other_threshold,
                    operand: other_operand,
                } => op == other_op && threshold == other_threshold && operand == other_operand,
                _ => false,
            },
            ComputationNode::LogicalOp {
                node_id: _,
                op,
                operands,
            } => match other {
                ComputationNode::LogicalOp {
                    node_id: _,
                    op: other_op,
                    operands: other_operands,
                } => op == other_op && operands == other_operands,
                _ => false,
            },
            ComputationNode::FunctionOpF64 {
                node_id: _,
                func: _,
                operand,
            } => match other {
                ComputationNode::FunctionOpF64 {
                    node_id: _,
                    func: _, // dyn Fn(f64) -> f64 does not implement PartialEq
                    operand: other_operand,
                } => operand == other_operand,
                _ => false,
            },

            ComputationNode::FunctionOpBool {
                node_id: _,
                func: _,
                operand,
            } => {
                match other {
                    ComputationNode::FunctionOpBool {
                        node_id: _,
                        func: _, // dyn Fn(f64) -> bool does not implement PartialEq
                        operand: other_operand,
                    } => operand == other_operand,
                    _ => false,
                }
            }
            ComputationNode::NegationOp {
                node_id: _,
                operand,
            } => match other {
                ComputationNode::NegationOp {
                    node_id: _,
                    operand: other_operand,
                } => operand == other_operand,
                _ => false,
            },
            ComputationNode::ConditionalOp {
                node_id: _,
                condition,
                if_true,
                if_false,
            } => match other {
                ComputationNode::ConditionalOp {
                    node_id: _,
                    condition: other_condition,
                    if_true: other_if_true,
                    if_false: other_if_false,
                } => {
                    condition == other_condition
                        && if_true == other_if_true
                        && if_false == other_if_false
                }
                _ => false,
            },
        }
    }
}
