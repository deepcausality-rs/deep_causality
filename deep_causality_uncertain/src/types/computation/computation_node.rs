/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::computation::computation_operator::{
    ArithmeticOperator, ComparisonOperator, LogicalOperator,
};

use std::sync::Arc;

/// Represents a node in the computation graph. This is now a single, non-generic enum.
#[derive(Clone)]
pub enum ComputationNode {
    // Leaf nodes now contain the specific distribution type directly.
    LeafF64(crate::DistributionEnum<f64>),
    LeafBool(crate::DistributionEnum<bool>),

    ArithmeticOp {
        op: ArithmeticOperator,
        lhs: Box<ComputationNode>,
        rhs: Box<ComputationNode>,
    },
    ComparisonOp {
        op: ComparisonOperator,
        threshold: f64,
        operand: Box<ComputationNode>,
    },
    LogicalOp {
        op: LogicalOperator,
        operands: Vec<Box<ComputationNode>>,
    },
    FunctionOp {
        func: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
        operand: Box<ComputationNode>,
    },
    FunctionOpBool {
        func: Arc<dyn Fn(f64) -> bool + Send + Sync>,
        operand: Box<ComputationNode>,
    },
    NegationOp {
        operand: Box<ComputationNode>,
    },
    ConditionalOp {
        condition: Box<ComputationNode>,
        if_true: Box<ComputationNode>,
        if_false: Box<ComputationNode>,
    },
}
