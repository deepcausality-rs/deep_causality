/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::errors::CsmError;
use crate::traits::causable::MonadicCausable;
use crate::types::csm_types::csm::CsmEvaluable;
use crate::{CausalState, Causaloid, Context, Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use deep_causality_core::PropagatingEffect;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

impl<I, O, D, S, T, ST, SYM, VS, VT> CausalState<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default + Clone + Debug + Send + Sync,
    O: CsmEvaluable + Default + Debug + Clone + Send + Sync,
    D: Datable + Clone + Debug + Send + Sync,
    S: Spatial<VS> + Clone + Debug + Send + Sync,
    T: Temporal<VT> + Clone + Debug + Send + Sync,
    ST: SpaceTemporal<VS, VT> + Clone + Debug + Send + Sync,
    SYM: Symbolic + Clone + Debug + Send + Sync,
    VS: Clone + Send + Sync,
    VT: Clone + Send + Sync,
    Causaloid<I, O, (), Arc<RwLock<Context<D, S, T, ST, SYM, VS, VT>>>>: MonadicCausable<I, O>,
{
    /// Evaluates the state using its internal data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the internal data value.
    ///
    /// # Returns
    /// - `Ok(PropagatingEffect<O>)` if evaluation succeeds
    /// - `Err(CausalStateError)` if an error occurs during evaluation
    ///
    pub fn eval(&self) -> Result<PropagatingEffect<O>, CsmError> {
        Ok(self.causaloid.evaluate(&self.data))
    }

    /// Evaluates the state using provided external data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the provided data value, rather than the internal data.
    ///
    /// # Parameters
    /// - `data`: The numerical value to use for evaluation
    ///
    /// # Returns
    /// - `Ok(PropagatingEffect<O>)` if evaluation succeeds
    /// - `Err(CsmError)` if an error occurs during evaluation
    ///
    pub fn eval_with_data(
        &self,
        data: &PropagatingEffect<I>,
    ) -> Result<PropagatingEffect<O>, CsmError> {
        Ok(self.causaloid.evaluate(data))
    }
}
