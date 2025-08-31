/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::computation::computation_operator::{
    ArithmeticOperator, ComparisonOperator, LogicalOperator,
};

/// Represents a node in the computation graph. This is now a single, non-generic enum.
#[derive(Debug, Clone, Copy)]
pub enum ComputationNode {
    // Leaf nodes now contain the specific distribution type directly.
    LeafF64(crate::DistributionEnum<f64>),
    LeafBool(crate::DistributionEnum<bool>),

    ArithmeticOp {
        op: ArithmeticOperator,
    },
    ComparisonOp {
        op: ComparisonOperator,
        threshold: f64,
    },
    LogicalOp {
        op: LogicalOperator,
    },
}
