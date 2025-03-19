// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::*;

use crate::errors::CausalityError;
use crate::prelude::{
    Causable, CausableGraph, CausableGraphExplaining, CausableGraphReasoning, CausableReasoning,
    Causaloid, Datable, IdentificationValue, NumericalValue, SpaceTemporal, Spatial, Temporable,
};
use crate::types::reasoning_types::causaloid::causal_type::CausalType;

impl<D, S, T, ST, V> Causable for Causaloid<'_, D, S, T, ST, V>
where
    D: Datable + Clone,
    S: Spatial<V> + Clone,
    T: Temporable<V> + Clone,
    ST: SpaceTemporal<V> + Clone,
    V: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<V, Output = V>
        + Sub<V, Output = V>
        + Mul<V, Output = V>
        + Clone,
{
    fn explain(&self) -> Result<String, CausalityError> {
        if self.is_active() {
            match self.causal_type {
                CausalType::Singleton => {
                    let reason = format!(
                        "Causaloid: {} {} evaluated to {}",
                        self.id,
                        self.description,
                        self.is_active()
                    );
                    Ok(reason)
                }

                CausalType::Collection => Ok(self.causal_coll.as_ref().unwrap().explain()),

                CausalType::Graph => {
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
            CausalType::Singleton => *self.active.read().unwrap(),
            CausalType::Collection => self.causal_coll.as_ref().unwrap().number_active() > 0f64,
            CausalType::Graph => self.causal_graph.as_ref().unwrap().number_active() > 0f64,
        }
    }

    fn is_singleton(&self) -> bool {
        match self.causal_type {
            CausalType::Singleton => true,
            CausalType::Collection => false,
            CausalType::Graph => false,
        }
    }

    fn verify_single_cause(&self, obs: &NumericalValue) -> Result<bool, CausalityError> {
        if self.has_context {
            let contextual_causal_fn = self
                .context_causal_fn
                .expect("Causaloid::verify_single_cause: context_causal_fn is None");

            let context = self
                .context
                .expect("Causaloid::verify_single_cause: context is None");

            let res = (contextual_causal_fn)(obs.to_owned(), context)?;

            let mut guard = self.active.write().unwrap();
            *guard = res;

            Ok(res)
        } else {
            let causal_fn = self
                .causal_fn
                .expect("Causaloid::verify_single_cause: causal_fn is None");
            let res = (causal_fn)(obs.to_owned())?;

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
            CausalType::Singleton => Err(CausalityError(
                "Causaloid is singleton. Call verify_single_cause instead.".into(),
            )),

            CausalType::Collection => match &self.causal_coll {
                None => Err(CausalityError(
                    "Causaloid::verify_all_causes: causal collection is None".into(),
                )),
                Some(coll) => coll.reason_all_causes(data),
            },

            CausalType::Graph => match &self.causal_graph {
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
