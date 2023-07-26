// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

use crate::prelude::*;
use crate::protocols::causable::Causable;

// Internal enum to represent the type of causaloid, which
// is required to redirect verify and explain method calls to
// either a singleton, a causal collection, or causal graph.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum CausalType {
    Singleton,
    Collection,
    Graph,
}

impl Display for CausalType { fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) } }


#[derive(Clone)]
pub struct Causaloid
{
    id: IdentificationValue,
    active: RefCell<bool>,
    causal_type: CausalType,
    causal_fn: CausalFn,
    causal_coll: Option<Vec<Causaloid>>,
    causal_graph: Option<CausaloidGraph<Causaloid>>,
    last_obs: RefCell<NumericalValue>,
    data_set_id: DescriptionValue,
    description: DescriptionValue,
}


impl Causaloid
{
    /// Singleton constructor. Assumes causality function is valid.
    /// Only use for non-fallible construction i.e.verified a-priori knowledge about the correctness of the causal function.
    pub fn new(
        id: IdentificationValue,
        causal_fn: CausalFn,
        data_set_id: DescriptionValue,
        description: DescriptionValue,
    )
        -> Self
    {
        Causaloid {
            id,
            active: RefCell::new(false),
            causal_type: CausalType::Singleton,
            causal_fn,
            causal_coll: None,
            causal_graph: None,
            last_obs: RefCell::new(0.0),
            data_set_id,
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
        causal_coll: Vec<Causaloid>,
        data_set_id: DescriptionValue,
        description: DescriptionValue,
    )
        -> Self
    {
        // empty causal function.
        fn causal_fn(_obs: NumericalValue) -> Result<bool, CausalityError> { Ok(false) }

        Causaloid {
            id,
            active: RefCell::new(false),
            causal_type: CausalType::Collection,
            causal_fn,
            causal_coll: Some(causal_coll),
            causal_graph: None,
            last_obs: RefCell::new(0.0),
            data_set_id,
            description,
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
        causal_graph: CausaloidGraph<Causaloid>,
        data_set_id: DescriptionValue,
        description: DescriptionValue,
    )
        -> Self
    {
        // empty causal function
        fn causal_fn(_obs: NumericalValue) -> Result<bool, CausalityError> { Ok(false) }

        Causaloid {
            id,
            active: RefCell::new(false),
            causal_type: CausalType::Graph,
            causal_fn,
            causal_coll: None,
            causal_graph: Some(causal_graph),
            last_obs: RefCell::new(0.0),
            data_set_id,
            description,
        }
    }
}


impl PartialEq for Causaloid {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Identifiable for Causaloid
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl Causable for Causaloid
{
    fn causal_collection(&self) -> Option<Vec<Causaloid>> {
        self.causal_coll.clone()
    }

    fn causal_graph(&self) -> Option<CausaloidGraph<Causaloid>> {
        self.causal_graph.clone()
    }

    fn description(&self) -> DescriptionValue {
        self.description.clone()
    }

    fn data_set_id(&self) -> DescriptionValue {
        self.data_set_id.clone()
    }

    fn explain(
        &self
    )
        -> Result<String, CausalityError>
    {
        return if *self.active.borrow()
        {
            match self.causal_type
            {
                CausalType::Singleton =>
                    {
                        let reason = format!("Causaloid: {} {} on last data {} evaluated to {}",
                                             self.id, self.description, self.last_obs.borrow(), self.is_active());
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
        *self.active.borrow()
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
        let res = match (self.causal_fn)(obs.to_owned()) {
            Ok(res) => {
                // store the applied data to provide details in explain()
                *self.last_obs.borrow_mut() = obs.to_owned();
                res
            }
            Err(e) => return Err(e),
        };

        Ok(self.check_active_return(res))
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
                        None => Err(CausalityError("Causal collection is None".into())),
                        Some(coll) =>
                            {
                                let res = match coll.reason_all_causes(data)
                                {
                                    Ok(res) => res,
                                    Err(e) => return Err(e),
                                };

                                Ok(self.check_active_return(res))
                            }
                    }
                }

            CausalType::Graph =>
                {
                    match &self.causal_graph
                    {
                        None => Err(CausalityError("Causal graph is None".into())),
                        Some(graph) =>
                            {
                                let res = match graph.reason_all_causes(data, data_index)
                                {
                                    Ok(res) => res,
                                    Err(e) => return Err(CausalityError(e.to_string())),
                                };

                                Ok(self.check_active_return(res))
                            }
                    }
                }
        }
    }
}


impl Causaloid
{
    #[inline(always)]
    fn check_active_return(
        &self,
        res: bool,
    )
        -> bool
    {
        return if res {
            *self.active.borrow_mut() = true;
            true
        } else {
            *self.active.borrow_mut() = false;
            false
        };
    }

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "Causaloid id: {} \n Causaloid type: {} \n description: {} is active: {}",
               self.id,
               self.causal_type,
               self.description,
               self.is_active(),
        )
    }
}

impl Debug for Causaloid
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        self.fmt(f)
    }
}

impl Display for Causaloid
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
    {
        self.fmt(f)
    }
}
