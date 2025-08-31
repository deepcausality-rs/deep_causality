/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DistributionEnum, Operator};

/// Represents a node in the computation graph.
#[derive(Debug, Clone, Copy)]
pub enum ComputationNode {
    /// A leaf node representing a source of uncertainty from a distribution.
    Leaf { dist: DistributionEnum },
    /// An intermediate node representing a binary arithmetic operation.
    BinaryOp { op: Operator },
}
