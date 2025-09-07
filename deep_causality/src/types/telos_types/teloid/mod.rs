mod display;
mod getters;
mod identifiable;
mod part_eq;

use crate::{
    Context, Datable, ProposedAction, SpaceTemporal, Spatial, Symbolic, TeloidID, TeloidModal,
    TeloidTag, Temporal, UncertainActivationPredicate, UncertainParameter,
};
use std::collections::HashMap;

pub type TeloidMetaData = HashMap<String, String>;

#[derive(Debug, Clone)]
#[allow(clippy::type_complexity)]
pub struct Teloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    id: TeloidID,
    // DDIC Norm Components
    action_identifier: String,
    // A teloid can have either a deterministic or an uncertain predicate.
    activation_predicate: Option<fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool>,
    uncertain_activation_predicate: Option<UncertainActivationPredicate<D, S, T, ST, SYM, VS, VT>>,
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
    /// Creates a new `Teloid` with a deterministic predicate.
    /// This represents a complete, computable norm with a hard, boolean activation condition.
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_deterministic(
        id: TeloidID,
        action_identifier: String,
        activation_predicate: fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool,
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
        uncertain_activation_predicate: UncertainActivationPredicate<D, S, T, ST, SYM, VS, VT>,
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
