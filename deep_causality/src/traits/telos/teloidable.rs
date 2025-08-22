/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DeonticError, TeloidID};

/// Defines the domain-specific API for a graph of Teloids.
pub trait Teloidable {
    /// Adds a Teloid to the graph by its ID.
    fn add_teloid(&mut self, id: TeloidID) -> Result<usize, DeonticError>;

    /// Gets the TeloidID of the node at a given index.
    fn get_teloid_id(&self, index: usize) -> Option<TeloidID>;

    /// Checks if a node with a given index exists in the graph.
    fn contains_teloid(&self, index: usize) -> bool;

    /// Adds an inheritance edge between two Teloid nodes.
    fn add_inheritance_edge(
        &mut self,
        parent_idx: usize,
        child_idx: usize,
    ) -> Result<(), DeonticError>;

    /// Adds a defeasance edge between two Teloid nodes.
    fn add_defeasance_edge(
        &mut self,
        defeater_idx: usize,
        defeated_idx: usize,
    ) -> Result<(), DeonticError>;
}
