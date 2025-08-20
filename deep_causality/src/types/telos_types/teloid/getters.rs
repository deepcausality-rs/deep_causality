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
    pub fn id(&self) -> TeloidID {
        self.id
    }

    pub fn action_identifier(&self) -> &str {
        &self.action_identifier
    }

    #[allow(clippy::type_complexity)]
    pub fn activation_predicate(
        &self,
    ) -> fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool {
        self.activation_predicate
    }

    pub fn modality(&self) -> TeloidModal {
        self.modality
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn specificity(&self) -> u32 {
        self.specificity
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn tags(&self) -> &Vec<TeloidTag> {
        &self.tags
    }

    pub fn metadata(&self) -> &Option<TeloidMetaData> {
        &self.metadata
    }
}
