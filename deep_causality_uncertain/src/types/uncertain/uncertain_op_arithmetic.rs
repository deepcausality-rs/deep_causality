/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::computation::operator::arithmetic_operator::ArithmeticOperator;
use crate::{ComputationNode, Uncertain};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::types::computation::node::NodeId; // Added this import

// Operator overloading is only implemented for f64 for now.
impl Add for Uncertain<f64> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_root_node(ComputationNode::ArithmeticOp {
            node_id: NodeId::new(), // Added node_id
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
            node_id: NodeId::new(), // Added node_id
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
            node_id: NodeId::new(), // Added node_id
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
            node_id: NodeId::new(), // Added node_id
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
            node_id: NodeId::new(), // Added node_id
            operand: Box::new((*self.root_node).clone()),
        })
    }
}
