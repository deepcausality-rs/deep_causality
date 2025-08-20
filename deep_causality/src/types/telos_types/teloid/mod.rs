mod getters;
mod identifiable;
mod part_eq;

use crate::{
    Context, Datable, ProposedAction, SpaceTemporal, Spatial, Symbolic, TeloidID, TeloidModal,
    TeloidTag, Temporal,
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
    activation_predicate: fn(&Context<D, S, T, ST, SYM, VS, VT>, &ProposedAction) -> bool,
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
    /// Creates a new `Teloid`, representing a complete, computable norm.
    ///
    /// This constructor initializes a Teloid with all the components required for the
    /// Defeasible Deontic Inheritance Calculus (DDIC), including the logical predicate
    /// for activation and the data needed for conflict resolution.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for this Teloid.
    /// * `action_identifier` - A string identifying the general class of action this norm governs (e.g., "vehicle.drive"). Used for efficient filtering.
    /// * `activation_predicate` - A function pointer that determines if the norm is active. It receives the current `Context` and `ProposedAction` and returns `true` if the norm's conditions are met.
    /// * `modality` - The deontic status (`Obligatory`, `Impermissible`, `Optional`) this norm assigns to the action when active.
    /// * `timestamp` - The creation time of the norm, used to resolve conflicts via the `Lex Posterior` (newer wins) heuristic.
    /// * `specificity` - A numerical value representing how specific the norm is. Used to resolve conflicts via the `Lex Specialis` (more specific wins) heuristic.
    /// * `priority` - A numerical value representing the norm's authority. Used to resolve conflicts via the `Lex Superior` (higher authority wins) heuristic.
    /// * `tags` - A vector of string tags for high-level categorization and filtering.
    /// * `metadata` - An optional `HashMap` for storing any additional, non-essential data about the norm.
    ///
    /// # Returns
    ///
    /// A new `Teloid` instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::collections::HashMap;
    /// use deep_causality::types::alias_types::alias_base::BaseContext;
    /// use deep_causality::{ProposedAction, ActionParameterValue, Teloid, TeloidModal};
    ///
    /// // 1. Define an activation predicate function matching the required signature.
    /// fn is_speeding(context: &BaseContext, proposed_action: &ProposedAction) -> bool {
    ///     // A real predicate would query the context graph's to get actual speed and the latest speed sign reading.
    ///     // For this example, we just check the action's parameters.
    ///     if let Some(ActionParameterValue::Number(speed)) = proposed_action.parameters().get("speed") {
    ///         *speed > 25.0
    ///     } else {
    ///         false
    ///     }
    /// }
    ///
    /// // 2. Create the Teloid using the constructor.
    /// let teloid = Teloid::new(
    ///     101,                                  // id
    ///     "vehicle.drive".to_string(),          // action_identifier
    ///     is_speeding,                          // activation_predicate
    ///     TeloidModal::Impermissible,           // modality
    ///     1672531200,                           // timestamp
    ///     10,                                   // specificity
    ///     5,                                    // priority
    ///     vec!["traffic_law", "speed_limit"],   // tags
    ///     None,                                 // metadata
    /// );
    ///
    /// // The Teloid is now created and represents the rule "driving faster than 25mph is impermissible".
    /// // An evaluation engine would later use its `activation_predicate` to check it.
    /// ```
    #[allow(clippy::type_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
            activation_predicate,
            modality,
            timestamp,
            specificity,
            priority,
            tags,
            metadata,
        }
    }
}
