// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Causable, Causaloid};
use crate::causal_state::CausalState;

struct CSM {}

impl CSM {
    pub fn new() -> Self {
        Self {}
    }
}

impl CSM {
    pub fn eval(c: &CausalState) {}
}