// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Causable, Causaloid};
use crate::causal_action::CausalAction;
use crate::causal_state::CausalState;

struct CSM<'l> {
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
    // Remove unwrap and add proper error propagation
    pub fn eval(&self)
    {
        for (state, action) in self.state_actions{
            if state.eval().is_ok() {
                if state.eval().unwrap() {
                    action.fire().unwrap()
                }
            }
        }
    }
}