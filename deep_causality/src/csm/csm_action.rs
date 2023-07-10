// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::ActionError;

#[derive(Clone)]
pub struct CausalAction {
    action: fn() -> Result<(), ActionError>,
}

impl CausalAction
{
    pub fn new(action: fn() -> Result<(), ActionError>) -> Self {
        Self { action }
    }
}

impl CausalAction
{
    pub fn fire(&self) -> Result<(), ActionError>
    {
        (self.action)()
    }
}
