// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::*;
use crate::protocols::causable::Causable;

// Internal enum to represent the type of causaloid, which
// is required to dispatch verify and explain method calls to
// either a singleton, a causal collection, or causal graph.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum CausalType {
    Singleton,
    Collection,
    Graph,
}

impl Display for CausalType { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } }


#[derive(Clone)]
pub struct Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
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
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
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

// Getters
impl<'l, D, S, T, ST> Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    pub fn active(&self) -> bool {
        self.active.get()
    }
    pub fn causal_collection(&self) -> Option<Vec<Causaloid<'l, D, S, T, ST>>> {
        self.causal_coll.clone()
    }
    pub fn causal_graph(&self) -> Option<CausaloidGraph<Causaloid<'l, D, S, T, ST>>> {
        self.causal_graph.clone()
    }
    pub fn last_obs(&self) -> NumericalValue {
        self.last_obs.get()
    }
    pub fn description(&self) -> &'l str {
        self.description
    }
    pub fn context(&self) -> Option<&'l Context<'l, D, S, T, ST>> {
        self.context
    }
}

impl<'l, D, S, T, ST> PartialEq for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'l, D, S, T, ST> Identifiable for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl<'l, D, S, T, ST> Causable for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn explain(&self)
               -> Result<String, CausalityError>
    {
        return if self.active.get()
        {
            match self.causal_type
            {
                CausalType::Singleton =>
                    {
                        let reason = format!("Causaloid: {} {} on last data {} evaluated to {}",
                                             self.id, self.description, self.last_obs.get(), self.is_active());
                        Ok(reason)
                    }

                CausalType::Collection =>
                    {
                        Ok(self.causal_coll.as_ref().unwrap().explain())
                    }

                CausalType::Graph =>
                    {
                        match self.causal_graph.as_ref().unwrap().explain_all_causes()
                        {
                            Ok(str) => Ok(str),
                            Err(e) => Err(CausalityError(e.to_string())),
                        }
                    }
            }
        } else {
            // Return an error message that the causaloid is not active
            let reason = format!("Causaloid: {} has not been evaluated. Call verify() to activate it", self.id);

            Err(CausalityError(reason))
        };
    }

    fn is_active(&self) -> bool {
        self.active.get()
    }

    fn is_singleton(&self) -> bool {
        match self.causal_type {
            CausalType::Singleton => true,
            CausalType::Collection => false,
            CausalType::Graph => false,
        }
    }

    fn verify_single_cause(
        &self,
        obs: &NumericalValue,
    )
        -> Result<bool, CausalityError>
    {
        if self.has_context
        {
            let contextual_causal_fn = self.context_causal_fn.expect("Causaloid::verify_single_cause: context_causal_fn is None");
            let context = self.context.expect("Causaloid::verify_single_cause: context is None");

            let res = match (contextual_causal_fn)(obs.to_owned(), context)
            {
                Ok(res) => {
                    // store the applied data to provide details in explain()
                    self.last_obs.set(obs.to_owned());
                    res
                }
                Err(e) => return Err(e),
            };

            Ok(self.check_active(res))
        } else {
            let causal_fn = self.causal_fn.expect("Causaloid::verify_single_cause: causal_fn is None");
            let res = match (causal_fn)(obs.to_owned())
            {
                Ok(res) => {
                    // store the applied data to provide details in explain()
                    self.last_obs.set(obs.to_owned());
                    res
                }
                Err(e) => return Err(e),
            };

            Ok(self.check_active(res))
        }
    }

    fn verify_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityError>
    {
        match self.causal_type
        {
            CausalType::Singleton => Err(CausalityError("Causaloid is singleton. Call verify_singleton instead.".into())),

            CausalType::Collection =>
                {
                    match &self.causal_coll
                    {
                        None => Err(CausalityError("Causaloid::verify_all_causes: causal collection is None".into())),
                        Some(coll) =>
                            {
                                let res = match coll.reason_all_causes(data)
                                {
                                    Ok(res) => res,
                                    Err(e) => return Err(e),
                                };

                                Ok(self.check_active(res))
                            }
                    }
                }

            CausalType::Graph =>
                {
                    match &self.causal_graph
                    {
                        None => Err(CausalityError("Causaloid::verify_all_causes: Causal graph is None".into())),
                        Some(graph) =>
                            {
                                let res = match graph.reason_all_causes(data, data_index)
                                {
                                    Ok(res) => res,
                                    Err(e) => return Err(CausalityError(e.to_string())),
                                };

                                Ok(self.check_active(res))
                            }
                    }
                }
        }
    }
}


impl<'l, D, S, T, ST> Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    #[inline(always)]
    fn check_active(
        &self,
        res: bool,
    )
        -> bool
    {
        if res {
            self.active.set(true);
            true
        } else {
            self.active.set(false);
            false
        }
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Causaloid id: {} \n Causaloid type: {} \n description: {} is active: {} has context: {}",
               self.id,
               self.causal_type,
               self.description,
               self.is_active(),
               self.has_context,
        )
    }
}

impl<'l, D, S, T, ST> Debug for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.fmt(f) }
}

impl<'l, D, S, T, ST> Display for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone + Copy,
        S: Spatial + Clone + Copy,
        T: Temporal + Clone + Copy,
        ST: SpaceTemporal + Clone + Copy,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { self.fmt(f) }
}
