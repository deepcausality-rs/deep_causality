/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
pub mod generative_processor;

use crate::errors::ModelGenerativeError;
use crate::prelude::{
    Context, Datable, GenerativeOutput, GenerativeTrigger, SpaceTemporal, Spatial, Symbolic,
    Temporal,
};
use std::hash::Hash;

/// Defines the core interface for a generative model capable of producing
/// commands to modify a causal structure.
///
/// This trait is central to creating dynamic and adaptive causal models. An
/// implementor of `Generatable` acts as the "brain" of the system, deciding
/// what actions to take in response to incoming data (`GenerativeTrigger`)
/// within a given `Context`.
///
/// # Key Concepts
///
/// ## 1. Generative Logic
/// The primary method, `generate`, encapsulates the model's decision-making
/// logic. It takes a trigger and the current context and produces a
/// `GenerativeOutput`, which is a command or a set of commands (e.g., create a
/// causaloid, update a context, add a node).
///
/// ## 2. Model Evolution with Recursive Generics (`G`)
/// The trait is generic over itself (`G: Generatable<...>`). This powerful
/// pattern enables a model to evolve. The `generate` method can return a
/// `GenerativeOutput::Evolve(new_model)`, where `new_model` is another,
/// potentially different, instance that also implements `Generatable`. This allows
/// the system's behavior to change over time, for example, by switching from a
/// simple rule-based model to a more complex one after certain conditions are met.
///
/// ## 3. Decoupling Logic from State
/// `Generatable` only *decides* what should happen. The actual execution of the
/// generated commands is handled by a separate entity, typically a
/// `GenerativeProcessor`. This separation of concerns makes the system more
/// modular and testable.
///
/// # Type Parameters
///
/// *   `D`: The core data type used in causaloids, which must be `Datable`, `Copy`, `Clone`, etc.
/// *   `S`: The type representing spatial properties, implementing `Spatial`.
/// *   `T`: The type representing temporal properties, implementing `Temporal`.
/// *   `ST`: The type representing combined spacetime properties, implementing `SpaceTemporal`.
/// *   `SYM`: The type for symbolic data or representations, implementing `Symbolic`.
/// *   `VS`: The underlying value type for spatial dimensions (e.g., `f64`).
/// *   `VT`: The underlying value type for the temporal dimension (e.g., `u64`).
/// *   `G`: A recursive type parameter representing the `Generatable` implementor itself.
///      This is crucial for the `GenerativeOutput::Evolve(G)` variant.
///
#[allow(clippy::type_complexity)]
pub trait Generatable<D, S, T, ST, SYM, VS, VT, G>
where
    D: Default + Datable + Copy + Clone + Hash + Eq + PartialEq,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
    G: Generatable<D, S, T, ST, SYM, VS, VT, G> + Sized,
{
    /// Generates a `GenerativeOutput` command based on a trigger and the current context.
    ///
    /// This method is the core of the generative logic. It allows the model to react
    /// to new information (`trigger`) by inspecting the state of the world (`context`)
    /// and deciding on a course of action. The implementor can mutate its own internal
    /// state during this process.
    ///
    /// # Arguments
    ///
    /// * `self`: A mutable reference to the model, allowing for internal state changes.
    /// * `trigger`: A reference to the `GenerativeTrigger` that initiated this action.
    ///   This typically contains the new data point or event that the model needs to process.
    /// * `context`: A reference to the full `Context`, providing a read-only view of the
    ///   current causal graph and its associated contextoids. This allows for informed,
    ///   context-aware decisions.
    ///
    /// # Returns
    ///
    /// * `Ok(GenerativeOutput)`: On success, returns a command to be executed by a
    ///   `GenerativeProcessor`. This can be a single action (`CreateCausaloid`), a
    ///   no-op (`NoOp`), a collection of actions (`Composite`), or a command to
    ///   evolve the model itself (`Evolve`).
    /// * `Err(ModelGenerativeError)`: If an error occurs during the generation process,
    ///   preventing a valid output from being created.
    fn generate(
        &mut self,
        trigger: &GenerativeTrigger<D>,
        context: &Context<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<GenerativeOutput<D, S, T, ST, SYM, VS, VT, G>, ModelGenerativeError>;
}
