/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the `Causaloid` struct, the fundamental building block for representing
//! causal units within the DeepCausality framework.
//!
//! A `Causaloid` encapsulates a piece of causal logic, which can range from a simple
//! stateless function to a complex causal graph or a collection of other causaloids.
//! It supports both stateless and context-aware operations, allowing for flexible
//! and dynamic causal modeling.
//!
//! The module also provides various constructors to facilitate the creation of
//! different types of `Causaloid` instances, tailored to specific causal modeling needs.
use crate::{
    AggregateLogic, CausalFn, CausalMonad, CausaloidGraph, CausaloidType, Context,
    ContextualCausalFn, IdentificationValue, NumericalValue,
};
use crate::{
    Datable, IntoEffectValue, MonadicCausable, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

mod causable;
mod causable_utils;
mod display;
mod getters;
mod identifiable;
mod part_eq;
mod setters;

/// Type alias for a vector of `Causaloid`s.
///
/// This is typically used to represent a linear collection of causaloids,
/// which can then be encapsulated within a parent `Causaloid` of type `Collection`.
pub type CausalVec<I, O, D, S, T, ST, SYM, VS, VT> = Vec<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>;

/// Type alias for a graph of `Causaloid`s.
///
/// This represents a more complex, interconnected structure of causaloids,
/// which can be encapsulated within a parent `Causaloid` of type `Graph`.
pub type CausalGraph<I, O, D, S, T, ST, SYM, VS, VT> =
    CausaloidGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>;

/// `Causaloid` is the fundamental building block for causal models in DeepCausality.
///
/// It represents a self-contained unit of causality, capable of encapsulating
/// various forms of causal logic:
/// - A single, stateless causal function (`CausaloidType::Singleton`).
/// - A single, context-aware causal function (`CausaloidType::Singleton` with `context`).
/// - A collection of other `Causaloid`s (`CausaloidType::Collection`).
/// - A directed acyclic graph (DAG) of other `Causaloid`s (`CausaloidType::Graph`).
///
/// `Causaloid`s are generic over their input (`I`) and output (`O`) effect values,
/// as well as various context-related types (`D`, `S`, `T`, `ST`, `SYM`, `VS`, `VT`).
///
/// # Type Parameters
/// - `I`: The type of the input effect value, must implement `IntoEffectValue`.
/// - `O`: The type of the output effect value, must implement `IntoEffectValue`.
/// - `D`: The type for data context, must implement `Datable` and `Clone`.
/// - `S`: The type for spatial context, must implement `Spatial<VS>` and `Clone`.
/// - `T`: The type for temporal context, must implement `Temporal<VT>` and `Clone`.
/// - `ST`: The type for spatiotemporal context, must implement `SpaceTemporal<VS, VT>` and `Clone`.
/// - `SYM`: The type for symbolic context, must implement `Symbolic` and `Clone`.
/// - `VS`: The value type for spatial data, must implement `Clone`.
/// - `VT`: The value type for temporal data, must implement `Clone`.
#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// A unique identifier for this `Causaloid`.
    id: IdentificationValue,
    /// The type of causal logic encapsulated by this `Causaloid`.
    causal_type: CausaloidType,
    /// An optional stateless causal function, used when `causal_type` is `Singleton`.
    causal_fn: Option<CausalFn<I, O>>,
    /// An optional aggregation logic for `Collection` type causaloids.
    coll_aggregate_logic: Option<AggregateLogic>,
    /// An optional threshold value for `Collection` type causaloids.
    coll_threshold_value: Option<NumericalValue>,
    /// An optional context-aware causal function, used when `causal_type` is `Singleton`.
    context_causal_fn: Option<ContextualCausalFn<I, O, D, S, T, ST, SYM, VS, VT>>,
    /// An optional shared context object, used by context-aware causaloids.
    context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>,
    /// An optional collection of child `Causaloid`s, used when `causal_type` is `Collection`.
    causal_coll: Option<Arc<Vec<Self>>>,
    /// An optional causal graph of child `Causaloid`s, used when `causal_type` is `Graph`.
    causal_graph: Option<Arc<CausaloidGraph<Self>>>,
    /// A human-readable description of the `Causaloid`.
    description: String,
    /// PhantomData to mark the usage of `VS` and `VT` type parameters.
    ty: PhantomData<(VS, VT)>,
    /// PhantomData to mark the usage of `I` and `O` type parameters.
    _phantom: PhantomData<(I, O)>,
}

// Constructors
#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Creates a new singleton `Causaloid` with a stateless causal function.
    ///
    /// This constructor is used for `Causaloid`s that represent a single, atomic
    /// causal relationship defined by a pure function, without requiring any external context.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `causal_fn` - The stateless function that defines the causaloid's reasoning logic.
    /// * `description` - A human-readable description of the causaloid.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Singleton`.
    pub fn new(id: IdentificationValue, causal_fn: CausalFn<I, O>, description: &str) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Singleton,
            causal_fn: Some(causal_fn),
            context_causal_fn: None,
            context: None,
            coll_aggregate_logic: None,
            coll_threshold_value: None,
            causal_coll: None,
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Creates a new singleton `Causaloid` with a context-aware causal function.
    ///
    /// This constructor is used for `Causaloid`s that represent a single, atomic
    /// causal relationship whose logic depends on an external, shared `Context`.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `context_causal_fn` - The context-aware stateless function for reasoning.
    /// * `context` - A shared `Arc<RwLock<Context>>` object accessible by the function.
    /// * `description` - A human-readable description of the causaloid.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Singleton` with an associated context.
    pub fn new_with_context(
        id: IdentificationValue,
        context_causal_fn: ContextualCausalFn<I, O, D, S, T, ST, SYM, VS, VT>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Singleton,
            causal_fn: None,
            context_causal_fn: Some(context_causal_fn),
            context: Some(context),
            coll_aggregate_logic: None,
            coll_threshold_value: None,
            causal_coll: None,
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }
}

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Creates a new `Causaloid` that encapsulates a linear collection of other `Causaloid`s.
    ///
    /// This allows treating a sequence of causal relationships as a single, composite causal unit.
    /// The collection can be evaluated individually, as part of another causal collection,
    /// or embedded into a causal graph.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `causal_coll` - An `Arc` to a vector of child `Causaloid`s forming the collection.
    /// * `description` - A human-readable description of the causaloid.
    /// * `aggregate_logic` - The logic used to aggregate the results of the child causaloids.
    /// * `threshold_value` - A numerical threshold relevant to the aggregation logic.
    ///
    /// # Type Bounds
    /// The child `Causaloid`s within the collection must implement `Send`, `Sync`, and `'static`
    /// to ensure thread safety and static lifetime requirements for shared ownership.
    /// They must also implement `MonadicCausable<CausalMonad>` for evaluation.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Collection`.
    pub fn from_causal_collection(
        id: IdentificationValue,
        causal_coll: Arc<Vec<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
        aggregate_logic: AggregateLogic,
        threshold_value: NumericalValue,
    ) -> Self
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        D: Send + Sync + 'static,
        S: Spatial<VS> + Clone + Send + Sync + 'static,
        T: Temporal<VT> + Clone + Send + Sync + 'static,
        ST: SpaceTemporal<VS, VT> + Clone + Send + Sync + 'static,
        SYM: Symbolic + Clone + Send + Sync + 'static,
        VS: Send + Sync + 'static,
        VT: Send + Sync + 'static,
        Causaloid<I, O, D, S, T, ST, SYM, VS, VT>: MonadicCausable<CausalMonad>,
    {
        Causaloid {
            id,
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            context_causal_fn: None,
            context: None,
            coll_aggregate_logic: Some(aggregate_logic),
            coll_threshold_value: Some(threshold_value),
            causal_coll: Some(causal_coll),
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `Causaloid` that encapsulates a linear collection of other `Causaloid`s
    /// with an associated shared context.
    ///
    /// This allows treating a sequence of causal relationships as a single, composite causal unit,
    /// where the evaluation of the collection can depend on an external, shared `Context`.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `causal_coll` - An `Arc` to a vector of child `Causaloid`s forming the collection.
    /// * `context` - A shared `Arc<RwLock<Context>>` object accessible by the collection's logic.
    /// * `description` - A human-readable description of the causaloid.
    /// * `aggregate_logic` - The logic used to aggregate the results of the child causaloids.
    /// * `threshold_value` - A numerical threshold relevant to the aggregation logic.
    ///
    /// # Type Bounds
    /// The child `Causaloid`s within the collection must implement `Send`, `Sync`, and `'static`
    /// to ensure thread safety and static lifetime requirements for shared ownership.
    /// They must also implement `MonadicCausable<CausalMonad>` for evaluation.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Collection` with an associated context.
    pub fn from_causal_collection_with_context(
        id: IdentificationValue,
        causal_coll: Arc<Vec<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
        aggregate_logic: AggregateLogic,
        threshold_value: NumericalValue,
    ) -> Self
    where
        I: Send + Sync + 'static,
        O: Send + Sync + 'static,
        D: Send + Sync + 'static,
        S: Spatial<VS> + Clone + Send + Sync + 'static,
        T: Temporal<VT> + Clone + Send + Sync + 'static,
        ST: SpaceTemporal<VS, VT> + Clone + Send + Sync + 'static,
        SYM: Symbolic + Clone + Send + Sync + 'static,
        VS: Send + Sync + 'static,
        VT: Send + Sync + 'static,
        Causaloid<I, O, D, S, T, ST, SYM, VS, VT>: MonadicCausable<CausalMonad>,
    {
        Causaloid {
            id,
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            coll_aggregate_logic: Some(aggregate_logic),
            coll_threshold_value: Some(threshold_value),
            causal_coll: Some(causal_coll),
            causal_graph: None,
            description: description.to_string(),
            context: Some(context),
            context_causal_fn: None,
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `Causaloid` that encapsulates a causal graph of other `Causaloid`s.
    ///
    /// This allows treating a complex, interconnected network of causal relationships
    /// as a single, composite causal unit. The graph can be evaluated independently,
    /// or embedded into a larger causal structure.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `description` - A human-readable description of the causaloid.
    /// * `causal_graph` - An `Arc` to a `CausaloidGraph` containing the interconnected child `Causaloid`s.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Graph`.
    pub fn from_causal_graph(
        id: IdentificationValue,
        description: &str,
        causal_graph: Arc<CausaloidGraph<Self>>,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            causal_coll: None,
            coll_aggregate_logic: None,
            coll_threshold_value: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            context: None,
            context_causal_fn: None,
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Creates a new `Causaloid` that encapsulates a causal graph of other `Causaloid`s
    /// with an associated shared context.
    ///
    /// This allows treating a complex, interconnected network of causal relationships
    /// as a single, composite causal unit, where the evaluation of the graph can depend
    /// on an external, shared `Context`.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `description` - A human-readable description of the causaloid.
    /// * `causal_graph` - An `Arc` to a `CausaloidGraph` containing the interconnected child `Causaloid`s.
    /// * `context` - A shared `Arc<RwLock<Context>>` object accessible by the graph's logic.
    ///
    /// # Returns
    /// A new `Causaloid` instance of `CausaloidType::Graph` with an associated context.
    pub fn from_causal_graph_with_context(
        id: IdentificationValue,
        description: &str,
        causal_graph: Arc<CausaloidGraph<Self>>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            causal_coll: None,
            coll_aggregate_logic: None,
            coll_threshold_value: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            context: Some(context),
            context_causal_fn: None,
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }
}
