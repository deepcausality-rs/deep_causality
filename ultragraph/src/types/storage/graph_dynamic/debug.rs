/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt;

use crate::{DynamicGraph, EdgeKey};

/// Prints the nodes, the forward edges, the root and the edge count. The backward lists hold
/// edge keys, which the `EdgeKey` trait does not require to be `Debug`, so they are left out.
impl<N, W> fmt::Debug for DynamicGraph<N, W>
where
    N: fmt::Debug,
    W: fmt::Debug + Clone + Default + EdgeKey,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynamicGraph")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .field("root_index", &self.root_index)
            .field("live_edges", &self.live_edges)
            .finish()
    }
}
