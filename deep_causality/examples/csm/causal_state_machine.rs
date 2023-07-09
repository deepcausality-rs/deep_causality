// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Causable, Causaloid};
use crate::causal_state::CausalState;

struct CSM<'l> {
    states: &'l [&'l CausalState<'l>],
}

impl<'l> CSM<'l>
{
    pub fn new(states: &'l [&'l CausalState<'l>]) -> Self {
        Self { states }
    }
}

impl<'l> CSM<'l>
{
    pub fn eval() {}
}