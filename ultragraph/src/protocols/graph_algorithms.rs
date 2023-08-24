// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::vec::IntoIter;

use crate::errors::UltraGraphError;
use crate::prelude::GraphLike;

pub trait GraphAlgorithms<T>: GraphLike<T> {
    /// Returns the path of subsequent NodeId from start to finish, if one was found.
    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>>;

    /// Returns all nodes with an outgoing edge starting from a.
    fn outgoing_edges(&self, a: usize) -> Result<IntoIter<usize>, UltraGraphError>;
}
