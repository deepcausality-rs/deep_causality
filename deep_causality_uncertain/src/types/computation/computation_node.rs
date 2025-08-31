/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ArithmeticOperator, ComparisonOperator, Distribution, LogicalOperator};

/// Represents a node in the computation graph with type-safe operations.
#[derive(Debug, Clone, Copy)]
pub enum ComputationNode {
    /// A leaf node representing a source of uncertainty from a distribution.
    Leaf { dist: Distribution },
    /// An intermediate node representing a binary arithmetic operation.
    ArithmeticOp { op: ArithmeticOperator },
    /// An intermediate node representing a binary comparison operation.
    ComparisonOp { op: ComparisonOperator },
    /// An intermediate node representing a logical operation.
    LogicalOp { op: LogicalOperator },
}
