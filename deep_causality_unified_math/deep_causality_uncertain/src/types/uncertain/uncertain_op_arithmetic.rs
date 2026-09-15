/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ArithmeticOperator, Node, Uncertain};
use deep_causality_rand::RandScalar;
use std::ops::{Add, Div, Mul, Neg, Sub};

// Arithmetic on the real carrier. The nodes are scalar-agnostic — they only thread `ConstTree`s —
// and the scalar arrives with the graph, so there is one impl per operator rather than one per
// precision. The Boolean carrier is a different type and does not get these, which is what the
// `RealField` bound used to be doing and now the type does.
impl<R: RandScalar> Add for Uncertain<R> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        binary(ArithmeticOperator::Add, self, rhs)
    }
}

impl<R: RandScalar> Sub for Uncertain<R> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        binary(ArithmeticOperator::Sub, self, rhs)
    }
}

impl<R: RandScalar> Mul for Uncertain<R> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        binary(ArithmeticOperator::Mul, self, rhs)
    }
}

impl<R: RandScalar> Div for Uncertain<R> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        binary(ArithmeticOperator::Div, self, rhs)
    }
}

impl<R: RandScalar> Neg for Uncertain<R> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_root_node(Node::NegationOp {
            operand: self.into_tree(),
        })
    }
}

/// One binary arithmetic node.
pub(crate) fn binary<R: RandScalar>(
    op: ArithmeticOperator,
    lhs: Uncertain<R>,
    rhs: Uncertain<R>,
) -> Uncertain<R> {
    Uncertain::from_root_node(Node::ArithmeticOp {
        op,
        lhs: lhs.into_tree(),
        rhs: rhs.into_tree(),
    })
}
