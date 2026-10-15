/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DeonticError, TeloidGraph, TeloidRelation};
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

impl TeloidGraph {
    /// The relation the edge from `from` to `to` carries, or `None` when there is no such edge.
    /// The read counterpart of `add_inheritance_edge` and `add_defeasance_edge`.
    pub fn relation(&self, from: usize, to: usize) -> Option<TeloidRelation> {
        self.graph
            .get_edges(from)
            .and_then(|edges| edges.into_iter().find(|(target, _)| *target == to))
            .map(|(_, carried)| *carried)
    }

    /// Sets the pair's relation. A pair carries one relation: adding the one it already carries
    /// leaves it as it is, and adding the other is refused, since a norm cannot both inherit
    /// from and defeat another. A frozen graph refuses every addition before the pair is read.
    pub(crate) fn add_relation(
        &mut self,
        from: usize,
        to: usize,
        relation: TeloidRelation,
    ) -> Result<(), DeonticError> {
        if self.graph.is_frozen() {
            return Err(DeonticError::GraphIsFrozen(
                ultragraph::GraphError::GraphIsFrozen,
            ));
        }
        if self
            .relation(from, to)
            .is_some_and(|carried| carried != relation)
        {
            return Err(DeonticError::ConflictingRelation { from, to });
        }
        self.graph
            .add_edge(from, to, relation)
            .map_err(DeonticError::from)
    }
}
