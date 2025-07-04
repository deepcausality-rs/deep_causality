/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DynamicGraph;
use crate::types::storage::graph_state::GraphState;

// This implementation is required for the pattern below.
// The default state of a graph is an empty, dynamic one.
impl<N, W: Default> Default for GraphState<N, W> {
    fn default() -> Self {
        GraphState::Dynamic(DynamicGraph::new())
    }
}
