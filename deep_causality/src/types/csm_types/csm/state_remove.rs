/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CSM, Datable, IntoEffectValue, SpaceTemporal, Spatial, Symbolic, Temporal, UpdateError,
};
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
    /// Removes a state action at the index position id.
    /// Returns UpdateError if the index does not exist.
    pub fn remove_single_state(&self, id: usize) -> Result<(), UpdateError> {
        let mut binding = self.state_actions.write().unwrap();

        if binding.remove(&id).is_none() {
            return Err(UpdateError(format!(
                "State {id} does not exist and cannot be removed"
            )));
        }

        Ok(())
    }
}
