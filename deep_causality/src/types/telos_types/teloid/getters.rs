/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TeloidMetaData;
use crate::{
    Context, Datable, ProposedAction, SpaceTemporal, Spatial, Symbolic, Teloid, TeloidID,
    TeloidModal, TeloidTag, Temporal,
};

impl<D, S, T, ST, SYM, VS, VT> Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Returns the unique identifier of the teloid.
    ///
    /// # Returns
    ///
    /// A `TeloidID` representing the teloid's unique identifier.
    pub fn id(&self) -> TeloidID {
        self.id
    }

    /// Returns a reference to the action identifier string of the teloid.
    ///
    /// # Returns
    ///
    /// A string slice (`&str`) representing the action identifier.
    pub fn action_identifier(&self) -> &str {
        &self.action_identifier
    }

    /// Returns the activation predicate function of the teloid.
    ///
    /// The activation predicate is a function that determines whether a teloid should be activated
    /// based on the current context and a proposed action.
    ///
    /// # Returns
    ///
    /// A function pointer `fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool`
    /// that represents the activation predicate.
    #[allow(clippy::type_complexity)]
    pub fn activation_predicate(
        &self,
    ) -> fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool {
        self.activation_predicate
    }

    /// Returns the modality of the teloid.
    ///
    /// # Returns
    ///
    /// A `TeloidModal` enum variant representing the teloid's modality.
    pub fn modality(&self) -> TeloidModal {
        self.modality
    }

    /// Returns the timestamp of the teloid.
    ///
    /// # Returns
    ///
    /// A `u64` representing the teloid's timestamp.
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn specificity(&self) -> u32 {
        self.specificity
    }

    /// Returns the priority of the teloid.
    ///
    /// # Returns
    ///
    /// A `u32` representing the teloid's priority.
    pub fn priority(&self) -> u32 {
        self.priority
    }

    /// Returns a reference to the vector of tags associated with the teloid.
    ///
    /// # Returns
    ///
    /// A reference to a `Vec<TeloidTag>` representing the teloid's tags.
    pub fn tags(&self) -> &Vec<TeloidTag> {
        &self.tags
    }

    /// Returns a reference to the optional metadata associated with the teloid.
    ///
    /// # Returns
    ///
    /// A reference to an `Option<TeloidMetaData>` representing the teloid's metadata.
    /// Returns `None` if no metadata is present.
    pub fn metadata(&self) -> &Option<TeloidMetaData> {
        &self.metadata
    }
}
