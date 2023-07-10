// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{ActionError, CausalAction, CausalState};


pub struct CSM<'l> {
    state_actions: &'l [(&'l CausalState<'l>, &'l CausalAction)],
}

impl<'l> CSM<'l>
{
    pub fn new(
        states: &'l [(&'l CausalState<'l>, &'l CausalAction)],
    )
        -> Self {
        Self { state_actions: states }
    }
}

impl<'l> CSM<'l>
{
    pub fn eval(&self) -> Result<(), ActionError>
    {
        for (state, action) in self.state_actions {
            let eval = state.eval();

            // check if the causal state evaluation returned an error
            if eval.is_err() {
                return Err(ActionError(format!("CSM[eval]: Error evaluating causal state: {}", state)));
            }

            // Unpack the bool result
            let trigger = eval
                .expect("CSM[eval]: Failed to unwrap evaluation result from causal state}");

            // If the state evaluated to true, fire the associated action.
            if trigger {
                if action.fire().is_err() {
                    return Err(ActionError(format!("CSM[eval]: Failed to fire action associated with causal state {}", state)));
                }
            }
        }

        Ok(())
    }
}