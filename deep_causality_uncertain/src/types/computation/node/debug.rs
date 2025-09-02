/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ComputationNode;
use std::fmt::{Debug, Formatter};

impl Debug for ComputationNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputationNode::LeafF64 { node_id, dist } => {
                write!(f, "LeafF64 {{ node_id: {:?}, dist: {:?} }}", node_id, dist)
            }
            ComputationNode::LeafBool { node_id, dist } => {
                write!(f, "LeafBool {{ node_id: {:?}, dist: {:?} }}", node_id, dist)
            }
            ComputationNode::ArithmeticOp {
                node_id,
                op,
                lhs,
                rhs,
            } => {
                write!(
                    f,
                    "ArithmeticOp {{ node_id: {:?}, op: {:?}, lhs: {:?}, rhs: {:?} }}",
                    node_id, op, lhs, rhs
                )
            }
            ComputationNode::ComparisonOp {
                node_id,
                op,
                threshold,
                operand,
            } => {
                write!(
                    f,
                    "ComparisonOp {{ node_id: {:?}, op: {:?}, threshold: {:?}, operand: {:?} }}",
                    node_id, op, threshold, operand
                )
            }
            ComputationNode::LogicalOp {
                node_id,
                op,
                operands,
            } => {
                write!(
                    f,
                    "LogicalOp {{ node_id: {:?}, op: {:?}, operands: {:?} }}",
                    node_id, op, operands
                )
            }
            ComputationNode::FunctionOp {
                node_id,
                func: _, // dyn Fn(f64) -> f64 does not implement Debug
                operand,
            } => {
                write!(
                    f,
                    "FunctionOp {{ node_id: {:?}, func:  Fn(f64) -> bool, operand: {:?} }}",
                    node_id, operand
                )
            }
            ComputationNode::FunctionOpBool {
                node_id,
                func: _, // dyn Fn(f64) -> bool does not implement Debug
                operand,
            } => {
                write!(
                    f,
                    "FunctionOpBool {{ node_id: {:?}, func:  Fn(f64) -> bool, operand: {:?} }}",
                    node_id, operand
                )
            }
            ComputationNode::NegationOp { node_id, operand } => {
                write!(
                    f,
                    "NegationOp {{ node_id: {:?}, operand: {:?} }}",
                    node_id, operand
                )
            }
            ComputationNode::ConditionalOp {
                node_id,
                condition,
                if_true,
                if_false,
            } => {
                write!(
                    f,
                    "ConditionalOp {{ node_id: {:?}, condition: {:?}, if_true: {:?}, if_false: {:?} }}",
                    node_id, condition, if_true, if_false
                )
            }
        }
    }
}
