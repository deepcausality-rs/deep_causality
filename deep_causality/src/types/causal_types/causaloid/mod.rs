/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::*;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

mod causable;
mod display;
mod getters;
mod identifiable;
mod part_eq;
mod setters;

pub type CausalVec<D, S, T, ST, SYM, VS, VT> = Vec<Causaloid<D, S, T, ST, SYM, VS, VT>>;
pub type CausalGraph<D, S, TM, ST, SYM, VS, VT> =
    CausaloidGraph<Causaloid<D, S, TM, ST, SYM, VS, VT>>;

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub struct Causaloid<D, S, T, ST, SYM, VS, VT>
where
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
    causal_fn: Option<CausalFn>,
    context_causal_fn: Option<ContextualCausalFn<D, S, T, ST, SYM, VS, VT>>,
    context: Option<Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>,
    causal_coll: Option<Arc<CausalVec<D, S, T, ST, SYM, VS, VT>>>,
    causal_graph: Option<Arc<CausalGraph<D, S, T, ST, SYM, VS, VT>>>,
    description: String,
    ty: PhantomData<(VS, VT)>,
    _phantom: PhantomData<fn() -> PropagatingEffect>,
}

// Constructors
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Causaloid<D, S, T, ST, SYM, VS, VT>
where
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
    pub fn new(id: IdentificationValue, causal_fn: CausalFn, description: &str) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Singleton,
            causal_fn: Some(causal_fn),
            context_causal_fn: None,
            context: None,
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
        context_causal_fn: ContextualCausalFn<D, S, T, ST, SYM, VS, VT>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Singleton,
            causal_fn: None,
            context_causal_fn: Some(context_causal_fn),
            context: Some(context),
            causal_coll: None,
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Create a new causaloid from a causal collection.
    /// Encapsulates a linear causal collection into one single causaloid
    /// that can be used individually, as part of another causal collection,
    /// or embedded into a causal graph.
    ///
    /// Only use for non-fallible construction i.e.verified a-priori knowledge
    /// about the correctness of the causal graph.
    pub fn from_causal_collection(
        id: IdentificationValue,
        causal_coll: Arc<Vec<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            context_causal_fn: None,
            context: None,
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
        causal_coll: Arc<Vec<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            description: description.to_string(),
            context: Some(context),
            context_causal_fn: None,
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Create a new causaloid from a causal graph.
    /// Encapsulates a complex causal graph into one single causaloid
    /// that can be used individually, as part of causal collection,
    /// or embedded into another causal graph.
    ///
    /// Only use for non-fallible construction i.e.verified a-priori knowledge
    /// about the correctness of the causal graph.
    pub fn from_causal_graph(
        id: IdentificationValue,
        causal_graph: Arc<CausaloidGraph<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            context_causal_fn: None,
            context: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }

    /// Create a new causaloid from a causal graph with a context embedded.
    /// Encapsulates a complex causal graph into one single causaloid
    /// that can be used individually, as part of causal collection,
    /// or embedded into another causal graph.
    pub fn from_causal_graph_with_context(
        id: IdentificationValue,
        causal_graph: Arc<CausaloidGraph<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        context: Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            context: Some(context),
            context_causal_fn: None,
            ty: PhantomData,
            _phantom: PhantomData,
        }
    }
}
