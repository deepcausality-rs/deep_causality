/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    Causaloid, CausaloidId, ContextId, Contextoid, ContextoidId, Datable, Generatable,
    SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::hash::Hash;

/// Represents the possible actions or state changes that can result from a generative process.
///
/// This enum defines a set of commands that a generative system can produce. These commands
/// are intended to be interpreted by a runtime or engine to modify the state of the overall
/// causality system. This includes creating or deleting `Context`s, adding or removing
/// `Causaloid`s and `Contextoid`s, or triggering more complex, user-defined evolutionary steps.
///
/// The `GenerativeOutput` enum is designed to be extensible through the `Evolve` variant,
/// which can wrap a user-defined type `G`. This allows for domain-specific generative
/// actions beyond the core set provided here.
///
/// # Generic Parameters
///
/// * `D`: The type for data, implementing `Datable`.
/// * `S`: The type for spatial information, implementing `Spatial`.
/// * `T`: The type for temporal information, implementing `Temporal`.
/// * `ST`: The type for spatio-temporal information, implementing `SpaceTemporal`.
/// * `SYM`: The type for symbolic information, implementing `Symbolic`.
/// * `VS`: The associated type for spatial values.
/// * `VT`: The associated type for temporal values.
/// * `G`: A user-defined enum that implements `Generatable`, allowing for custom evolutionary outputs.
#[allow(clippy::type_complexity)]
#[derive(Debug, Clone, PartialEq)]
pub enum GenerativeOutput<D, S, T, ST, SYM, VS, VT, G>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G> + Sized, // G is the user's enum
{
    /// Represents no operation. This is useful for generative functions that may not
    /// always need to produce a state change.
    NoOp,

    /// Signals the creation of a new `Causaloid`.
    CreateCausaloid(
        /// The ID to assign to the new `Causaloid`.
        CausaloidId,
        /// The `Causaloid` instance to create.
        Causaloid<D, S, T, ST, SYM, VS, VT>,
    ),
    /// Signals an update to an existing `Causaloid`.
    UpdateCausaloid(
        /// The ID of the `Causaloid` to update.
        CausaloidId,
        /// The new `Causaloid` data that will replace the existing one.
        Causaloid<D, S, T, ST, SYM, VS, VT>,
    ),
    /// Signals the removal of an existing `Causaloid` by its ID.
    DeleteCausaloid(
        /// The ID of the `Causaloid` to remove.
        CausaloidId,
    ),

    /// Signals the creation of a new, empty `Context` container.
    CreateBaseContext {
        /// The unique identifier for the new context.
        id: ContextId,
        /// A human-readable name for the context.
        name: String,
        /// The initial storage capacity for the context's underlying graph.
        capacity: usize,
    },

    /// Signals the creation of a new, empty extra_context within the main Context.
    /// This corresponds to calling `extra_ctx_add_new`.
    CreateExtraContext {
        /// The unique ID to assign to this new extra context graph.
        extra_context_id: u64,
        /// The initial storage capacity for the extra context's graph.
        capacity: usize,
    },

    /// Signals an update to the metadata of an existing `Context` container.
    UpdateContext {
        /// The ID of the context to update.
        id: ContextId,
        /// An Option to provide a new name. `None` means no change.
        new_name: Option<String>,
    },
    /// Signals the deletion of an entire `Context` container, identified by its ID.
    DeleteContext {
        /// The ID of the context to delete.
        id: ContextId,
    },

    /// Signals the addition of a new `Contextoid` to a specific `Context`.
    AddContextoidToContext {
        /// The ID of the target `Context`.
        context_id: ContextId,
        /// The `Contextoid` data point to add to the context's graph.
        contextoid: Contextoid<D, S, T, ST, SYM, VS, VT>,
    },

    /// Signals an update to an existing `Contextoid` within a specific `Context`.
    UpdateContextoidInContext {
        /// The ID of the target `Context`.
        context_id: ContextId,
        /// The ID of the existing `Contextoid` to be updated.
        existing_contextoid: ContextoidId,
        /// The new `Contextoid` that will replace the existing one.
        new_contextoid: Contextoid<D, S, T, ST, SYM, VS, VT>,
    },

    /// Signals the removal of an existing `Contextoid` from a specific `Context`.
    DeleteContextoidFromContext {
        /// The ID of the target `Context`.
        context_id: ContextId,
        /// The ID of the `Contextoid` to remove from the context's graph.
        contextoid_id: ContextoidId,
    },

    /// A composite output that bundles multiple `GenerativeOutput` actions.
    /// This allows for complex, multi-step state transitions to be treated as a single unit.
    Composite(Vec<GenerativeOutput<D, S, T, ST, SYM, VS, VT, G>>),

    /// An extensible variant that wraps a user-defined generative type `G`.
    /// This allows the system to be extended with custom, domain-specific outputs
    /// that can be handled by the user's own logic.
    Evolve(G),
}
