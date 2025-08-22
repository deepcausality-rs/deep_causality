/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DeonticError, TeloidGraph};
use ultragraph::{GraphMut, GraphView};

impl TeloidGraph {
    pub fn number_nodes(&self) -> usize {
        self.graph.number_nodes()
    }

    pub fn number_edges(&self) -> usize {
        self.graph.number_edges()
    }

    pub fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.graph.contains_edge(a, b)
    }

    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    pub fn clear(&mut self) -> Result<(), DeonticError> {
        self.graph.clear().map_err(DeonticError::from)
    }
}

impl TeloidGraph {
    pub fn is_frozen(&self) -> bool {
        self.graph.is_frozen()
    }

    pub fn freeze(&mut self) {
        self.graph.freeze()
    }

    pub fn unfreeze(&mut self) {
        self.graph.unfreeze()
    }
}
