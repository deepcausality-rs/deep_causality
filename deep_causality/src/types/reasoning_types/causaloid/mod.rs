// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::{Causable, CausalFn, CausaloidGraph, Context, ContextualCausalFn, Datable, Identifiable, IdentificationValue, NumericalValue, SpaceTemporal, Spatial, Temporal};
use crate::types::reasoning_types::causaloid::causal_type::CausalType;

mod causal_type;
mod part_eq;
mod getters;
mod identifiable;
mod display;
mod causable;

#[derive(Clone)]
pub struct Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    id: IdentificationValue,
    active: Cell<bool>,
    causal_type: CausalType,
    causal_fn: Option<CausalFn>,
    context_causal_fn: Option<ContextualCausalFn<'l, D, S, T, ST>>,
    context: Option<&'l Context<'l, D, S, T, ST>, >,
    has_context: bool,
    causal_coll: Option<Vec<Causaloid<'l, D, S, T, ST>>>,
    causal_graph: Option<CausaloidGraph<Causaloid<'l, D, S, T, ST>>>,
    last_obs: Cell<NumericalValue>,
    description: &'l str,
}


// Constructors
impl<'l, D, S, T, ST> Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
{
    /// Singleton constructor. Assumes causality function is valid.
    /// Only use for non-fallible construction i.e.verified a-priori knowledge about the correctness of the causal function.
    pub fn new(
        id: IdentificationValue,
        causal_fn: CausalFn,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Singleton,
            causal_fn: Some(causal_fn),
            context_causal_fn: None,
            context: None,
            has_context: false,
            causal_coll: None,
            causal_graph: None,
            last_obs: Cell::new(0.0),
            description,
        }
    }

    pub fn new_with_context(
        id: IdentificationValue,
        context_causal_fn: ContextualCausalFn<'l, D, S, T, ST>,
        context: Option<&'l Context<'l, D, S, T, ST>, >,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Singleton,
            causal_fn: None,
            context_causal_fn: Some(context_causal_fn),
            context,
            has_context: true,
            causal_coll: None,
            causal_graph: None,
            last_obs: Cell::new(0.0),
            description,
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
        causal_coll: Vec<Causaloid<'l, D, S, T, ST>>,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Collection,
            causal_fn: None,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            last_obs: Cell::new(0.0),
            description,
            context: None,
            has_context: false,
            context_causal_fn: None,
        }
    }

    /// Create a new causaloid from a causal collection with a context.
    /// Encapsulates a linear causal collection into one single causaloid
    /// that can be used individually, as part of another causal collection,
    /// or embedded into a causal graph.
    pub fn from_causal_collection_with_context(
        id: IdentificationValue,
        causal_coll: Vec<Causaloid<'l, D, S, T, ST>>,
        context: Option<&'l Context<'l, D, S, T, ST>, >,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Collection,
            causal_fn: None,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            last_obs: Cell::new(0.0),
            description,
            context,
            has_context: true,
            context_causal_fn: None,
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
        causal_graph: CausaloidGraph<Causaloid<'l, D, S, T, ST>>,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Graph,
            causal_fn: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            last_obs: Cell::new(0.0),
            description,
            context: None,
            has_context: false,
            context_causal_fn: None,
        }
    }

    /// Create a new causaloid from a causal graph with a context embedded.
    /// Encapsulates a complex causal graph into one single causaloid
    /// that can be used individually, as part of causal collection,
    /// or embedded into another causal graph.
    pub fn from_causal_graph_with_context(
        id: IdentificationValue,
        causal_graph: CausaloidGraph<Causaloid<'l, D, S, T, ST>>,
        context: Option<&'l Context<'l, D, S, T, ST>, >,
        description: &'l str,
    )
        -> Self
    {
        Causaloid {
            id,
            active: Cell::new(false),
            causal_type: CausalType::Graph,
            causal_fn: None,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            last_obs: Cell::new(0.0),
            description,
            context,
            has_context: true,
            context_causal_fn: None,
        }
    }
}





