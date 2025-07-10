/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::graph_algo_topological::TopologicalGraphAlgorithms;
use crate::{CentralityGraphAlgorithms, PathfindingGraphAlgorithms, StructuralGraphAlgorithms};

/// A comprehensive suite of graph algorithms.
///
/// This trait aggregates several focused algorithm traits into a single, convenient
/// supertrait. A type that implements `GraphAlgorithms` has access to all methods
/// from the component traits.
pub trait GraphAlgorithms<N, W>:
    TopologicalGraphAlgorithms<N, W>
    + PathfindingGraphAlgorithms<N, W>
    + StructuralGraphAlgorithms<N, W>
    + CentralityGraphAlgorithms<N, W>
{
}
