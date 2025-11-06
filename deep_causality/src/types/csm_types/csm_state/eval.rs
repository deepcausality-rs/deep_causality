/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CausalState, CsmError, Datable, MonadicCausable, PropagatingEffect, SpaceTemporal, Spatial,
    Symbolic, Temporal,
};

impl<D, S, T, ST, SYM, VS, VT> CausalState<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evaluates the state using its internal data.
    ///
    /// This method uses the state's causaloid to determine if the state's conditions
    /// are met based on the internal data value.
    ///
    /// # Returns
    /// - `Ok(true)` if the state's conditions are met
    /// - `Ok(false)` if the state's conditions are not met
    /// - `Err(CausalityError)` if an error occurs during evaluation
    ///
    pub fn eval(&self) -> Result<PropagatingEffect, CsmError> {
        Ok(self.causaloid.evaluate_monadic(self.data.clone()))
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
    /// - `Ok(true)` if the state's conditions are met with the provided data
    /// - `Ok(false)` if the state's conditions are not met with the provided data
    /// - `Err(CausalityError)` if an error occurs during evaluation
    ///
    /// ```texttext
    pub fn eval_with_data(&self, data: PropagatingEffect) -> Result<PropagatingEffect, CsmError> {
        Ok(self.causaloid.evaluate_monadic(data))
    }
}
