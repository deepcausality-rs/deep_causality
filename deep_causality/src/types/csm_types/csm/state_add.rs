/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CSM, CsmEvaluable, Datable, SpaceTemporal, Spatial, StateAction, Symbolic, Temporal,
    UpdateError,
};
use std::fmt::Debug;

#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> CSM<I, O, D, S, T, ST, SYM, VS, VT>
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
{
    /// Inserts a new state action at the index position idx.
    /// Returns UpdateError if the index already exists.
    pub fn add_single_state(
        &self,
        idx: usize,
        state_action: StateAction<I, O, D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(), UpdateError> {
        // Check if the key exists, if so return error
        if self.state_actions.read().unwrap().contains_key(&idx) {
            return Err(UpdateError(format!("State {idx} already exists.")));
        }

        // Insert the new state/action at the idx position
        self.state_actions
            .write()
            .unwrap()
            .insert(idx, state_action);

        Ok(())
    }
}
