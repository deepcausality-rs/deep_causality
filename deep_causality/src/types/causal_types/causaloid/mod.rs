// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::*;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};

mod causable;
mod display;
mod getters;
mod identifiable;
mod part_eq;

pub type CausalVec<D, S, T, ST, SYM, VS, VT> = Vec<Causaloid<D, S, T, ST, SYM, VS, VT>>;
pub type CausalGraph<D, S, TM, ST, SYM, VS, VT> =
    CausaloidGraph<Causaloid<D, S, TM, ST, SYM, VS, VT>>;

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
    active: ArcRWLock<bool>,
    causal_type: CausaloidType,
    causal_fn: Option<CausalFn>,
    context_causal_fn: Option<ContextualCausalDataFn<D, S, T, ST, SYM, VS, VT>>,
    context: Option<Arc<Context<D, S, T, ST, SYM, VS, VT>>>,
    has_context: bool,
    causal_coll: Option<Arc<CausalVec<D, S, T, ST, SYM, VS, VT>>>,
    causal_graph: Option<Arc<CausalGraph<D, S, T, ST, SYM, VS, VT>>>,
    description: String,
    ty: PhantomData<(VS, VT)>,
}

// Constructors
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
    /// Singleton constructor. Assumes causality function is valid.
    /// Only use for non-fallible construction i.e.verified a-priori knowledge about the correctness of the causal function.
    pub fn new(id: IdentificationValue, causal_fn: CausalFn, description: &str) -> Self {
        Causaloid {
            id,
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Singleton,
            causal_fn: Some(causal_fn),
            context_causal_fn: None,
            context: None,
            has_context: false,
            causal_coll: None,
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
        }
    }

    pub fn new_with_context(
        id: IdentificationValue,
        context_causal_fn: ContextualCausalDataFn<D, S, T, ST, SYM, VS, VT>,
        context: Arc<Context<D, S, T, ST, SYM, VS, VT>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Singleton,
            causal_fn: None,
            context_causal_fn: Some(context_causal_fn),
            context: Some(context),
            has_context: true,
            causal_coll: None,
            causal_graph: None,
            description: description.to_string(),
            ty: PhantomData,
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
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            description: description.to_string(),
            context: None,
            has_context: false,
            context_causal_fn: None,
            ty: PhantomData,
        }
    }

    /// Create a new causaloid from a causal collection with a context.
    /// Encapsulates a linear causal collection into one single causaloid
    /// that can be used individually, as part of another causal collection,
    /// or embedded into a causal graph.
    pub fn from_causal_collection_with_context(
        id: IdentificationValue,
        causal_coll: Arc<Vec<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        context: Arc<Context<D, S, T, ST, SYM, VS, VT>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Collection,
            causal_fn: None,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            description: description.to_string(),
            context: Some(context),
            has_context: true,
            context_causal_fn: None,
            ty: PhantomData,
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
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            context: None,
            has_context: false,
            context_causal_fn: None,
            ty: PhantomData,
        }
    }

    /// Create a new causaloid from a causal graph with a context embedded.
    /// Encapsulates a complex causal graph into one single causaloid
    /// that can be used individually, as part of causal collection,
    /// or embedded into another causal graph.
    pub fn from_causal_graph_with_context(
        id: IdentificationValue,
        causal_graph: Arc<CausaloidGraph<Causaloid<D, S, T, ST, SYM, VS, VT>>>,
        context: Arc<Context<D, S, T, ST, SYM, VS, VT>>,
        description: &str,
    ) -> Self {
        Causaloid {
            id,
            active: Arc::new(RwLock::new(false)),
            causal_type: CausaloidType::Graph,
            causal_fn: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            description: description.to_string(),
            context: Some(context),
            has_context: true,
            context_causal_fn: None,
            ty: PhantomData,
        }
    }
}
