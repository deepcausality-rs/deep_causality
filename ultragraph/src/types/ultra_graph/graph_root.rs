// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{GraphRoot, GraphStorage, UltraGraphContainer, UltraGraphError};

impl<S, T> GraphRoot<T> for UltraGraphContainer<S, T>
where
    S: GraphStorage<T>,
{
    fn add_root_node(&mut self, value: T) -> usize {
        self.storage.add_root_node(value)
    }

    fn contains_root_node(&self) -> bool {
        self.storage.contains_root_node()
    }

    fn get_root_node(&self) -> Option<&T> {
        self.storage.get_root_node()
    }

    fn get_root_index(&self) -> Option<usize> {
        self.storage.get_root_index()
    }

    fn get_last_index(&self) -> Result<usize, UltraGraphError> {
        self.storage.get_last_index()
    }
}
