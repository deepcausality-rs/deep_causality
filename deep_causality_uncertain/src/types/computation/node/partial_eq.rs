/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ComputationNode;

impl PartialEq for ComputationNode {
    fn eq(&self, other: &Self) -> bool {
        match self {
            ComputationNode::LeafF64 { .. } => {
                self.is_leaf_64() && other.is_leaf_64() && self.id() == other.id()
            }

            ComputationNode::LeafBool { .. } => {
                self.is_leaf_bool() && other.is_leaf_bool() && self.id() == other.id()
            }

            ComputationNode::ArithmeticOp {
                node_id,
                op,
                lhs,
                rhs,
            } => match other {
                ComputationNode::ArithmeticOp {
                    node_id: other_node_id,
                    op: other_op,
                    lhs: other_lhs,
                    rhs: other_rhs,
                } => {
                    node_id == other_node_id
                        && op == other_op
                        && lhs == other_lhs
                        && rhs == other_rhs
                }
                _ => false,
            },
            ComputationNode::ComparisonOp {
                node_id,
                op,
                threshold,
                operand,
            } => match other {
                ComputationNode::ComparisonOp {
                    node_id: other_node_id,
                    op: other_op,
                    threshold: other_threshold,
                    operand: other_operand,
                } => {
                    node_id == other_node_id
                        && op == other_op
                        && threshold == other_threshold
                        && operand == other_operand
                }
                _ => false,
            },
            ComputationNode::LogicalOp {
                node_id,
                op,
                operands,
            } => match other {
                ComputationNode::LogicalOp {
                    node_id: other_node,
                    op: other_op,
                    operands: other_operands,
                } => node_id == other_node && op == other_op && operands == other_operands,
                _ => false,
            },
            ComputationNode::FunctionOpF64 {
                node_id,
                func: _,
                operand,
            } => match other {
                ComputationNode::FunctionOpF64 {
                    node_id: other_node_id,
                    func: _, // dyn Fn(f64) -> f64 does not implement PartialEq
                    operand: other_operand,
                } => node_id == other_node_id && operand == other_operand,
                _ => false,
            },

            ComputationNode::FunctionOpBool {
                node_id,
                func: _,
                operand,
            } => {
                match other {
                    ComputationNode::FunctionOpBool {
                        node_id: other_node_id,
                        func: _, // dyn Fn(f64) -> bool does not implement PartialEq
                        operand: other_operand,
                    } => node_id == other_node_id && operand == other_operand,
                    _ => false,
                }
            }
            ComputationNode::NegationOp { node_id, operand } => match other {
                ComputationNode::NegationOp {
                    node_id: other_node_id,
                    operand: other_operand,
                } => node_id == other_node_id && operand == other_operand,
                _ => false,
            },
            ComputationNode::ConditionalOp {
                node_id,
                condition,
                if_true,
                if_false,
            } => match other {
                ComputationNode::ConditionalOp {
                    node_id: other_node_id,
                    condition: other_condition,
                    if_true: other_if_true,
                    if_false: other_if_false,
                } => {
                    node_id == other_node_id
                        && condition == other_condition
                        && if_true == other_if_true
                        && if_false == other_if_false
                }
                _ => false,
            },
        }
    }
}
