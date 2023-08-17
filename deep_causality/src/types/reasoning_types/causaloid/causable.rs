// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::collections::HashMap;

use crate::errors::CausalityError;
use crate::prelude::{Causable, CausableGraphExplaining, CausableGraphReasoning, CausableReasoning, Causaloid, Datable, IdentificationValue, NumericalValue, SpaceTemporal, Spatial, Temporal};
use crate::types::reasoning_types::causaloid::causal_type::CausalType;

impl<'l, D, S, T, ST> Causable for Causaloid<'l, D, S, T, ST>
    where
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
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
        D: Datable + Clone,
        S: Spatial + Clone,
        T: Temporal + Clone,
        ST: SpaceTemporal + Clone,
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
}