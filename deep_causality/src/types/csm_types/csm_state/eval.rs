/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{
    CausalState, Causaloid, CsmError, CsmEvaluable, Datable, MonadicCausable, PropagatingEffect,
    SpaceTemporal, Spatial, Symbolic, Temporal,
};
use std::fmt::Debug;

impl<I, O, D, S, T, ST, SYM, VS, VT> CausalState<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: Default + Clone,
    O: CsmEvaluable + Default + Debug + Clone,
    D: Datable + Clone + Debug,
    S: Spatial<VS> + Clone + Debug,
    T: Temporal<VT> + Clone + Debug,
    ST: SpaceTemporal<VS, VT> + Clone + Debug,
    SYM: Symbolic + Clone + Debug,
    VS: Clone + Debug,
    VT: Clone + Debug,
    Causaloid<I, O, D, S, T, ST, SYM, VS, VT>: MonadicCausable<I, O>,
{
    /// Evaluates the state using its internal data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the internal data value.
    ///
    /// # Returns
    /// - `Ok(PropagatingEffect<O>)` if evaluation succeeds
    /// - `Err(CsmError)` if an error occurs during evaluation
    ///
    pub fn eval(&self) -> Result<PropagatingEffect<O>, CsmError> {
        let res = self.causaloid.evaluate(&self.data);
        match res.is_ok() {
            true => Ok(res),
            false => Err(CsmError::Causal(res.error.unwrap())),
        }
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
