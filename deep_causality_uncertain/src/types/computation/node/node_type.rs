/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ComputationNode;

impl ComputationNode {
    pub fn is_leaf_64(&self) -> bool {
        matches!(
            self,
            ComputationNode::LeafF64 {
                node_id: _,
                dist: _
            }
        )
    }

    pub fn is_leaf_bool(&self) -> bool {
        matches!(
            self,
            ComputationNode::LeafBool {
                node_id: _,
                dist: _
            }
        )
    }

    pub fn is_arithmetic_op(&self) -> bool {
        matches!(
            self,
            ComputationNode::ArithmeticOp {
                node_id: _,
                op: _,
                lhs: _,
                rhs: _,
            }
        )
    }

    pub fn is_comparison_op(&self) -> bool {
        matches!(
            self,
            ComputationNode::ComparisonOp {
                node_id: _,
                op: _,
                threshold: _,
                operand: _,
            }
        )
    }

    pub fn is_logical_op(&self) -> bool {
        matches!(
            self,
            ComputationNode::LogicalOp {
                node_id: _,
                op: _,
                operands: _
            }
        )
    }

    pub fn is_function_op_f64(&self) -> bool {
        matches!(
            self,
            ComputationNode::FunctionOpF64 {
                node_id: _,
                func: _,
                operand: _,
            }
        )
    }

    pub fn is_function_op_bool(&self) -> bool {
        matches!(
            self,
            ComputationNode::FunctionOpBool {
                node_id: _,
                func: _,
                operand: _,
            }
        )
    }

    pub fn is_negation_op(&self) -> bool {
        matches!(
            self,
            ComputationNode::NegationOp {
                node_id: _,
                operand: _,
            }
        )
    }
    pub fn is_conditional_op(&self) -> bool {
        matches!(
            self,
            ComputationNode::ConditionalOp {
                node_id: _,
                condition: _,
                if_true: _,
                if_false: _,
            }
        )
    }
}
