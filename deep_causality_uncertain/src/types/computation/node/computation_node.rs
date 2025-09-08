/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ArithmeticOperator, ComparisonOperator, DistributionEnum, LogicalOperator, NodeId};
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

pub static NEXT_NODE_ID: AtomicUsize = AtomicUsize::new(0);

/// Represents a node in the computation graph. This is now a single, non-generic enum.
#[derive(Clone)]
pub enum ComputationNode {
    // Leaf nodes now contain the specific distribution type directly.
    LeafF64 {
        node_id: NodeId,
        dist: DistributionEnum<f64>,
    },
    LeafBool {
        node_id: NodeId,
        dist: DistributionEnum<bool>,
    },

    ArithmeticOp {
        node_id: NodeId,
        op: ArithmeticOperator,
        lhs: Box<ComputationNode>,
        rhs: Box<ComputationNode>,
    },
    ComparisonOp {
        node_id: NodeId,
        op: ComparisonOperator,
        threshold: f64,
        operand: Box<ComputationNode>,
    },
    LogicalOp {
        node_id: NodeId,
        op: LogicalOperator,
        operands: Vec<Box<ComputationNode>>,
    },
    FunctionOpF64 {
        node_id: NodeId,
        func: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
        operand: Box<ComputationNode>,
    },
    FunctionOpBool {
        node_id: NodeId,
        func: Arc<dyn Fn(f64) -> bool + Send + Sync>,
        operand: Box<ComputationNode>,
    },
    NegationOp {
        node_id: NodeId,
        operand: Box<ComputationNode>,
    },
    ConditionalOp {
        node_id: NodeId,
        condition: Box<ComputationNode>,
        if_true: Box<ComputationNode>,
        if_false: Box<ComputationNode>,
    },
}
