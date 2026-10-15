/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EdgeKey, GraphError, GraphView};

pub trait GraphMut<N, W>: GraphView<N, W>
where
    W: EdgeKey,
{
    fn add_node(&mut self, node: N) -> Result<usize, GraphError>;
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError>;

    fn remove_node(&mut self, index: usize) -> Result<(), GraphError>;

    // Edge Mutation
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError>;
    /// Removes every edge from `a` to `b`, whatever its key.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError>;
    /// Removes the edge from `a` to `b` with key `key` alone.
    fn remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) -> Result<(), GraphError>;

    // Root Node Mutation
    fn add_root_node(&mut self, node: N) -> Result<usize, GraphError>;

    // Graph-wide Mutation
    fn clear(&mut self) -> Result<(), GraphError>;
}
