/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Context, Datable, DeonticError, EffectEthos, ProposedAction, SpaceTemporal, Spatial, Symbolic,
    Teloid, TeloidID, TeloidModal, TeloidStorable, TeloidTag, Teloidable, Temporal,
    UncertainActivationPredicate, UncertainParameter,
};
use ultragraph::GraphMut;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Adds a new deterministic norm to the ethos using a builder-style pattern.
    ///
    /// This high-level method simplifies the process of adding norms by accepting
    /// all necessary components as arguments, constructing the `Teloid` internally,
    /// and handling all necessary updates to the store, graph, and indices.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for the new norm.
    /// * `action_identifier` - A string slice identifying the action this norm governs.
    /// * `tags` - A slice of string tags for categorization.
    /// * `predicate` - The function pointer that determines if the norm is active.
    /// * `modality` - The deontic status (`Obligatory`, `Impermissible`, `Optional`).
    /// * `timestamp` - The creation time of the norm for `Lex Posterior`.
    /// * `specificity` - The specificity level for `Lex Specialis`.
    /// * `priority` - The priority level for `Lex Superior`.
    ///
    /// # Returns
    /// A `Result` containing `self` for method chaining, or a `DeonticError` if the
    /// `id` already exists.
    #[allow(clippy::too_many_arguments)]
    pub fn add_deterministic_norm(
        mut self,
        id: TeloidID,
        action_identifier: &str,
        tags: &[TeloidTag],
        predicate: fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool,
        modality: TeloidModal,
        timestamp: u64,
        specificity: u32,
        priority: u32,
    ) -> Result<Self, DeonticError> {
        if self.teloid_store.contains_key(&id) {
            return Err(DeonticError::FailedToAddTeloid);
        }

        let teloid = Teloid::new_deterministic(
            id,
            action_identifier.to_string(),
            predicate,
            modality,
            timestamp,
            specificity,
            priority,
            tags.to_vec(),
            None, // Metadata can be added later if needed
        );

        let id = teloid.id();
        let tags = teloid.tags().clone();

        self.teloid_store.insert(teloid);
        for tag in tags {
            self.tag_index.add(tag, id);
        }

        let node_index = self
            .teloid_graph
            .graph
            .add_node(id)
            .expect("Failed to add node");

        self.id_to_index_map.insert(id, node_index);
        self.is_verified = false; // A modification invalidates prior verification.

        Ok(self)
    }

    /// Adds a new uncertain norm to the ethos using a builder-style pattern.
    #[allow(clippy::too_many_arguments)]
    pub fn add_uncertain_norm(
        mut self,
        id: TeloidID,
        action_identifier: &str,
        tags: &[TeloidTag],
        predicate: UncertainActivationPredicate<D, S, T, ST, SYM, VS, VT>,
        predicate_parameter: UncertainParameter,
        modality: TeloidModal,
        timestamp: u64,
        specificity: u32,
        priority: u32,
    ) -> Result<Self, DeonticError> {
        if self.teloid_store.contains_key(&id) {
            return Err(DeonticError::FailedToAddTeloid);
        }

        let teloid = Teloid::new_uncertain(
            id,
            action_identifier.to_string(),
            predicate,
            predicate_parameter,
            modality,
            timestamp,
            specificity,
            priority,
            tags.to_vec(),
            None, // Metadata can be added later if needed
        );

        let id = teloid.id();
        let tags = teloid.tags().clone();

        self.teloid_store.insert(teloid);
        for tag in tags {
            self.tag_index.add(tag, id);
        }

        let node_index = self
            .teloid_graph
            .graph
            .add_node(id)
            .expect("Failed to add node");

        self.id_to_index_map.insert(id, node_index);
        self.is_verified = false; // A modification invalidates prior verification.

        Ok(self)
    }

    /// Retrieves a reference to a norm by its unique identifier.
    ///
    /// This method provides read-only access to a `Teloid` stored within the `EffectEthos`
    /// based on its `TeloidID`.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the norm to retrieve.
    ///
    /// # Returns
    /// An `Option` containing a reference to the `Teloid` if found, or `None` otherwise.
    pub fn get_norm(&self, id: TeloidID) -> Option<&Teloid<D, S, T, ST, SYM, VS, VT>> {
        self.teloid_store.get(&id)
    }

    /// Defines an inheritance relationship between two norms by their IDs.
    ///
    /// # Arguments
    /// * `parent_id` - The ID of the more general norm.
    /// * `child_id` - The ID of the more specific norm that inherits from the parent.
    ///
    /// # Returns
    /// A `Result` containing `self` for method chaining, or a `DeonticError` if either ID
    /// is not found or the graph is frozen.
    pub fn link_inheritance(
        mut self,
        parent_id: TeloidID,
        child_id: TeloidID,
    ) -> Result<Self, DeonticError> {
        if self.is_frozen() {
            return Err(DeonticError::GraphIsFrozen);
        }

        let parent_idx = *self
            .id_to_index_map
            .get(&parent_id)
            .ok_or(DeonticError::TeloidNotFound { id: parent_id })?;

        let child_idx = *self
            .id_to_index_map
            .get(&child_id)
            .ok_or(DeonticError::TeloidNotFound { id: child_id })?;

        self.teloid_graph
            .add_inheritance_edge(parent_idx, child_idx)?;
        self.is_verified = false;

        Ok(self)
    }

    /// Defines a defeasance relationship between two norms by their IDs.
    ///
    /// # Arguments
    /// * `defeater_id` - The ID of the norm that can defeat the other.
    /// * `defeated_id` - The ID of the norm that can be defeated.
    ///
    /// # Returns
    /// A `Result` containing `self` for method chaining, or a `DeonticError` if either ID
    /// is not found or the graph is frozen.
    pub fn link_defeasance(
        mut self,
        defeater_id: TeloidID,
        defeated_id: TeloidID,
    ) -> Result<Self, DeonticError> {
        if self.is_frozen() {
            return Err(DeonticError::GraphIsFrozen);
        }

        let defeater_idx = *self
            .id_to_index_map
            .get(&defeater_id)
            .ok_or(DeonticError::TeloidNotFound { id: defeater_id })?;

        let defeated_idx = *self
            .id_to_index_map
            .get(&defeated_id)
            .ok_or(DeonticError::TeloidNotFound { id: defeated_id })?;

        self.teloid_graph
            .add_defeasance_edge(defeater_idx, defeated_idx)?;
        self.is_verified = false;

        Ok(self)
    }
}
