/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CSM, CsmEvaluable, StateAction, UpdateError};
use std::fmt::Debug;

impl<I, O, C> CSM<I, O, C>
where
    I: Default + Clone,
    O: CsmEvaluable + Default + Debug + Clone,
    C: Clone,
{
    /// Inserts a new state action at the index position idx.
    /// Returns UpdateError if the index already exists.
    pub fn add_single_state(
        &self,
        idx: usize,
        state_action: StateAction<I, O, C>,
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
