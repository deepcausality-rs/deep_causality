// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::vec::IntoIter;

use crate::prelude::{GraphAlgorithms, GraphStorage, UltraGraphContainer, UltraGraphError};

impl<S, T> GraphAlgorithms<T> for UltraGraphContainer<S, T>
    where S: GraphStorage<T>
{
    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>>
    {
        self.storage.shortest_path(start_index, stop_index)
    }

    fn outgoing_edges(&self, a: usize) -> Result<IntoIter<usize>, UltraGraphError> {
        self.storage.outgoing_edges(a)
    }
}
