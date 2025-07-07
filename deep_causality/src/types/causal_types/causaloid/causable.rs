/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::collections::HashMap;

use crate::errors::CausalityError;
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::types::causal_types::causaloid::causal_type::CausaloidType;
use crate::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausableReasoning,
    Causaloid, Datable, IdentificationValue, NumericalValue, Symbolic,
};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Causable for Causaloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn explain(&self) -> Result<String, CausalityError> {
        if self.is_active() {
            match self.causal_type {
                CausaloidType::Singleton => {
                    let reason = format!(
                        "Causaloid: {} {} evaluated to {}",
                        self.id,
                        self.description,
                        self.is_active()
                    );
                    Ok(reason)
                }

                CausaloidType::Collection => Ok(self.causal_coll.as_ref().unwrap().explain()),

                CausaloidType::Graph => {
                    match self.causal_graph.as_ref().unwrap().explain_all_causes() {
                        Ok(str) => Ok(str),
                        Err(e) => Err(CausalityError(e.to_string())),
                    }
                }
            }
        } else {
            let reason = format!(
                "Causaloid: {} has not been evaluated. Call verify() to activate it",
                self.id
            );
            Err(CausalityError(reason))
        }
    }

    fn is_active(&self) -> bool {
        match self.causal_type {
            CausaloidType::Singleton => *self.active.read().unwrap(),
            CausaloidType::Collection => self.causal_coll.as_ref().unwrap().number_active() > 0f64,
            CausaloidType::Graph => self.causal_graph.as_ref().unwrap().number_active() > 0f64,
        }
    }

    fn is_singleton(&self) -> bool {
        match self.causal_type {
            CausaloidType::Singleton => true,
            CausaloidType::Collection => false,
            CausaloidType::Graph => false,
        }
    }

    fn verify_single_cause(&self, obs: &NumericalValue) -> Result<bool, CausalityError> {
        if self.has_context {
            let contextual_causal_fn = self.context_causal_fn.ok_or_else(|| {
                CausalityError(format!(
                    "Causaloid {}: verify_single_cause: context_causal_fn is None",
                    self.id
                ))
            })?;

            let context = self.context.as_ref().ok_or_else(|| {
                CausalityError(format!(
                    "Causaloid {}: verify_single_cause:  context is None",
                    self.id
                ))
            })?;

            let res = (contextual_causal_fn)(obs, context)?;

            let mut guard = self.active.write().unwrap();
            *guard = res;

            Ok(res)
        } else {
            let causal_fn = self.causal_fn.ok_or_else(|| {
                CausalityError(format!(
                    "Causaloid {}: verify_single_cause:  causal_fn is is None",
                    self.id
                ))
            })?;

            let res = (causal_fn)(obs)?;

            let mut guard = self.active.write().unwrap();
            *guard = res;

            Ok(res)
        }
    }

    fn verify_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityError> {
        match self.causal_type {
            CausaloidType::Singleton => Err(CausalityError(
                "Causaloid is singleton. Call verify_single_cause instead.".into(),
            )),

            CausaloidType::Collection => match &self.causal_coll {
                None => Err(CausalityError(
                    "Causaloid::verify_all_causes: causal collection is None".into(),
                )),
                Some(coll) => coll.reason_all_causes(data),
            },

            CausaloidType::Graph => match &self.causal_graph {
                None => Err(CausalityError(
                    "Causaloid::verify_all_causes: Causal graph is None".into(),
                )),
                Some(graph) => {
                    let res = match graph.reason_all_causes(data, data_index) {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityError(e.to_string())),
                    };

                    Ok(res)
                }
            },
        }
    }
}
