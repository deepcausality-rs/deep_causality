/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GraphError, GraphView};

pub trait StructuralGraphAlgorithms<N, W>: GraphView<N, W> {
    /// Finds all Strongly Connected Components in the graph.
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError>;
}
