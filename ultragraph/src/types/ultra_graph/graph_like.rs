// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{GraphLike, GraphStorage, UltraGraphContainer, UltraGraphError};

impl<S, T> GraphLike<T> for UltraGraphContainer<S, T>
where
    S: GraphStorage<T>,
{
    fn add_node(&mut self, value: T) -> usize {
        self.storage.add_node(value)
    }

    fn contains_node(&self, index: usize) -> bool {
        self.storage.contains_node(index)
    }

    fn get_node(&self, index: usize) -> Option<&T> {
        self.storage.get_node(index)
    }

    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError> {
        self.storage.remove_node(index)
    }

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        self.storage.add_edge(a, b)
    }

    fn add_edge_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), UltraGraphError> {
        self.storage.add_edge_with_weight(a, b, weight)
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.storage.contains_edge(a, b)
    }

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError> {
        self.storage.remove_edge(a, b)
    }
}
