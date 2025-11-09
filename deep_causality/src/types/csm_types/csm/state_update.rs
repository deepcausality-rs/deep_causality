/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CSM, CausalAction, CausalState, Datable, IntoEffectValue, SpaceTemporal, Spatial, StateAction,
    Symbolic, Temporal, UpdateError,
};
use std::collections::HashMap;
use std::fmt::Debug;

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> CSM<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Datable + Clone + Debug,
    S: Spatial<VS> + Clone + Debug,
    T: Temporal<VT> + Clone + Debug,
    ST: SpaceTemporal<VS, VT> + Clone + Debug,
    SYM: Symbolic + Clone + Debug,
    VS: Clone + Debug,
    VT: Clone + Debug,
{
    /// Updates a causal state with a new state at the index position idx.
    /// Returns UpdateError if the update operation failed.
    pub fn update_single_state(
        &self,
        idx: usize,
        state_action: StateAction<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if not return error
        if !self.state_actions.read().unwrap().contains_key(&idx) {
            return Err(UpdateError(format!(
                "State {idx} does not exist. Add it first before updating."
            )));
        }

        // Update state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

        Ok(())
    }

    /// Updates all causal state with a new state collection.
    /// Note, this operation erases all previous states in the CSM by generating a new collection.
    /// Returns UpdateError if the update operation failed.
    pub fn update_all_states(
        &self,
        state_actions: &[(&CausalState<I, O, D, S, T, ST, SYM, VS, VT>, &CausalAction)],
    ) -> Result<(), UpdateError> {
        let mut state_map = HashMap::with_capacity(state_actions.len());

        for (state, action) in state_actions {
            state_map.insert(state.id(), ((*state).clone(), (*action).clone()));
        }

        // Replace the existing map with the newly generated one.
        *self.state_actions.write().unwrap() = state_map;
        Ok(())
    }
}
