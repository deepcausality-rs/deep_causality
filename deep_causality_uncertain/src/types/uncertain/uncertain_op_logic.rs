/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::{BitAnd, BitOr, BitXor, Not};

use crate::{ComputationNode, LogicalOperator, NodeId, Uncertain};

impl BitAnd for Uncertain<bool> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::LogicalOp {
            node_id: NodeId::new(), // Added node_id
            op: LogicalOperator::And,
            operands: vec![
                Box::new((*self.root_node).clone()),
                Box::new((*rhs.root_node).clone()),
            ],
        })
    }
}

impl BitOr for Uncertain<bool> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::LogicalOp {
            node_id: NodeId::new(), // Added node_id
            op: LogicalOperator::Or,
            operands: vec![
                Box::new((*self.root_node).clone()),
                Box::new((*rhs.root_node).clone()),
            ],
        })
    }
}

impl Not for Uncertain<bool> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::from_root_node(ComputationNode::LogicalOp {
            node_id: NodeId::new(), // Added node_id
            op: LogicalOperator::Not,
            operands: vec![Box::new((*self.root_node).clone())],
        })
    }
}

impl BitXor for Uncertain<bool> {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::LogicalOp {
            node_id: NodeId::new(), // Added node_id
            op: LogicalOperator::XOR,
            operands: vec![
                Box::new((*self.root_node).clone()),
                Box::new((*rhs.root_node).clone()),
            ],
        })
    }
}
