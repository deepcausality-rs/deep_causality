/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ComputationNode, NodeId};

impl ComputationNode {
    pub fn id(&self) -> NodeId {
        match self {
            ComputationNode::LeafF64 { node_id, dist: _ } => *node_id,
            ComputationNode::LeafBool { node_id, dist: _ } => *node_id,
            ComputationNode::ArithmeticOp {
                node_id,
                op: _,
                lhs: _,
                rhs: _,
            } => *node_id,
            ComputationNode::ComparisonOp {
                node_id,
                op: _,
                threshold: _,
                operand: _,
            } => *node_id,
            ComputationNode::LogicalOp {
                node_id,
                op: _,
                operands: _,
            } => *node_id,

            ComputationNode::FunctionOpF64 {
                node_id,
                func: _,
                operand: _,
            } => *node_id,
            ComputationNode::FunctionOpBool {
                node_id,
                func: _,
                operand: _,
            } => *node_id,
            ComputationNode::NegationOp {
                node_id,
                operand: _,
            } => *node_id,
            ComputationNode::ConditionalOp {
                node_id,
                condition: _,
                if_true: _,
                if_false: _,
            } => *node_id,
        }
    }
}
