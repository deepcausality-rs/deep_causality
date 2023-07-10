// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::cell::RefCell;
use crate::errors::ActionError;
use crate::prelude::{CausalAction, CausalState};

pub mod csm_action;
pub mod csm_state;

pub struct CSM<'l> {
    state_actions: RefCell<&'l [(&'l CausalState<'l>, &'l CausalAction)]>,
}

impl<'l> CSM<'l>
{
    pub fn new(state_actions: &'l [(&'l CausalState<'l>, &'l CausalAction)]) -> Self {
        Self { state_actions: RefCell::new(state_actions) }
    }
}

impl<'l> CSM<'l>
{

}

impl<'l> CSM<'l>
{
    pub fn len(&self) -> usize {
        self.state_actions.borrow().len()
    }

    pub fn update_all_states(&self, state_actions: &'l [(&'l CausalState<'l>, &'l CausalAction)]) {
        *self.state_actions.borrow_mut() = state_actions
    }

    pub fn eval_all_states(&self) -> Result<(), ActionError>
    {
        for (state, action) in self.state_actions.borrow().iter() {
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
