/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::computation::ArithmeticOperator;
use crate::{ComputationNode, Uncertain};
use std::ops::{Add, Div, Mul, Neg, Sub};

// Operator overloading is only implemented for f64 for now.
impl Add for Uncertain<f64> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Add,
            lhs: Box::new((*self.root_node).clone()),
            rhs: Box::new((*rhs.root_node).clone()),
        })
    }
}

impl Sub for Uncertain<f64> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Sub,
            lhs: Box::new((*self.root_node).clone()),
            rhs: Box::new((*rhs.root_node).clone()),
        })
    }
}

impl Mul for Uncertain<f64> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Mul,
            lhs: Box::new((*self.root_node).clone()),
            rhs: Box::new((*rhs.root_node).clone()),
        })
    }
}

impl Div for Uncertain<f64> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::ArithmeticOp {
            op: ArithmeticOperator::Div,
            lhs: Box::new((*self.root_node).clone()),
            rhs: Box::new((*rhs.root_node).clone()),
        })
    }
}

impl Neg for Uncertain<f64> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from_root_node(ComputationNode::NegationOp {
            operand: Box::new((*self.root_node).clone()),
        })
    }
}
