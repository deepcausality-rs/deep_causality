/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Uncertain;
use crate::types::computation::{ComputationNode, LogicalOperator};
use std::ops::{BitAnd, BitOr, Not};

impl BitAnd for Uncertain<bool> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::LogicalOp {
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
            op: LogicalOperator::Not,
            operands: vec![Box::new((*self.root_node).clone())],
        })
    }
}
