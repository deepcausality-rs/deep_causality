/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod display;
mod getters;
mod identifiable;
mod part_eq;

use crate::{TeloidID, TeloidModal, TeloidTag};
use deep_causality::{ProposedAction, UncertainActivationPredicate, UncertainParameter};
use deep_causality_context::Context;
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};

use std::collections::HashMap;

pub type TeloidMetaData = HashMap<String, String>;

#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
pub struct Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    id: TeloidID,
    // DDIC Norm Components
    action_identifier: String,
    // A teloid can have either a deterministic or an uncertain predicate.
    activation_predicate: Option<fn(&Context<D, S, T, ST>, &ProposedAction) -> bool>,
    uncertain_activation_predicate: Option<UncertainActivationPredicate<D, S, T, ST>>,
    uncertain_parameter: Option<UncertainParameter>,
    modality: TeloidModal,

    // Conflict Resolution Data (Heuristics)
    timestamp: u64,
    specificity: u32,
    priority: u32,

    // Helper Fields
    tags: Vec<TeloidTag>,
    metadata: Option<TeloidMetaData>,
}

impl<D, S, T, ST> Teloid<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Creates a new `Teloid` with a deterministic predicate.
    /// This represents a complete, computable norm with a hard, boolean activation condition.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_deterministic(
        id: TeloidID,
        action_identifier: String,
        activation_predicate: fn(&Context<D, S, T, ST>, &ProposedAction) -> bool,
        modality: TeloidModal,
        timestamp: u64,
        specificity: u32,
        priority: u32,
        tags: Vec<TeloidTag>,
        metadata: Option<TeloidMetaData>,
    ) -> Self {
        Self {
            id,
            action_identifier,
            activation_predicate: Some(activation_predicate),
            uncertain_activation_predicate: None,
            uncertain_parameter: None,
            modality,
            timestamp,
            specificity,
            priority,
            tags,
            metadata,
        }
    }

    /// Creates a new `Teloid` with an uncertain predicate.
    /// This represents a complete, computable norm with a soft, probabilistic activation condition.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_uncertain(
        id: TeloidID,
        action_identifier: String,
        uncertain_activation_predicate: UncertainActivationPredicate<D, S, T, ST>,
        predicate_parameter: UncertainParameter,
        modality: TeloidModal,
        timestamp: u64,
        specificity: u32,
        priority: u32,
        tags: Vec<TeloidTag>,
        metadata: Option<TeloidMetaData>,
    ) -> Self {
        Self {
            id,
            action_identifier,
            activation_predicate: None,
            uncertain_activation_predicate: Some(uncertain_activation_predicate),
            uncertain_parameter: Some(predicate_parameter),
            modality,
            timestamp,
            specificity,
            priority,
            tags,
            metadata,
        }
    }
}
