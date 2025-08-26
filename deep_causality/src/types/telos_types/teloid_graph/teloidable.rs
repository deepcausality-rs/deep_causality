/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DeonticError, TeloidGraph, TeloidID, TeloidRelation, Teloidable};
use ultragraph::{GraphMut, GraphView};

impl Teloidable for TeloidGraph {
    /// Adds a new teloid (node) to the graph.
    ///
    /// # Arguments
    ///
    /// * `id` - The `TeloidID` of the teloid to add.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(usize)` containing the index of the newly added teloid,
    /// or `Err(GraphError)` if the operation fails.
    fn add_teloid(&mut self, id: TeloidID) -> Result<usize, DeonticError> {
        self.graph.add_node(id).map_err(DeonticError::from)
    }

    /// Retrieves the `TeloidID` associated with a given node index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the teloid (node) to retrieve the ID for.
    ///
    /// # Returns
    ///
    /// An `Option<TeloidID>` which is `Some(TeloidID)` if the teloid exists at the given index,
    /// or `None` otherwise.
    fn get_teloid_id(&self, index: usize) -> Option<TeloidID> {
        self.graph.get_node(index).copied()
    }

    /// Checks if a teloid (node) exists at the given index in the graph.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to check for the presence of a teloid.
    ///
    /// # Returns
    ///
    /// `true` if a teloid exists at the specified index, `false` otherwise.
    fn contains_teloid(&self, index: usize) -> bool {
        self.graph.contains_node(index)
    }

    /// Adds an inheritance edge between a parent teloid and a child teloid.
    ///
    /// # Arguments
    ///
    /// * `parent_idx` - The index of the parent teloid.
    /// * `child_idx` - The index of the child teloid.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the edge was added successfully,
    /// or `Err(GraphError)` if the operation fails.
    fn add_inheritance_edge(
        &mut self,
        parent_idx: usize,
        child_idx: usize,
    ) -> Result<(), DeonticError> {
        self.graph
            .add_edge(parent_idx, child_idx, TeloidRelation::Inherits)
            .map_err(DeonticError::from)
    }

    /// Adds a defeasance edge between a defeater teloid and a defeated teloid.
    ///
    /// # Arguments
    ///
    /// * `defeater_idx` - The index of the defeater teloid.
    /// * `defeated_idx` - The index of the defeated teloid.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` if the edge was added successfully,
    /// or `Err(GraphError)` if the operation fails.
    fn add_defeasance_edge(
        &mut self,
        defeater_idx: usize,
        defeated_idx: usize,
    ) -> Result<(), DeonticError> {
        self.graph
            .add_edge(defeater_idx, defeated_idx, TeloidRelation::Defeats)
            .map_err(DeonticError::from)
    }
}
