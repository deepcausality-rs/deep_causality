/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::{BitAnd, BitOr, BitXor, Not};

use crate::{LogicalOperator, Node, UncertainBool};
use deep_causality_rand::RandScalar;

// The logical operators belong to the Boolean carrier and to nothing else. Under the closed
// dispatcher they were `impl BitAnd for Uncertain<bool>` — a concrete instantiation, which is the
// only way to keep them off the real instantiations. Here the type does it, so they are one
// blanket impl each and every scalar gets them.
impl<R: RandScalar> BitAnd for UncertainBool<R> {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        binary(LogicalOperator::And, self, rhs)
    }
}

impl<R: RandScalar> BitOr for UncertainBool<R> {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        binary(LogicalOperator::Or, self, rhs)
    }
}

impl<R: RandScalar> BitXor for UncertainBool<R> {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        binary(LogicalOperator::XOR, self, rhs)
    }
}

impl<R: RandScalar> Not for UncertainBool<R> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::from_root_node(Node::LogicalOp {
            op: LogicalOperator::Not,
            operands: vec![self.into_tree()],
        })
    }
}

/// One binary logical node.
pub(crate) fn binary<R: RandScalar>(
    op: LogicalOperator,
    lhs: UncertainBool<R>,
    rhs: UncertainBool<R>,
) -> UncertainBool<R> {
    UncertainBool::from_root_node(Node::LogicalOp {
        op,
        operands: vec![lhs.into_tree(), rhs.into_tree()],
    })
}
