// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use deep_causality_macros::{Constructor, Getters};

use crate::prelude::ActionError;

#[derive(Getters, Constructor, Clone, Debug)]
pub struct CausalAction {
    action: fn() -> Result<(), ActionError>,
    descr: &'static str,
    version: usize,
}

impl CausalAction
{
    pub fn fire(&self) -> Result<(), ActionError>
    {
        (self.action)()
    }
}
