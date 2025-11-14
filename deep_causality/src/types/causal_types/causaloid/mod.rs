/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::*;
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

pub type CausalVec<I, O, D, S, T, ST, SYM, VS, VT> = Vec<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>;
pub type CausalGraph<I, O, D, S, T, ST, SYM, VS, VT> =
    CausaloidGraph<Causaloid<I, O, D, S, T, ST, SYM, VS, VT>>;

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
    id: IdentificationValue,
    causal_type: CausaloidType,
    causal_fn: Option<CausalFn<I, O>>,
    coll_aggregate_logic: Option<AggregateLogic>,
    coll_threshold_value: Option<NumericalValue>,
    context_causal_fn: Option<ContextualCausalFn<I, O, D, S, T, ST, SYM, VS, VT>>,
    context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>,
    causal_coll: Option<Arc<Vec<Self>>>,
    causal_graph: Option<Arc<CausaloidGraph<Self>>>,
    description: String,
    ty: PhantomData<(VS, VT)>,
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
    /// Creates a new singleton `Causaloid`.
    ///
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `causal_fn` - The stateless function that defines the causaloid's reasoning logic.
    /// * `description` - A human-readable description of the causaloid.
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
    /// # Arguments
    ///
    /// * `id` - A unique identifier for the causaloid.
    /// * `context_causal_fn` - The context-aware stateless function for reasoning.
    /// * `context` - A shared `Context` object accessible by the function.
    /// * `description` - A human-readable description of the causaloid.
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
    /// Create a new causaloid from a causal collection.
    /// Encapsulates a linear causal collection into one single causaloid
    /// that can be used individually, as part of another causal collection,
    /// or embedded into a causal graph.
    ///
    /// Only use for non-fallible construction i.e.verified a-priori knowledge
    /// about the correctness of the causal graph.
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

    /// Create a new causaloid from a causal collection with a context.
    /// Encapsulates a linear causal collection into one single causaloid
    /// that can be used individually, as part of another causal collection,
    /// or embedded into a causal graph.
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
