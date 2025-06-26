/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{GraphStorage, UltraGraphContainer};

impl<S, T> GraphStorage<T> for UltraGraphContainer<S, T>
where
    S: GraphStorage<T>,
{
    fn size(&self) -> usize {
        self.storage.size()
    }

    fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    fn number_nodes(&self) -> usize {
        self.storage.number_nodes()
    }

    fn number_edges(&self) -> usize {
        self.storage.number_edges()
    }

    fn get_all_nodes(&self) -> Vec<&T> {
        self.storage.get_all_nodes()
    }

    fn get_all_edges(&self) -> Vec<(usize, usize)> {
        self.storage.get_all_edges()
    }

    fn clear(&mut self) {
        self.storage.clear()
    }
}
